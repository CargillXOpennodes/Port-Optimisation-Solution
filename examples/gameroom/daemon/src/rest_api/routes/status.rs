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
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_web::{web, Error, error, HttpResponse};
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
    id: i64,
    circuit_id: String,
    name: String,
    sender: String,
    participant1: String,
    participant2: String,
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
    created_time: SystemTime,
    updated_time: SystemTime
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
    fn from(sta: Status) -> ApiStatus {
        ApiStatus {
            id: sta.id,
            circuit_id: sta.circuit_id,
            name: sta.status_name,
            sender: sta.sender,
            participant1: sta.participant_1,
            participant2: sta.participant_2,
            eta: sta.eta
                .map(|system_time| system_time
                    .duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(0))),
            etb: sta.etb
                .map(|system_time| system_time
                    .duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(0))),
            ata: sta.ata
                .map(|system_time| system_time
                    .duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(0))),
            eto: sta.eto
                .map(|system_time| system_time
                    .duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(0))),
            ato: sta.ato
                .map(|system_time| system_time
                    .duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(0))),
            etc: sta.etc
                .map(|system_time| system_time
                    .duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(0))),
            etd: sta.etd
                .map(|system_time| system_time
                    .duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(0))),
            is_bunkering: sta.is_bunkering,
            bunkering_time: sta.bunkering_time
                .map(|system_time| system_time
                    .duration_since(UNIX_EPOCH).unwrap_or(Duration::from_nanos(0))),
            logs: sta.logs.to_string().clone(),
            created_time: sta.created_time,
            updated_time: sta.updated_time
        }
    }
}

pub async fn fetch_status(
    pool: web::Data<ConnectionPool>,
    circuit_id: web::Path<String>,
    game_name: web::Path<String>,
) -> Result<HttpResponse, Error> {
    match web::block(move || fetch_status_from_db(pool, &circuit_id, &game_name)).await {
        Ok(status) => Ok(HttpResponse::Ok().json(SuccessResponse::new(status))),
        Err(err) => {
            match err {
                error::BlockingError::Error(err) => match err {
                    RestApiResponseError::NotFound(err) => {
                        Ok(HttpResponse::NotFound().json(ErrorResponse::not_found(&err)))
                    }
                    _ => Ok(HttpResponse::BadRequest()
                        .json(ErrorResponse::bad_request(&err.to_string()))),
                },
                error::BlockingError::Canceled => {
                    debug!("Internal Server Error: {}", err);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
                }
            }
        }
    }
}

fn fetch_status_from_db(
    pool: web::Data<ConnectionPool>,
    circuit_id: &str,
    game_name: &str,
) -> Result<ApiStatus, RestApiResponseError> {
    if let Some(status) = helpers::fetch_status(&*pool.get()?, circuit_id, game_name)? {
        return Ok(ApiStatus::from(status));
    }
    Err(RestApiResponseError::NotFound(format!(
        "Status with name {} not found",
        game_name
    )))
}

pub async fn list_statuses(
    pool: web::Data<ConnectionPool>,
    circuit_id: web::Path<String>,
    query: web::Query<HashMap<String, usize>>,
) -> Result<HttpResponse, Error> {
    let offset: usize = query
        .get("offset")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_OFFSET);

    let limit: usize = query
        .get("limit")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_LIMIT);
    let base_link = format!("api/status/{}/statuses?", &circuit_id);

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
    pool: web::Data<ConnectionPool>,
    circuit_id: &str,
    limit: usize,
    offset: usize,
) -> Result<(Vec<ApiStatus>, i64), RestApiResponseError> {
    let db_limit = validate_limit(limit);
    let db_offset = offset as i64;

    let status = helpers::list_statuses(&*pool.get()?, circuit_id, db_limit, db_offset)?
        .into_iter()
        .map(ApiStatus::from)
        .collect();
    let status_count = helpers::get_status_count(&*pool.get()?)?;

    Ok((status, status_count))
}

