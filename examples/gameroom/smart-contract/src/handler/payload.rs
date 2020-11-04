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

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

pub struct StatusPayload {
    name: String,
    action: String,
    status_string: String,
}

impl StatusPayload {
    // payload_data is a utf-8 encoded string
    pub fn new(payload_data: &[u8]) -> Result<StatusPayload, ApplyError> {
        let payload_string = match ::std::str::from_utf8(&payload_data) {
            Ok(s) => s,
            Err(_) => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Invalid payload serialization",
                )));
            }
        };

        let items: Vec<&str> = payload_string.split(',').collect();

        if items.len() != 13 {
            return Err(ApplyError::InvalidTransaction(String::from(
                "Payload must have exactly 12 commas",
            )));
        }

        let (name, action) = (items[0], items[1]);
        
        let mut status_string = items[2].to_string().clone();
        for i in 3..13 {
            status_string = status_string + "," + items[i];
        }

        if name.is_empty() {
            return Err(ApplyError::InvalidTransaction(String::from(
                "Name is required",
            )));
        }

        if action.is_empty() {
            return Err(ApplyError::InvalidTransaction(String::from(
                "Action is required",
            )));
        }

        if name.contains('|') {
            return Err(ApplyError::InvalidTransaction(String::from(
                "Name cannot contain |",
            )));
        }
        match action {
            "create" | "delay" | "prepone" | "delete" => (),
            _ => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    format!("Invalid action: {}", action).as_str(),
                )));
            }
        };

        Ok(StatusPayload {
            name: name.to_string(),
            action: action.to_string(),
            status_string: status_string.to_string()
        })
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_action(&self) -> String {
        self.action.clone()
    }    
    
    pub fn get_status_string(&self) -> String {
        self.status_string.clone()
    }

}
