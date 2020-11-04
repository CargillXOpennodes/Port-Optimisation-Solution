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
use std::time::Duration;
use std::default::Default;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

#[derive(Debug, Copy, Clone)]
enum DockingType {
    LOADING,
    DISCHARGE,
    ERROR
}

impl ToString for DockingType {
    fn to_string(&self) -> String {
        return match self {
            DockingType::LOADING => "LOADING".to_string(),
            DockingType::DISCHARGE => "DISCHARGE".to_string(),
            _ => "error".to_string()
        }
    }
}

impl Default for DockingType {
    fn default() -> DockingType {
        DockingType::ERROR
    }
}

#[derive(Debug, Clone, Default)]
pub struct Status {
    name: String,
    sender: String,
    participant1: String,
    participant2: String,
    participant1_short: String,
    participant2_short: String,
    docking_type: DockingType,
    eta: Option<Duration>,
    etb: Option<Duration>,
    ata: Option<Duration>,
    eto: Option<Duration>,
    ato: Option<Duration>,
    etc: Option<Duration>,
    etd: Option<Duration>,
    is_bunkering: Option<bool>,
    bunkering_time: Option<Duration>,
    logs: String,
}

impl ToString for Status {
    fn to_string(&self) -> String {
        let mut fields = vec![
            self.name.clone(),
            self.sender.clone(),
            self.participant1.clone(),
            self.participant2.clone(),
            self.docking_type.to_string().clone(),
            "none".to_string().clone(),
            "none".to_string().clone(),
            "none".to_string().clone(),
            "none".to_string().clone(),
            "none".to_string().clone(),
            "none".to_string().clone(),
            "none".to_string().clone(),
            "none".to_string().clone(),
            "none".to_string().clone(),
            self.logs.to_string().clone(),
        ];    
        
        fn set_option(vec: &mut Vec<String>, index: usize, field: Option<Duration>) {
            if let Some(n) = field {
                vec[index] = n.as_millis().to_string().clone();
            }
        }

        set_option(&mut fields, 5, self.eta);
        set_option(&mut fields, 6, self.etb);
        set_option(&mut fields, 7, self.ata);
        set_option(&mut fields, 8, self.eto);
        set_option(&mut fields, 9, self.ato);
        set_option(&mut fields, 10, self.etc);
        set_option(&mut fields, 11, self.etd);
        if let Some(n) = self.is_bunkering {
            fields[12] = n.to_string().clone();
        }
        set_option(&mut fields, 13, self.bunkering_time);
        fields.join(",")
    }

}

impl Status {
    pub fn new(name: String) -> Status {
        Status {
            name,
            sender: "P1".to_string(),
            participant1: String::from(""),
            participant2: String::from(""),
            participant1_short: String::from(""),
            participant2_short: String::from(""),
            docking_type: DockingType::ERROR,
            logs: String::from(""),
            ..Default::default()
        }
    }

    fn from_string(status_string: &str) -> Option<Status> {
        let items: Vec<&str> = status_string.split(',').collect();
        
        if items.len() != 15 {
            return None;
        }

        let mut s = Status {
            name: items[0].to_string(),
            sender: items[1].to_string(),
            participant1: String::from(""),
            participant2: String::from(""),
            participant1_short: String::from(""),
            participant2_short: String::from(""),
            logs: String::from(""),
            docking_type: DockingType::ERROR,
            ..Default::default()
        };

        s.docking_type = match items[4] {
            "LOADING" => DockingType::LOADING,
            "DISCHARGE" => DockingType::DISCHARGE,
            _ => DockingType::ERROR
        };

        s.set_participant1(items[2]);
        s.set_participant2(items[3]);

        s.eta = s.get_duration_from_string(items[5]);
        s.etb = s.get_duration_from_string(items[6]);
        s.ata = s.get_duration_from_string(items[7]);
        s.eto = s.get_duration_from_string(items[8]);
        s.ato = s.get_duration_from_string(items[9]);
        s.etc = s.get_duration_from_string(items[10]);
        s.etd = s.get_duration_from_string(items[11]);
        s.bunkering_time = s.get_duration_from_string(items[13]);

        s.is_bunkering = match items[12] {
            "true" => Some(true),
            "false" => Some(false),
            _ => None
        };

        Some(s)
    }

