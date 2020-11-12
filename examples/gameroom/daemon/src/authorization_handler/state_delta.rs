/*
 * Copyright 2018-2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use std::{error::Error, fmt, time::{SystemTime, Duration, UNIX_EPOCH}};

use diesel::connection::Connection;
use gameroom_database::{
    error, helpers,
    models::Status,
    ConnectionPool,
};
use scabbard::service::{StateChange, StateChangeEvent};

use crate::authorization_handler::sabre::{get_status_contract_address, STATUS_PREFIX};
use crate::authorization_handler::AppAuthHandlerError;

pub struct StatusStateDeltaProcessor {
    circuit_id: String,
    node_id: String,
    requester: String,
    contract_address: String,
    db_pool: ConnectionPool,
}

impl StatusStateDeltaProcessor {
    pub fn new(
        circuit_id: &str,
        node_id: &str,
        requester: &str,
        db_pool: &ConnectionPool,
    ) -> Result<Self, AppAuthHandlerError> {
        Ok(StatusStateDeltaProcessor {
            circuit_id: circuit_id.into(),
            node_id: node_id.to_string(),
            requester: requester.to_string(),
            contract_address: get_status_contract_address()?,
            db_pool: db_pool.clone(),
        })
    }

    pub fn handle_state_change_event(
        &self,
        change_event: StateChangeEvent,
    ) -> Result<(), StateDeltaError> {
        // Update the last seen state change event
        let time = SystemTime::now();
        let conn = &*self.db_pool.get()?;
        conn.transaction::<_, error::DatabaseError, _>(|| {
            helpers::update_gameroom_service_last_event(
                &conn,
                &self.circuit_id,
                &time,
                &change_event.id,
            )?;
            Ok(())
        })?;

        for change in change_event.state_changes {
            self.handle_state_change(&change)?;
        }

        Ok(())
    }

    fn handle_state_change(&self, change: &StateChange) -> Result<(), StateDeltaError> {
        debug!("Received state change: {}", change);
        debug!("Contract address: {}", self.contract_address);
        match change {
            StateChange::Set { key, .. } if key == &self.contract_address => {
                debug!("Status contract created successfully");
                let time = SystemTime::now();
                let conn = &*self.db_pool.get()?;
                conn.transaction::<_, error::DatabaseError, _>(|| {
                    let notification = helpers::create_new_notification(
                        "circuit_active",
                        &self.requester,
                        &self.node_id,
                        &self.circuit_id,
                    );
                    helpers::insert_gameroom_notification(&conn, &[notification])?;
                    helpers::update_gameroom_status(&conn, &self.circuit_id, &time, "Active")?;
                    helpers::update_gameroom_member_status(
                        &conn,
                        &self.circuit_id,
                        &time,
                        "Ready",
                        "Active",
                    )?;
                    helpers::update_gameroom_service_status(
                        &conn,
                        &self.circuit_id,
                        &time,
                        "Ready",
                        "Active",
                    )?;

                    Ok(())
                })
                    .map_err(StateDeltaError::from)
            }
            StateChange::Set { key, value } if &key[..6] == STATUS_PREFIX => {
                let time = SystemTime::now();
                let status_state: Vec<String> = String::from_utf8(value.to_vec())
                    .map_err(|err| StateDeltaError::StatusPayloadParseError(format!("{:?}", err)))
                    .map(|s| s.split(',').map(String::from).collect())?;

                let conn = &*self.db_pool.get()?;
                let status_name = status_state[0].to_string().clone();
                conn.transaction::<_, error::DatabaseError, _>(|| {
                    let mut m;
                    let notification;
                    debug!("{:?}", status_state);
                    match helpers::fetch_status(&conn, &self.circuit_id, &status_name)? {
                        Some(status) => {
                            m = Status {
                                id: 0,
                                status_name,
                                circuit_id: self.circuit_id.to_string().clone(),
                                sender: status_state[1].to_string().clone(),
                                participant_1: "".to_string(),
                                participant_2: status_state[3].to_string().clone(),
                                eta: None,
                                etb: None,
                                ata: None,
                                eto: None,
                                ato: None,
                                etc: None,
                                etd: None,
                                is_bunkering: None,
                                bunkering_time: None,
                                logs: status_state[14].to_string().clone(),
                                updated_time: time,
                                ..status
                            };
                            let epoch = time
                                .checked_sub(time
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or(Duration::from_nanos(0)));

                            m.is_bunkering = match status_state[12].as_str() {
                                "true" => Some(true),
                                "false" => Some(false),
                                _ => None
                            };
                            if let Some(epoch_time) = epoch {
                                if let Ok(n) = status_state[5].parse::<u64>() {
                                    let eta = epoch_time.checked_add(Duration::from_millis(n));
                                    m.eta = eta;
                                }
                                if let Ok(n) = status_state[6].parse::<u64>() {
                                    let etb = epoch_time.checked_add(Duration::from_millis(n));
                                    m.etb = etb;
                                }
                                if let Ok(n) = status_state[7].parse::<u64>() {
                                    let ata = epoch_time.checked_add(Duration::from_millis(n));
                                    m.ata = ata;
                                }
                                if let Ok(n) = status_state[8].parse::<u64>() {
                                    let eto = epoch_time.checked_add(Duration::from_millis(n));
                                    m.eto = eto;
                                }
                                if let Ok(n) = status_state[9].parse::<u64>() {
                                    let ato = epoch_time.checked_add(Duration::from_millis(n));
                                    m.ato = ato;
                                }
                                if let Ok(n) = status_state[10].parse::<u64>() {
                                    let etc = epoch_time.checked_add(Duration::from_millis(n));
                                    m.etc = etc;
                                }
                                if let Ok(n) = status_state[11].parse::<u64>() {
                                    let etd = epoch_time.checked_add(Duration::from_millis(n));
                                    m.etd = etd;
                                }
                                if let Ok(n) = status_state[13].parse::<u64>() {
                                    let bunkering_time = epoch_time.checked_add(Duration::from_millis(n));
                                    m.bunkering_time = bunkering_time;
                                }
                            }
                            notification = helpers::create_new_notification(
                                &format!("new_status_created:{}", status_state[0]),
                                &self.requester,
                                &self.node_id,
                                &self.circuit_id,
                            );
                            helpers::update_status(
                                &conn, m,
                            )?;
                        }
                        None => {
                            m = Status {
                                id: 0,
                                status_name,
                                circuit_id: self.circuit_id.to_string().clone(),
                                sender: status_state[1].to_string().clone(),
                                participant_1: "".to_string(),
                                participant_2: status_state[3].to_string().clone(),
                                eta: None,
                                etb: None,
                                ata: None,
                                eto: None,
                                ato: None,
                                etc: None,
                                etd: None,
                                is_bunkering: None,
                                bunkering_time: None,
                                logs: status_state[14].to_string().clone(),
                                created_time: time,
                                updated_time: time
                            };
                            let epoch = time.checked_sub(time.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(0)));
                            m.is_bunkering = match status_state[12].as_str() {
                                "true" => Some(true),
                                "false" => Some(false),
                                _ => None
                            };
                            if let Some(epoch_time) = epoch {
                                if let Ok(n) = status_state[5].parse::<u64>() {
                                    let eta = epoch_time.checked_add(Duration::from_millis(n));
                                    m.eta = eta;
                                }
                                if let Ok(n) = status_state[6].parse::<u64>() {
                                    let etb = epoch_time.checked_add(Duration::from_millis(n));
                                    m.etb = etb;
                                }
                                if let Ok(n) = status_state[7].parse::<u64>() {
                                    let ata = epoch_time.checked_add(Duration::from_millis(n));
                                    m.ata = ata;
                                }
                                if let Ok(n) = status_state[8].parse::<u64>() {
                                    let eto = epoch_time.checked_add(Duration::from_millis(n));
                                    m.eto = eto;
                                }
                                if let Ok(n) = status_state[9].parse::<u64>() {
                                    let ato = epoch_time.checked_add(Duration::from_millis(n));
                                    m.ato = ato;
                                }
                                if let Ok(n) = status_state[10].parse::<u64>() {
                                    let etc = epoch_time.checked_add(Duration::from_millis(n));
                                    m.etc = etc;
                                }
                                if let Ok(n) = status_state[11].parse::<u64>() {
                                    let etd = epoch_time.checked_add(Duration::from_millis(n));
                                    m.etd = etd;
                                }
                                if let Ok(n) = status_state[13].parse::<u64>() {
                                    let bunkering_time = epoch_time.checked_add(Duration::from_millis(n));
                                    m.bunkering_time = bunkering_time;
                                }
                            }

                            notification = helpers::create_new_notification(
                                &format!("status_updated:{}", status_state[0]),
                                &self.requester,
                                &self.node_id,
                                &self.circuit_id,
                            );
                            helpers::insert_status(
                                &conn, m,
                            )?;
                        }
                    };

                    helpers::insert_gameroom_notification(&conn, &[notification])?;



                    Ok(())
                })
                .map_err(StateDeltaError::from)
            }
            StateChange::Delete {
                ..
            } => {
                debug!("Delete state skipping...");
                Ok(())
            }
            _ => {
            
                debug!("Unrecognized state change skipping...");
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum StateDeltaError {
    StatusPayloadParseError(String),
    DatabaseError(error::DatabaseError),
}

impl Error for StateDeltaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StateDeltaError::StatusPayloadParseError(_) => None,
            StateDeltaError::DatabaseError(err) => Some(err),
        }
    }
}

impl fmt::Display for StateDeltaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StateDeltaError::StatusPayloadParseError(err) => {
                write!(f, "Failed to parse xo payload: {}", err)
            }
            StateDeltaError::DatabaseError(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl From<error::DatabaseError> for StateDeltaError {
    fn from(err: error::DatabaseError) -> Self {
        StateDeltaError::DatabaseError(err)
    }
}