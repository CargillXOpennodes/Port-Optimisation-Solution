-- Copyright 2018-2020 Cargill Incorporated
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

-- Create tables
CREATE TABLE IF NOT EXISTS gameroom_user (
  email                     TEXT        PRIMARY KEY,
  public_key                TEXT        NOT NULL,
  encrypted_private_key     TEXT        NOT NULL,
  hashed_password           TEXT        NOT NULL
);

CREATE TABLE IF NOT EXISTS  gameroom (
  circuit_id                TEXT        PRIMARY KEY,
  authorization_type        TEXT        NOT NULL,
  persistence               TEXT        NOT NULL,
  durability                TEXT        NOT NULL,
  routes                    TEXT        NOT NULL,
  circuit_management_type   TEXT        NOT NULL,
  alias                     TEXT       NOT NULL,
  status                    TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  updated_time              TIMESTAMP   NOT NULL
);

CREATE TABLE IF NOT EXISTS gameroom_proposal (
  id                        BIGSERIAL   PRIMARY KEY,
  proposal_type             TEXT        NOT NULL ,
  circuit_id                TEXT        NOT NULL,
  circuit_hash              TEXT        NOT NULL,
  requester                 TEXT        NOT NULL,
  requester_node_id         TEXT        NOT NULL,
  status                    TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  updated_time              TIMESTAMP   NOT NULL,
  FOREIGN KEY (circuit_id) REFERENCES gameroom(circuit_id) ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS proposal_vote_record(
  id                        BIGSERIAL   PRIMARY KEY,
  proposal_id               BIGSERIAL   NOT NULL,
  voter_public_key          TEXT        NOT NULL,
  voter_node_id             TEXT        NOT NULL,
  vote                      TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  FOREIGN KEY (proposal_id) REFERENCES gameroom_proposal(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS gameroom_member (
  id                        BIGSERIAL   PRIMARY KEY,
  circuit_id               TEXT        NOT NULL,
  node_id                   TEXT        NOT NULL,
  endpoint                 TEXT[]      NOT NULL,
  status                    TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  updated_time              TIMESTAMP   NOT NULL,
  FOREIGN KEY (circuit_id) REFERENCES gameroom(circuit_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS gameroom_service (
  id                        BIGSERIAL   PRIMARY KEY,
  circuit_id                TEXT        NOT NULL,
  service_id                TEXT        NOT NULL,
  service_type              TEXT        NOT NULL,
  allowed_nodes             TEXT[][]    NOT NULL,
  arguments                 JSON []     NOT NULL,
  status                    TEXT        NOT NULL,
  last_event                TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  updated_time              TIMESTAMP   NOT NULL,
  FOREIGN KEY (circuit_id) REFERENCES gameroom(circuit_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS gameroom_notification (
  id                        BIGSERIAL   PRIMARY KEY,
  notification_type         TEXT        NOT NULL,
  requester                 TEXT        NOT NULL,
  requester_node_id         TEXT        NOT NULL,
  target                    TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  read                      BOOLEAN     NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
  id                        BIGSERIAL   PRIMARY KEY,
  circuit_id                TEXT        NOT NULL,
  message_name              TEXT        NOT NULL,
  message_content           TEXT        NOT NULL,
  message_type              TEXT        NOT NULL,
  previous_id               INTEGER,
  sender                    TEXT        NOT NULL,
  participant_1             TEXT        NOT NULL,
  participant_2             TEXT        NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  updated_time              TIMESTAMP   NOT NULL,
  FOREIGN KEY (circuit_id) REFERENCES gameroom(circuit_id) ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS statuses (
  id                        BIGSERIAL   PRIMARY KEY,
  circuit_id                TEXT        NOT NULL,
  proposal_id               BIGSERIAL   NOT NULL,
  status_name               TEXT        NOT NULL,
  sender                    TEXT        NOT NULL,
  participant1              TEXT        NOT NULL,
  participant2              TEXT        NOT NULL,
  participant1_short        TEXT        NOT NULL,
  participant2_short        TEXT        NOT NULL,
  eta                       TIMESTAMP   NULL,
  etb                       TIMESTAMP   NULL,
  ata                       TIMESTAMP   NULL,
  eto                       TIMESTAMP   NULL,
  ato                       TIMESTAMP   NULL,
  etc                       TIMESTAMP   NULL,
  etd                       TIMESTAMP   NULL,
  is_bunkering              BOOLEAN     NOT NULL,
  bunkering_time            TIMESTAMP   NOT NULL,
  created_time              TIMESTAMP   NOT NULL,
  updated_time              TIMESTAMP   NOT NULL,
  FOREIGN KEY (circuit_id) REFERENCES gameroom(circuit_id) ON DELETE CASCADE
  FOREIGN KEY (proposal_id) REFERENCES gameroom_proposal(id) ON DELETE CASCADE
);