    pub fn serialize_statuses(statuses: HashMap<String, Status>) -> String {
        let mut status_strings: Vec<String> = vec![];
        for (_, status) in statuses {
            status_strings.push(status.to_string().clone());
        }
        status_strings.sort();
        status_strings.join("|")
    }

    pub fn deserialize_statuses(status_strings: &str) -> Option<HashMap<String, Status>> {
        let mut ret: HashMap<String, Status> = HashMap::new();
        let status_string_list: Vec<&str> = status_strings.split('|').collect();
        for s in status_string_list {
            let status = Status::from_string(s);
            match status {
                Some(status_item) => ret.insert(status_item.name.clone(), status_item),
                None => return None,
            };
        }
        Some(ret)
    }

    pub fn update_status(&mut self, status_string: &str, sender: &str) -> Result<(), ApplyError> {
        let items: Vec<&str> = status_string.split(',').collect();

        if sender != self.participant1 || items.len() != 11 {
            return Err(ApplyError::InvalidTransaction(format!(
                "Invalid: status_string : {} sender : {}",
                status_string,
                sender
            )));
        }
        self.docking_type = match items[0] {
            "LOADING" => DockingType::LOADING,
            "DISCHARGE" => DockingType::DISCHARGE,
            _ => DockingType::ERROR
        };
        if !items[1].is_empty() {
            self.eta = self.get_duration_from_string(items[1]);
        }
        if !items[2].is_empty() {
            self.etb = self.get_duration_from_string(items[2]);
        }
        if !items[3].is_empty() {
            self.ata = self.get_duration_from_string(items[3]);
        }
        if !items[4].is_empty() {
                self.eto = self.get_duration_from_string(items[4]);
        }
        if !items[5].is_empty() {
            self.ato = self.get_duration_from_string(items[5]);
        }
        if !items[6].is_empty() {
            self.etc = self.get_duration_from_string(items[6]);
        }
        if !items[7].is_empty() {
            self.etd = self.get_duration_from_string(items[7]);
        }
        if !items[9].is_empty() {
            self.bunkering_time = self.get_duration_from_string(items[9]);
        }
        
        if !items[8].is_empty() {
            self.is_bunkering = match items[8] {
                "true" => Some(true),
                "false" => Some(false),
                _ => None
            };
        }

        if !items[10].is_empty() {
            self.logs = self.logs.to_string().clone() + ";" + items[10];
        }
        Ok(())
    }

    pub fn display(&self) {
        info!(
            "
    name: {},
    sender: {},
    participant1: {},
    participant2: {},
    docking_type: {:?},
    eta: {:?},
    etb: {:?},
    ata: {:?},
    eto: {:?},
    ato: {:?},
    etc: {:?},
    etd: {:?},
    is_bunkering: {:?},
    bunkering_time: {:?},
    logs: {},
    ",
        self.name,
        self.sender,
        self.participant1,
        self.participant2,
        self.docking_type,
        self.eta,
        self.etb,
        self.ata,
        self.eto,
        self.ato,
        self.etc,
        self.etd,
        self.is_bunkering,
        self.bunkering_time,
        self.logs,
        );
    }

    pub fn get_participant1(&self) -> String {
        self.participant1.clone()
    }

    pub fn get_participant2(&self) -> String {
        self.participant2.clone()
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

    pub fn get_duration_from_string(&mut self, field_string: &str) -> Option<Duration> {
        match field_string.parse::<u64>() {
            Ok(n) => Some(Duration::from_millis(n)),
            Err(_err) => None
        }
    }
    
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.sender == other.sender
            && self.participant1 == other.participant1
            && self.participant2 == other.participant2
            && self.eta == other.eta
            && self.etb == other.etb
            && self.ata == other.ata
            && self.eto == other.eto
            && self.ato == other.ato
            && self.etc == other.etc
            && self.etd == other.etd
            && self.is_bunkering == other.is_bunkering
            && self.bunkering_time == other.bunkering_time
            && self.logs == other.logs
    }
}
