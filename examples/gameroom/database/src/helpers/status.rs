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

use crate::models::Status;
use crate::schema::statuses;

use diesel::{
    dsl::insert_into, pg::PgConnection, prelude::*, result::Error::NotFound, QueryResult,
};

pub fn list_statuses(
    conn: &PgConnection,
    circuit_id: &str,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<Status>> {
    statuses::table
        .filter(statuses::circuit_id.eq(circuit_id))
        .limit(limit)
        .offset(offset)
        .load::<Status>(conn)
}

pub fn fetch_status(
    conn: &PgConnection,
    circuit_id: &str,
    name: &str,
) -> QueryResult<Option<Status>> {
    statuses::table
        .filter(
            statuses::status_name
                .eq(name)
                .and(statuses::circuit_id.eq(circuit_id)),
        )
        .first::<Status>(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn insert_status(conn: &PgConnection, status: Status) -> QueryResult<()> {
    insert_into(statuses::table)
        .values(status)
        .execute(conn)
        .map(|_| ())
}

pub fn update_status(conn: &PgConnection, updated_status: Status) -> QueryResult<()> {
    diesel::update(
        statuses::table.filter(
            statuses::status_name
                .eq(&updated_status.status_name)
                .and(statuses::circuit_id.eq(&updated_status.circuit_id)),
        ),
    )
        .set(updated_status.clone())
        .execute(conn)
        .map(|_| ())
}

pub fn get_status_count(conn: &PgConnection) -> QueryResult<i64> {
    statuses::table.count().get_result(conn)
}
