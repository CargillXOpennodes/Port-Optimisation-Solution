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
use crate::rest_api::routes::submit_scabbard_payload;

#[derive(Debug, Serialize, Deserialize)]
pub struct  DSL{
    pub payload1: web::Bytes,
    pub payload2: web::Bytes,
}

impl DSL {
    pub fn get(&self,index: i32) -> web::Bytes
    {
        if index == 0 {
            self.payload1.clone()
        } else {
            self.payload2.clone()
        }
    }
}


pub async fn data_sharing_layer(
    client: web::Data<Client>,
    splinterd_url: web::Data<String>,
    pool: web::Data<ConnectionPool>,
    circuit_id: web::Path<String>,
    node_info: web::Data<NodeInfo>,
    signed_payload: web::Bytes,
    body: DSL,
    query: web::Query<HashMap<String, String>>,

) -> Result<HttpResponse, Error> {
    client
        .post(format!("{}/admin/submit", *splinterd_url))
        .header(
            "SplinterProtocolVersion",
            ADMIN_PROTOCOL_VERSION.to_string(),
        )
        .send_body(Body::Bytes(signed_payload))
        .await
        .map_err(Error::from)?;

    let mut count = 0;
    helpers::list_gamerooms_with_paging(&*pool.get()?, 2, 0)
        .or_else(Vec::new())
        .into_iter()
        .for_each(|gameroom| {
            submit_scabbard_payload(client, splinterd_url, pool, circuit_id, node_info, body.get(count), query).await;
            count += 1;
        });
    Ok(HttpResponse::new(StatusCode::ACCEPTED))
}