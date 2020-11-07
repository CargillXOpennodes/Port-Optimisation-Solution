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

use super::schema::*;
use std::time::{SystemTime, Duration};

#[derive(Insertable, Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "gameroom"]
#[primary_key(circuit_id)]
pub struct Gameroom {
    pub circuit_id: String,
    pub authorization_type: String,
    pub persistence: String,
    pub durability: String,
    pub routes: String,
    pub circuit_management_type: String,
    pub alias: String,
    pub status: String,
    pub created_time: SystemTime,
    pub updated_time: SystemTime,
}

#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[table_name = "gameroom_proposal"]
#[belongs_to(Gameroom, foreign_key = "circuit_id")]
pub struct GameroomProposal {
    pub id: i64,
    pub proposal_type: String,
    pub circuit_id: String,
    pub circuit_hash: String,
    pub requester: String,
    pub requester_node_id: String,
    pub status: String,
    pub created_time: SystemTime,
    pub updated_time: SystemTime,
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "gameroom_proposal"]
pub struct NewGameroomProposal {
    pub proposal_type: String,
    pub circuit_id: String,
    pub circuit_hash: String,
    pub requester: String,
    pub requester_node_id: String,
    pub status: String,
    pub created_time: SystemTime,
    pub updated_time: SystemTime,
}

#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[table_name = "proposal_vote_record"]
#[belongs_to(GameroomProposal, foreign_key = "proposal_id")]
pub struct ProposalVoteRecord {
    pub id: i64,
    pub proposal_id: i64,
    pub voter_public_key: String,
    pub voter_node_id: String,
    pub vote: String,
    pub created_time: SystemTime,
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "proposal_vote_record"]
pub struct NewProposalVoteRecord {
    pub proposal_id: i64,
    pub voter_public_key: String,
    pub voter_node_id: String,
    pub vote: String,
    pub created_time: SystemTime,
}

#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[table_name = "gameroom_member"]
#[belongs_to(Gameroom, foreign_key = "circuit_id")]
pub struct GameroomMember {
    pub id: i64,
    pub circuit_id: String,
    pub node_id: String,
    pub endpoints: Vec<String>,
    pub status: String,
    pub created_time: SystemTime,
    pub updated_time: SystemTime,
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "gameroom_member"]
pub struct NewGameroomMember {
    pub circuit_id: String,
    pub node_id: String,
    pub endpoints: Vec<String>,
    pub status: String,
    pub created_time: SystemTime,
    pub updated_time: SystemTime,
}

#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[table_name = "gameroom_service"]
#[belongs_to(Gameroom, foreign_key = "circuit_id")]
pub struct GameroomService {
    pub id: i64,
    pub circuit_id: String,
    pub service_id: String,
    pub service_type: String,
    pub allowed_nodes: Vec<String>,
    pub arguments: Vec<serde_json::Value>,
    pub status: String,
    pub last_event: String,
    pub created_time: SystemTime,
    pub updated_time: SystemTime,
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name = "gameroom_service"]
pub struct NewGameroomService {
    pub circuit_id: String,
    pub service_id: String,
    pub service_type: String,
    pub allowed_nodes: Vec<String>,
    pub arguments: Vec<serde_json::Value>,
    pub status: String,
    pub last_event: String,
    pub created_time: SystemTime,
    pub updated_time: SystemTime,
}

#[derive(Queryable, Identifiable, Associations)]
#[table_name = "gameroom_notification"]
pub struct GameroomNotification {
    pub id: i64,
    pub notification_type: String,
    pub requester: String,
    pub requester_node_id: String,
    pub target: String,
    pub created_time: SystemTime,
    pub read: bool,
}

#[derive(Debug, Insertable)]
#[table_name = "gameroom_notification"]
pub struct NewGameroomNotification {
    pub notification_type: String,
    pub requester: String,
    pub requester_node_id: String,
    pub target: String,
    pub created_time: SystemTime,
    pub read: bool,
}

// #[derive(Clone, Queryable, Identifiable, Associations, Insertable, AsChangeset)]
// #[table_name = "messages"]
// pub struct Message {
//     pub id: i32,
//     pub circuit_id: String,
//     pub message_name: String,
//     pub message_content: String,
//     pub message_type: String,
//     pub previous_id: Option<i32>,
//     pub sender: String,
//     pub participant_1: String,
//     pub participant_2: String,
//     pub created_time: SystemTime,
//     pub updated_time: SystemTime,
// }


#[derive(Clone, Queryable, Identifiable, Associations, Insertable, AsChangeset)]
#[table_name = "statuses"]
pub struct Status {
    id: i32,
    status_name: String,
    sender: String,
    participant1: String,
    participant2: String,
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

#[derive(Queryable, PartialEq, Debug)]
pub struct ActiveGameroom {
    pub circuit_id: String,
    pub service_id: String,
    pub status: String,
    pub last_event: String,
    pub requester: String,
    pub requester_node_id: String,
}

// for message type handling
// #[derive(Debug, Copy, Clone)]
// pub enum MessageType {
//     TEXT,
//     ERROR
// }

// impl ToString for MessageType {
//     fn to_string(&self) -> String {
//         return match self {
//             MessageType::TEXT => "TEXT".to_string(),
//             _ => "error".to_string()
//         }
//     }
// }

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
