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

//! Structs for building proposed nodes

use super::error::BuilderError;

/// Native representation of a node in a proposed circuit
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ProposedNode {
    node_id: String,
    endpoints: Vec<String>,
}

impl ProposedNode {
    /// Returns the ID of the proposed node
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Returns the list of endpoints that belong to the proposed node
    pub fn endpoints(&self) -> &[String] {
        &self.endpoints
    }
}

/// Builder for creating a `ProposedNode`
#[derive(Default, Clone)]
pub struct ProposedNodeBuilder {
    node_id: Option<String>,
    endpoints: Option<Vec<String>>,
}

impl ProposedNodeBuilder {
    /// Creates a `ProposedNodeBuider`
    pub fn new() -> Self {
        ProposedNodeBuilder::default()
    }

    /// Returns the unique node ID
    pub fn node_id(&self) -> Option<String> {
        self.node_id.clone()
    }

    /// Returns the list of endpoints for the node
    pub fn endpoints(&self) -> Option<Vec<String>> {
        self.endpoints.clone()
    }

    /// Sets the node ID
    ///
    /// # Arguments
    ///
    ///  * `node_id` - The unique node ID for node
    pub fn with_node_id(mut self, node_id: &str) -> ProposedNodeBuilder {
        self.node_id = Some(node_id.into());
        self
    }

    /// Sets the endpoints
    ///
    /// # Arguments
    ///
    ///  * `endpoints` - The list of endpoints for the node
    pub fn with_endpoints(mut self, endpoints: &[String]) -> ProposedNodeBuilder {
        self.endpoints = Some(endpoints.into());
        self
    }

    /// Builds the `ProposedNode`
    ///
    /// Returns an error if the node ID or endpoints are not set
    pub fn build(self) -> Result<ProposedNode, BuilderError> {
        let node_id = self
            .node_id
            .ok_or_else(|| BuilderError::MissingField("node_id".to_string()))?;

        let endpoints = self
            .endpoints
            .ok_or_else(|| BuilderError::MissingField("endpoints".to_string()))?;

        let node = ProposedNode { node_id, endpoints };

        Ok(node)
    }
}
