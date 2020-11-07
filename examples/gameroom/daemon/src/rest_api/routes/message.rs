// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use actix_web::{web, Error, HttpResponse};
use gameroom_database::{helpers, models::Status, ConnectionPool};

use crate::rest_api::RestApiResponseError;

use super::{
    get_response_paging_info, validate_limit, ErrorResponse, SuccessResponse, DEFAULT_LIMIT,
    DEFAULT_OFFSET,
};


// pub struct ApiMessage {
//     id : i32,
//     circuit_id: String,
//     message_name: String,
//     message_content: String,
//     message_type: String,
//     sender: String,
//     previous_id: Option<i32>,
//     participant_1: String,
//     participant_2: String,
//     created_time: u64,
//     updated_time: u64,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiStatus {
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

// impl From<Message> for ApiMessage {
//     fn from(sta: Message) -> Self {
//         Self {
//             id: sta.id,
//             circuit_id: sta.circuit_id.to_string(),
//             message_name: sta.message_name.to_string(),
//             message_content: sta.message_content.to_string(),
//             message_type: sta.message_type.to_string(),
//             sender: sta.sender.to_string(),
//             previous_id: sta.previous_id,
//             participant_1: sta.participant_1.to_string(),
//             participant_2: sta.participant_2.to_string(),
//             created_time: sta
//                 .created_time
//                 .duration_since(SystemTime::UNIX_EPOCH)
//                 .unwrap_or_else(|_| Duration::new(0, 0))
//                 .as_secs(),
//             updated_time: sta
//                 .updated_time
//                 .duration_since(SystemTime::UNIX_EPOCH)
//                 .unwrap_or_else(|_| Duration::new(0, 0))
//                 .as_secs(),
//         }
//     }
// }

impl From<Status> for ApiStatus {
    fn from(sta: Status) -> Status {
        Status {
            name: name,
            sender: sta.sender,
            participant1: sta.participant1
            participant2: sta.partcipant2
            participant1_short: sta.participant1_short
            participant2_short: sta.participant2_short
            docking_type: sta.docking_type.DockingType(),
            eta: sta.eta,
            etb: sta.etb,
            ata: sta.ata,
            eto: sta.eto,
            ato: sta.ato,
            etc: sta.etc,
            etd: sta.etd
            is_bunkering: sta.is_bunkering
            bunkering_time: sta.bunkering_time
            logs: sta.logs

        }
    }
}

pub async fn list_statuses(
    statuses: HashMap<String, Status>
) -> String {
    let mut status_strings: Vec<String> = vec![];
        for (_, status) in statuses {
            status_strings.push(status.to_string().clone());
        }
        status_strings.sort();
        status_strings.join("|")

    match web::block(move || list_statuses_from_db(pool, &circuit_id.clone(), limit, offset)).await
    {
        Ok((games, query_count)) => {
            let paging_info =
                get_response_paging_info(limit, offset, &base_link, query_count as usize);
            Ok(HttpResponse::Ok().json(SuccessResponse::list(games, paging_info)))
        }
        Err(err) => {
            debug!("Internal Server Error: {}", err);
            Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
        }
    }
}



fn list_statuses_from_db(
    statuses: HashMap<String, Status>
    limit: usize,
    offset: usize,
) -> Result<(Vec<ApiStatus>, i64), RestApiResponseError> {
    let db_limit = validate_limit(limit);
    let db_offset = offset as i64;

    let statuses = helpers::list_statuses(&*statuses.get()?, circuit_id, db_limit, db_offset)?
        .into_iter()
        .map(ApiStatus::from)
        .collect();

    Ok((statuses))
}
