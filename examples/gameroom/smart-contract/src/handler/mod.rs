/*
 * Copyright 2018 Bitwise IO, Inc.
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

mod status;
mod payload;
mod state;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::{ApplyError, TpProcessRequest, TransactionContext, TransactionHandler,};
    } else {
        use sawtooth_sdk::messages::processor::TpProcessRequest;
        use sawtooth_sdk::processor::handler::{ApplyError, TransactionContext, TransactionHandler};
    }
}

use crate::handler::status::Status;
use crate::handler::payload::StatusPayload;
use crate::handler::state::{get_status_prefix, StatusState};

pub struct StatusTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl StatusTransactionHandler {
    pub fn new() -> StatusTransactionHandler {
        StatusTransactionHandler {
            family_name: "sawtooth_message".into(),
            family_versions: vec!["1.0".into()],
            namespaces: vec![get_status_prefix()],
        }
    }
}

impl Default for StatusTransactionHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionHandler for StatusTransactionHandler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.namespaces.clone()
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext,
    ) -> Result<(), ApplyError> {
        let signer = request.get_header().get_signer_public_key();

        let payload = StatusPayload::new(request.get_payload())?;

        let mut state = StatusState::new(context);

        info!(
            "Payload: {} {} {}",
            payload.get_name(),
            payload.get_action(),
            payload.get_status_string(),
        );

        let status = state.get_status(payload.get_name().as_str())?;

        match payload.get_action().as_str() {
            "delete" => {
                if status.is_none() {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Invalid action: status does not exist",
                    )));
                }
                state.delete_status(payload.get_name().as_str())?;
            }
            "create" => {
                if status.is_none() {
                    let mut new_status = Status::new(payload.get_name());
                    new_status.set_participant1(signer);
                    state.set_status(payload.get_name().as_str(), new_status)?;
                    info!("Created status: {}", payload.get_name().as_str());
                } else {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Invalid action: Status already exists",
                    )));
                }
            }
            "delay" => {
                if let Some(mut s) = status {
                    if s.get_participant2().is_empty() {
                        s.set_participant2(signer);
                    }

                    s.update_status(&payload.get_status_string(), signer)?;

                    s.display();

                    state.set_status(payload.get_name().as_str(), s)?;
                } else {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Invalid action: Add requires an existing status",
                    )));
                }
            }
            "prepone" => {
                if let Some(mut s) = status {
                    if s.get_participant2().is_empty() {
                        s.set_participant2(signer);
                    }

                    s.update_status(&payload.get_status_string(), signer)?;

                    s.display();

                    state.set_status(payload.get_name().as_str(), s)?;
                } else {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Invalid action: Add requires an existing status",
                    )));
                }
            }
            other_action => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Invalid action: '{}'",
                    other_action
                )));
            }
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
// Sabre apply must return a bool
pub fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = StatusTransactionHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => {
            info!("{}", err);
            Err(err)
        }
    }
}
