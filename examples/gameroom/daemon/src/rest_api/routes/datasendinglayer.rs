use std::collections::HashMap;
use std::thread::sleep;
use std::time::{Duration, Instant};

use actix_web::{client::Client, dev::Body, error, http::StatusCode, web, Error, HttpResponse};
use gameroom_database::{helpers, ConnectionPool};
use scabbard::{
    protocol::SCABBARD_PROTOCOL_VERSION,
    service::{BatchInfo, BatchStatus},
};
use splinter::protocol::ADMIN_PROTOCOL_VERSION;

use super::{ErrorResponse, SuccessResponse};

use crate::config::NodeInfo;
use crate::rest_api::RestApiResponseError;
use crate::rest_api::routes::{submit_scabbard_payload, submit_scabbard_payload_internal, list_gamerooms_from_db};
use actix_web::dev::Path;
use actix_web::web::Query;
use std::ops::Deref;

#[derive(Debug, Serialize, Deserialize)]
pub struct DSL {
    pub payload1: String,
    pub payload2: String,
}

impl DSL {
    pub fn get(&self,index: i32) -> web::Bytes {
        if index == 0 {
            web::Bytes::from(self.payload1.clone())
        } else {
            web::Bytes::from(self.payload2.clone())
        }
    }
}


pub async fn data_sharing_layer(
    client: web::Data<Client>,
    splinterd_url: web::Data<String>,
    pool: web::Data<ConnectionPool>,
    node_info: web::Data<NodeInfo>,
    body: web::Json<DSL>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    debug!("{:?}", body);

    let mut count = 0;
    let gamerooms = list_gamerooms_from_db(pool.clone(), Some("Active".to_string()), 2, 0)
        .unwrap_or((Vec::new(), 0));

    for gameroom in gamerooms.0 {
        submit_scabbard_payload_internal(client.clone(), splinterd_url.clone(),
                                         pool.clone(), gameroom.circuit_id.clone(),
                                         node_info.clone(), body.get(count),
                                         query.clone())
            .await;
        count += 1;
        debug!("{}, {}, {:?}", gameroom.circuit_id, count, body.get(count));
    }

    Ok(HttpResponse::new(StatusCode::ACCEPTED))
}