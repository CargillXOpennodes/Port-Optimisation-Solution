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

use actix_web::{error, web, Error, HttpResponse};
use gameroom_database::{helpers, models::XoGame, ConnectionPool};

use crate::rest_api::RestApiResponseError;

use super::{
    get_response_paging_info, validate_limit, ErrorResponse, SuccessResponse, DEFAULT_LIMIT,
    DEFAULT_OFFSET,
};
use gameroom_database::models::Message;
use gameroom_database::schema::messages::dsl::messages;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiMessage {
    id : i32,
    circuit_id: String,
    message_name: String,
    message_content: String,
    message_type: String,
    sender: String,
    previous_id: Option<i32>,
    participant_1: String,
    participant_2: String,
    created_time: u64,
    updated_time: u64,
}

impl From<Message> for ApiMessage {
    fn from(msg: Message) -> Self {
        Self {
            id: msg.id,
            circuit_id: msg.circuit_id.to_string(),
            message_name: msg.message_name.to_string(),
            message_content: msg.message_content.to_string(),
            message_type: msg.message_type.to_string(),
            sender: msg.sender.to_string(),
            previous_id: msg.previous_id,
            participant_1: msg.participant_1.to_string(),
            participant_2: msg.participant_2.to_string(),
            created_time: msg
                .created_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::new(0, 0))
                .as_secs(),
            updated_time: msg
                .updated_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::new(0, 0))
                .as_secs(),
        }
    }
}

pub async fn list_messages(
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
    let base_link = format!("api/xo/{}/games?", &circuit_id);

    match web::block(move || list_messages_from_db(pool, &circuit_id.clone(), limit, offset)).await
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

fn list_messages_from_db(
    pool: web::Data<ConnectionPool>,
    circuit_id: &str,
    limit: usize,
    offset: usize,
) -> Result<(Vec<ApiMessage>, i64), RestApiResponseError> {
    let db_limit = validate_limit(limit);
    let db_offset = offset as i64;

    let messages = helpers::list_messages(&*pool.get()?, circuit_id, db_limit, db_offset)?
        .into_iter()
        .map(ApiMessage::from)
        .collect();
    let message_count = helpers::get_message_count(&*pool.get()?)?;

    Ok((messages, message_count))
}
