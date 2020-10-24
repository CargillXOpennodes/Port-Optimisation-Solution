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

use crate::models::{Message};
use crate::schema::messages;

use diesel::{
    dsl::insert_into, pg::PgConnection, prelude::*, QueryResult,
};

pub fn get_latest_message_id(circuit_id: &str, conn: &PgConnection) -> QueryResult<i64> {
    messages::table
        .filter(messages::circuit_id.eq(circuit_id))
        .count()
        .get_result(conn)
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

// pub fn fetch_message(
//     conn: &PgConnection,
//     circuit_id: &str,
//     name: &str,
// ) -> QueryResult<Option<Message>> {
//     message::table
//         .filter(
//             message::message_name
//                 .eq(name)
//                 .and(message::circuit_id.eq(circuit_id)),
//         )
//         .first::<Message>(conn)
//         .map(Some)
//         .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
// }

// pub fn insert_xo_game(conn: &PgConnection, game: NewXoGame) -> QueryResult<()> {
//     insert_into(xo_games::table)
//         .values(game)
//         .execute(conn)
//         .map(|_| ())
// }

pub fn add_message(conn: &PgConnection, sent_message: Message) -> QueryResult<()> {
    insert_into(messages::table)
        .values(sent_message.clone())
        .execute(conn)
        .map(|_| ())
    
    // diesel::update(
    //     xo_games::table.filter(
    //         xo_games::game_name
    //             .eq(&updated_game.game_name)
    //             .and(xo_games::circuit_id.eq(&updated_game.circuit_id)),
    //     ),
    // )
    // .set(updated_game.clone())
    // .execute(conn)
    // .map(|_| ())
}

pub fn get_message_count(conn: &PgConnection) -> QueryResult<i64> {
    messages::table.count().get_result(conn)
}
