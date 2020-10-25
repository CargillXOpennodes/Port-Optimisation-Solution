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

use num_traits::cast::ToPrimitive;
use crate::models::{Message};
use crate::schema::messages;

use diesel::{
    dsl::insert_into, pg::PgConnection, prelude::*, result::Error::NotFound, QueryResult,
};

pub fn get_latest_message_id(circuit_id: &str, conn: &PgConnection) -> QueryResult<i64> {
    messages::table
        .filter(messages::circuit_id.eq(circuit_id))
        .count()
        .get_result(conn)
}

pub fn get_latest_message(circuit_id: &str, conn: &PgConnection) -> QueryResult<Option<Message>> {
     get_latest_message_id(circuit_id, conn)
        .and_then(|id|
            messages::table
                .filter(
                    messages::id
                        .eq(id.to_i32().unwrap() - 1)
                        .and(messages::circuit_id.eq(circuit_id)),
                )
                .first::<Message>(conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
        )
        .or_else(|err| Err(err))
}

pub fn list_messages(
    conn: &PgConnection,
    circuit_id: &str,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Message>> {
    messages::table
        .filter(messages::circuit_id.eq(circuit_id))
        .order_by(messages::id.desc())
        .limit(limit)
        .offset(offset)
        .load::<Message>(conn)
}

pub fn add_message(conn: &PgConnection, sent_message: Message) -> QueryResult<()> {
    insert_into(messages::table)
        .values(sent_message.clone())
        .execute(conn)
        .map(|_| ())
}

pub fn get_message_count(conn: &PgConnection) -> QueryResult<i64> {
    messages::table.count().get_result(conn)
}
