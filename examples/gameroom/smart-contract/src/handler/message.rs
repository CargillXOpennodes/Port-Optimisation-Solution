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

use std::collections::HashMap;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

// const POSSIBLE_WINS: [(usize, usize, usize); 8] = [
//     (1, 2, 3),
//     (4, 5, 6),
//     (7, 8, 9),
//     (1, 4, 7),
//     (2, 5, 8),
//     (3, 6, 9),
//     (1, 5, 9),
//     (3, 5, 7),
// ];
#[derive(Debug, Copy, Clone)]
enum MessageType {
    TEXT,
    ERROR
}

impl ToString for MessageType {
    fn to_string(&self) -> String {
        return match self {
            MessageType::TEXT => "TEXT".to_string(),
            _ => "error".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    name: String,
    message_content: String,
    message_type: MessageType,
    id: u32,
    previous_id: Option<u32>,
    sender: String,
    participant1: String,
    participant2: String,
    participant1_short: String,
    participant2_short: String,
}

impl ToString for Message {
    fn to_string(&self) -> String {
        let mut fields = vec![
            self.name.clone(),
            self.message_content.clone(),
            self.message_type.to_string().clone(),
            self.id.to_string().clone(),
            (-1).to_string(),
            self.participant1.clone(),
            self.participant2.clone(),
        ];
        if let Some(n) = self.previous_id {
            fields[4] = n.to_string();
        }
        fields.join(",")
    }
}

impl Message {
    pub fn new(name: String) -> Message {
        Message {
            name,
            message_content: "Chat Created".to_string(),
            message_type: MessageType::TEXT,
            id: 0,
            previous_id: None,
            sender: "P1".to_string(),
            participant1: String::from(""),
            participant2: String::from(""),
            participant1_short: String::from(""),
            participant2_short: String::from(""),
        }
    }

    fn from_string(message_string: &str) -> Option<Message> {
        let items: Vec<&str> = message_string.split(',').collect();
        
        if items.len() != 8 {
            return None;
        }

        let mut m = Message {
            name: items[0].to_string(),
            message_content: items[1].to_string(),
            message_type: MessageType::TEXT,
            id: items[3].parse::<u32>().unwrap(),
            previous_id: None,
            sender: items[5].to_string(),
            participant1: String::from(""),
            participant2: String::from(""),
            participant1_short: String::from(""),
            participant2_short: String::from(""),
        };

        m.message_type = match items[2] {
            "TEXT" => MessageType::TEXT,
            _ => MessageType::ERROR
        };

        m.set_previous_id(items[4]);
        m.set_participant1(items[6]);
        m.set_participant2(items[7]);
        Some(m)
    }

    pub fn serialize_messages(messages: HashMap<String, Message>) -> String {
        let mut message_strings: Vec<String> = vec![];
        for (_, message) in messages {
            message_strings.push(message.to_string().clone());
        }
        message_strings.sort();
        message_strings.join("|")
    }

    pub fn deserialize_messages(messages_string: &str) -> Option<HashMap<String, Message>> {
        let mut ret: HashMap<String, Message> = HashMap::new();
        let message_string_list: Vec<&str> = messages_string.split('|').collect();
        for m in message_string_list {
            let message = Message::from_string(m);
            match message {
                Some(message_item) => ret.insert(message_item.name.clone(), message_item),
                None => return None,
            };
        }
        Some(ret)
    }

    pub fn add_message(&mut self, message_content: &str, sender: &str) -> Result<(), ApplyError> {
        self.message_content = message_content.to_string();
        self.sender = sender.to_string();

        self.previous_id = Some(self.id);
        self.id += 1;

        Ok(())
    }

    pub fn display(&self) {
        let prev_id =  match self.previous_id {
            Some(n) => n.to_string(),
            None => "None".to_string()
        };

        info!(
            "
    name: {}
    message: {}
    message type: {}
    id: {}
    previous id: {}
    sender: {}
    participant 1: {}
    participant 2: {}
    ",
        self.name,
        self.message_content,
        self.message_type.to_string(),
        self.id,
        prev_id,
        self.sender,
        self.participant1_short,
        self.participant2_short
        );
    }

    pub fn get_participant1(&self) -> String {
        self.participant1.clone()
    }

    pub fn get_participant2(&self) -> String {
        self.participant2.clone()
    }

    pub fn get_message_content(&self) -> String {
        self.message_content.clone()
    }

    pub fn set_participant1(&mut self, p1: &str) {
        self.participant1 = p1.to_string();
        if p1.len() > 6 {
            self.participant1_short = p1[..6].to_string();
        } else {
            self.participant1_short = String::from(p1);
        }
    }

    pub fn set_participant2(&mut self, p2: &str) {
        self.participant2 = p2.to_string();
        if p2.len() > 6 {
            self.participant2_short = p2[..6].to_string();
        } else {
            self.participant2_short = String::from(p2);
        }
    }

    pub fn set_previous_id(&mut self, prev_id: &str) {
        self.previous_id = match prev_id.parse::<u32>() {
            Ok(n) => Some(n),
            Err(_n) => None
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.message_content == other.message_content
            && self.id == other.id
            && self.previous_id == other.previous_id
            && self.sender == other.sender
            && self.participant1 == other.participant1
            && self.participant2 == other.participant2
    }
}
