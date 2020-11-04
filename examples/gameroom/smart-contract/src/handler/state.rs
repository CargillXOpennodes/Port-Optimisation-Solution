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

use crate::handler::status::Status;

pub fn get_status_prefix() -> String {
    let mut sha = Sha512::new();
    sha.input_str("sawtooth_message");
    sha.result_str()[..6].to_string()
}

pub struct StatusState<'a> {
    context: &'a mut dyn TransactionContext,
    address_map: HashMap<String, Option<String>>,
}

impl<'a> StatusState<'a> {
    pub fn new(context: &'a mut dyn TransactionContext) -> StatusState {
        StatusState {
            context,
            address_map: HashMap::new(),
        }
    }

    fn calculate_address(name: &str) -> String {
        let mut sha = Sha512::new();
        sha.input_str(name);
        get_status_prefix() + &sha.result_str()[..64].to_string()
    }

    pub fn delete_status(&mut self, status_name: &str) -> Result<(), ApplyError> {
        let mut statuses = self._load_statuses(status_name)?;
        statuses.remove(status_name);
        if statuses.is_empty() {
            self._delete_status(status_name)?;
        } else {
            self._store_status(status_name, statuses)?;
        }
        Ok(())
    }

    pub fn set_status(&mut self, status_name: &str, s: Status) -> Result<(), ApplyError> {
        let mut statuses = self._load_statuses(status_name)?;
        statuses.insert(status_name.to_string(), s);
        self._store_status(status_name, statuses)?;
        Ok(())
    }

    pub fn get_status(&mut self, status_name: &str) -> Result<Option<Status>, ApplyError> {
        let statuses = self._load_statuses(status_name)?;
        if statuses.contains_key(status_name) {
            Ok(Some(statuses[status_name].clone()))
        } else {
            Ok(None)
        }
    }

    fn _store_status(
        &mut self,
        status_name: &str,
        statuses: HashMap<String, Status>,
    ) -> Result<(), ApplyError> {
        let address = StatusState::calculate_address(status_name);
        let state_string = Status::serialize_statuses(statuses);
        self.address_map
            .insert(address.clone(), Some(state_string.clone()));
        self.context
            .set_state_entry(address, state_string.into_bytes())?;
        Ok(())
    }

    fn _delete_status(&mut self, status_name: &str) -> Result<(), ApplyError> {
        let address = StatusState::calculate_address(status_name);
        if self.address_map.contains_key(&address) {
            self.address_map.insert(address.clone(), None);
        }
        self.context.delete_state_entry(&address)?;
        Ok(())
    }

    fn _load_statuses(&mut self, status_name: &str) -> Result<HashMap<String, Status>, ApplyError> {
        let address = StatusState::calculate_address(status_name);

        Ok(match self.address_map.entry(address.clone()) {
            Entry::Occupied(entry) => match entry.get() {
                Some(addr) => Status::deserialize_statuses(addr).ok_or_else(|| {
                    ApplyError::InvalidTransaction("Invalid serialization of status state".into())
                })?,
                None => HashMap::new(),
            },
            Entry::Vacant(entry) => match self.context.get_state_entry(&address)? {
                Some(state_bytes) => {
                    let state_string = from_utf8(&state_bytes).map_err(|e| {
                        ApplyError::InvalidTransaction(format!(
                            "Invalid serialization of status state: {}",
                            e
                        ))
                    })?;

                    entry.insert(Some(state_string.to_string()));

                    Status::deserialize_statuses(state_string).ok_or_else(|| {
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
