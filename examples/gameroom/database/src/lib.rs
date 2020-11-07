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

#[macro_use]
extern crate diesel;

pub mod error;
pub mod helpers;
pub mod models;
pub mod schema;

use std::ops::Deref;

use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

pub use crate::error::DatabaseError;

pub fn create_connection_pool(database_url: &str) -> Result<ConnectionPool, DatabaseError> {
    let connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    Ok(ConnectionPool {
        pool: Pool::builder()
            .build(connection_manager)
            .map_err(|err| DatabaseError::ConnectionError(Box::new(err)))?,
    })
}

pub struct Connection(PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for Connection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct ConnectionPool {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl ConnectionPool {
    pub fn get(&self) -> Result<Connection, DatabaseError> {
        self.pool
            .get()
            .map(Connection)
            .map_err(|err| DatabaseError::ConnectionError(Box::new(err)))
    }
}


impl FromSql<BigInt, Sqlite> for std::Duration {
    fn from_sql(value: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        let i64_value = <i64 as FromSql<BigInt, Sqlite>>::from_sql(value)?;
        Ok(std::Duration::nanoseconds(i64_value))
    }
}

impl ToSql<BigInt, Sqlite> for std::Duration {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        if let Some(num_nanoseconds) = self.num_nanoseconds() {
            ToSql::<BigInt, Sqlite>::to_sql(&num_nanoseconds, out)
        } else {
            Err(format!("{:?} as nanoseconds is too larg to fit in an i64", self).into())
        }
    }
}