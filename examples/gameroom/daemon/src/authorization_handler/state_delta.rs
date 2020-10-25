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

use std::{error::Error, fmt, time::SystemTime};

use diesel::connection::Connection;
use gameroom_database::{
    error, helpers,
    models::{Message, MessageType},
    ConnectionPool,
};
use scabbard::service::{StateChange, StateChangeEvent};

use crate::authorization_handler::sabre::{get_message_contract_address, MESSAGE_PREFIX};
use crate::authorization_handler::AppAuthHandlerError;

pub struct MessageStateDeltaProcessor {
    circuit_id: String,
    node_id: String,
    requester: String,
    contract_address: String,
    db_pool: ConnectionPool,
}

impl MessageStateDeltaProcessor {
    pub fn new(
        circuit_id: &str,
        node_id: &str,
        requester: &str,
        db_pool: &ConnectionPool,
    ) -> Result<Self, AppAuthHandlerError> {
        Ok(MessageStateDeltaProcessor {
            circuit_id: circuit_id.into(),
            node_id: node_id.to_string(),
            requester: requester.to_string(),
            contract_address: get_message_contract_address()?,
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
        match change {
            StateChange::Set { key, .. } if key == &self.contract_address => {
                debug!("Message contract created successfully");
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
            StateChange::Set { key, value } if &key[..6] == MESSAGE_PREFIX => {
                let time = SystemTime::now();
                let message_state: Vec<String> = String::from_utf8(value.to_vec())
                    .map_err(|err| StateDeltaError::MessagePayloadParseError(format!("{:?}", err)))
                    .map(|s| s.split(',').map(String::from).collect())?;

                let conn = &*self.db_pool.get()?;
                conn.transaction::<_, error::DatabaseError, _>(|| {
                    let m;
                    let notification;
                    match helpers::get_latest_message(&self.circuit_id, &conn)? {
                        Some(message) => {
                            m = Message {
                                message_content: message_state[1].to_string(),
                                message_type: MessageType::TEXT.to_string(),
                                id: message_state[3].parse::<i32>().unwrap(),
                                previous_id: Some(message_state[4].to_string().parse::<i32>().unwrap()),
                                sender: message_state[5].to_string(),
                                updated_time: time.clone(),
                                participant_2: message_state[6].to_string(),
                                ..message
                            };

                            notification = helpers::create_new_notification(
                                &format!("new_message_created:{}", message_state[0]),
                                &self.requester,
                                &self.node_id,
                                &self.circuit_id,
                            );
                        }
                        None => {
                            m = Message {
                                message_name: message_state[0].to_string(),
                                message_content: message_state[1].to_string(),
                                message_type: MessageType::TEXT.to_string(),
                                id: message_state[3].parse::<i32>().unwrap(),
                                previous_id: None,
                                sender: message_state[5].to_string(),
                                participant_1: message_state[5].to_string(),
                                participant_2: message_state[6].to_string(),
                                created_time: time.clone(),
                                circuit_id: self.circuit_id.clone(),
                                updated_time: time.clone(),
                            };

                            notification = helpers::create_new_notification(
                                &format!("message_updated:{}", message_state[0]),
                                &self.requester,
                                &self.node_id,
                                &self.circuit_id,
                            );
                        }
                    };

                    helpers::insert_gameroom_notification(&conn, &[notification])?;

                    helpers::add_message(
                        &conn, m,
                    )?;

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
    MessagePayloadParseError(String),
    DatabaseError(error::DatabaseError),
}

impl Error for StateDeltaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StateDeltaError::MessagePayloadParseError(_) => None,
            StateDeltaError::DatabaseError(err) => Some(err),
        }
    }
}

impl fmt::Display for StateDeltaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StateDeltaError::MessagePayloadParseError(err) => {
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