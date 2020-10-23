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

mod message;
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

use crate::handler::message::Message;
use crate::handler::payload::MessagePayload;
use crate::handler::state::{get_message_prefix, MessageState};

pub struct MessageTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl MessageTransactionHandler {
    pub fn new() -> MessageTransactionHandler {
        MessageTransactionHandler {
            family_name: "message".into(),
            family_versions: vec!["1.0".into()],
            namespaces: vec![get_message_prefix()],
        }
    }
}

impl Default for MessageTransactionHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionHandler for MessageTransactionHandler {
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

        let payload = MessagePayload::new(request.get_payload())?;

        let mut state = MessageState::new(context);

        info!(
            "Payload: {} {} {}",
            payload.get_name(),
            payload.get_action(),
            payload.get_message_content(),
        );

        let message = state.get_message(payload.get_name().as_str())?;

        match payload.get_action().as_str() {
            "delete" => {
                if message.is_none() {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Invalid action: message does not exist",
                    )));
                }
                state.delete_message(payload.get_name().as_str())?;
            }
            "create" => {
                if message.is_none() {
                    let message = Message::new(payload.get_name());
                    state.set_message(payload.get_name().as_str(), message)?;
                    info!("Created message: {}", payload.get_name().as_str());
                } else {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Invalid action: Message already exists",
                    )));
                }
            }
            "add" => {
                if let Some(mut m) = message {

                    if m.get_participant1().is_empty() {
                        m.set_participant1(signer);
                    } else if m.get_participant2().is_empty() {
                        m.set_participant2(signer)
                    }

                    m.add_message(&payload.get_message_content(), signer)?;

                    m.display();

                    state.set_message(payload.get_name().as_str(), m)?;
                } else {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Invalid action: Add requires an existing message",
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
    let handler = MessageTransactionHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => {
            info!("{}", err);
            Err(err)
        }
    }
}
