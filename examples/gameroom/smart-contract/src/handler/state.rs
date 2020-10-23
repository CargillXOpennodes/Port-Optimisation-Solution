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
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::str::from_utf8;

use crypto::digest::Digest;
use crypto::sha2::Sha512;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::{ApplyError, TransactionContext};
    } else {
        use sawtooth_sdk::processor::handler::{ApplyError, TransactionContext};
    }
}

use crate::handler::message::Message;

pub fn get_message_prefix() -> String {
    let mut sha = Sha512::new();
    sha.input_str("message");
    sha.result_str()[..6].to_string()
}

pub struct MessageState<'a> {
    context: &'a mut dyn TransactionContext,
    address_map: HashMap<String, Option<String>>,
}

impl<'a> MessageState<'a> {
    pub fn new(context: &'a mut dyn TransactionContext) -> MessageState {
        MessageState {
            context,
            address_map: HashMap::new(),
        }
    }

    fn calculate_address(name: &str) -> String {
        let mut sha = Sha512::new();
        sha.input_str(name);
        get_message_prefix() + &sha.result_str()[..64].to_string()
    }

    pub fn delete_message(&mut self, message_name: &str) -> Result<(), ApplyError> {
        let mut messages = self._load_messages(message_name)?;
        messages.remove(message_name);
        if messages.is_empty() {
            self._delete_message(message_name)?;
        } else {
            self._store_message(message_name, messages)?;
        }
        Ok(())
    }

    pub fn set_message(&mut self, message_name: &str, m: Message) -> Result<(), ApplyError> {
        let mut messages = self._load_messages(message_name)?;
        messages.insert(message_name.to_string(), m);
        self._store_message(message_name, messages)?;
        Ok(())
    }

    pub fn get_message(&mut self, message_name: &str) -> Result<Option<Message>, ApplyError> {
        let messages = self._load_messages(message_name)?;
        if messages.contains_key(message_name) {
            Ok(Some(messages[message_name].clone()))
        } else {
            Ok(None)
        }
    }

    fn _store_message(
        &mut self,
        message_name: &str,
        messages: HashMap<String, Message>,
    ) -> Result<(), ApplyError> {
        let address = MessageState::calculate_address(message_name);
        let state_string = Message::serialize_messages(messages);
        self.address_map
            .insert(address.clone(), Some(state_string.clone()));
        self.context
            .set_state_entry(address, state_string.into_bytes())?;
        Ok(())
    }

    fn _delete_message(&mut self, message_name: &str) -> Result<(), ApplyError> {
        let address = MessageState::calculate_address(message_name);
        if self.address_map.contains_key(&address) {
            self.address_map.insert(address.clone(), None);
        }
        self.context.delete_state_entry(&address)?;
        Ok(())
    }

    fn _load_messages(&mut self, message_name: &str) -> Result<HashMap<String, Message>, ApplyError> {
        let address = MessageState::calculate_address(message_name);

        Ok(match self.address_map.entry(address.clone()) {
            Entry::Occupied(entry) => match entry.get() {
                Some(addr) => Message::deserialize_messages(addr).ok_or_else(|| {
                    ApplyError::InvalidTransaction("Invalid serialization of message state".into())
                })?,
                None => HashMap::new(),
            },
            Entry::Vacant(entry) => match self.context.get_state_entry(&address)? {
                Some(state_bytes) => {
                    let state_string = from_utf8(&state_bytes).map_err(|e| {
                        ApplyError::InvalidTransaction(format!(
                            "Invalid serialization of message state: {}",
                            e
                        ))
                    })?;

                    entry.insert(Some(state_string.to_string()));

                    Message::deserialize_messages(state_string).ok_or_else(|| {
                        ApplyError::InvalidTransaction("Invalid serialization of message state".into())
                    })?
                }
                None => {
                    entry.insert(None);
                    HashMap::new()
                }
            },
        })
    }
}
