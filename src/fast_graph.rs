/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use serde::Deserialize;
use serde::Serialize;

use crate::constants::Weight;
use crate::constants::{EdgeId, NodeId, INVALID_EDGE};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct FastGraph {
    num_nodes: usize,
    pub(crate) ranks: Vec<usize>,
    pub(crate) edges_fwd: Vec<FastGraphEdge>,
    pub(crate) first_edge_ids_fwd: Vec<EdgeId>,

    pub(crate) edges_bwd: Vec<FastGraphEdge>,
    pub(crate) first_edge_ids_bwd: Vec<EdgeId>,
}

impl FastGraph {
    pub fn new(num_nodes: usize) -> Self {
        FastGraph {
            ranks: vec![0; num_nodes],
            num_nodes,
            edges_fwd: vec![],
            first_edge_ids_fwd: vec![0; num_nodes + 1],
            edges_bwd: vec![],
            first_edge_ids_bwd: vec![0; num_nodes + 1],
        }
    }
}

pub trait FastGraphLike {
    fn get_node_ordering(&self) -> Vec<NodeId>;
    fn get_num_nodes(&self) -> usize;
    fn get_num_out_edges(&self) -> usize;
    fn get_num_in_edges(&self) -> usize;
    fn get_out_edge(&self, id: EdgeId) -> &impl FastGraphEdgeLike;
    fn get_in_edge(&self, id: EdgeId) -> &impl FastGraphEdgeLike;
    fn begin_in_edges(&self, node: NodeId) -> usize;
    fn end_in_edges(&self, node: NodeId) -> usize;
    fn begin_out_edges(&self, node: NodeId) -> usize;
    fn end_out_edges(&self, node: NodeId) -> usize;
}

impl FastGraphLike for FastGraph {
    fn get_node_ordering(&self) -> Vec<NodeId> {
        let mut ordering = vec![0; self.ranks.len()];
        for i in 0..self.ranks.len() {
            ordering[self.ranks[i]] = i;
        }
        ordering
    }

    fn get_num_nodes(&self) -> usize {
        self.num_nodes
    }

    fn get_num_out_edges(&self) -> usize {
        self.edges_fwd.len()
    }

    fn get_num_in_edges(&self) -> usize {
        self.edges_bwd.len()
    }

    fn get_out_edge(&self, id: EdgeId) -> &impl FastGraphEdgeLike {
        &self.edges_fwd[id]
    }

    fn get_in_edge(&self, id: EdgeId) -> &impl FastGraphEdgeLike {
        &self.edges_bwd[id]
    }

    fn begin_in_edges(&self, node: NodeId) -> usize {
        self.first_edge_ids_bwd[self.ranks[node]]
    }

    fn end_in_edges(&self, node: NodeId) -> usize {
        self.first_edge_ids_bwd[self.ranks[node] + 1]
    }

    fn begin_out_edges(&self, node: NodeId) -> usize {
        self.first_edge_ids_fwd[self.ranks[node]]
    }

    fn end_out_edges(&self, node: NodeId) -> usize {
        self.first_edge_ids_fwd[self.ranks[node] + 1]
    }
}

#[cfg(feature = "rkyv")]
impl ArchivedFastGraph {
    unsafe fn from_bytes(bytes: &[u8]) -> &Self {
        rkyv::access_unchecked::<ArchivedFastGraph>(bytes)
    }
}

#[cfg(feature = "rkyv")]
impl FastGraphLike for ArchivedFastGraph {
    fn get_node_ordering(&self) -> Vec<NodeId> {
        let mut ordering = vec![0; self.ranks.len()];
        for i in 0..self.ranks.len() {
            ordering[self.ranks[i].to_native() as usize] = i;
        }
        ordering
    }

    fn get_num_nodes(&self) -> usize {
        self.num_nodes.to_native() as usize
    }

    fn get_num_out_edges(&self) -> usize {
        self.edges_fwd.len()
    }

    fn get_num_in_edges(&self) -> usize {
        self.edges_bwd.len()
    }

    fn get_out_edge(&self, id: EdgeId) -> &impl FastGraphEdgeLike {
        &self.edges_fwd[id]
    }

    fn get_in_edge(&self, id: EdgeId) -> &impl FastGraphEdgeLike {
        &self.edges_bwd[id]
    }

    fn begin_in_edges(&self, node: NodeId) -> usize {
        self.first_edge_ids_bwd[self.ranks[node].to_native() as usize].to_native() as usize
    }

    fn end_in_edges(&self, node: NodeId) -> usize {
        self.first_edge_ids_bwd[self.ranks[node].to_native() as usize + 1].to_native() as usize
    }

    fn begin_out_edges(&self, node: NodeId) -> usize {
        self.first_edge_ids_fwd[self.ranks[node].to_native() as usize].to_native() as usize
    }

    fn end_out_edges(&self, node: NodeId) -> usize {
        self.first_edge_ids_fwd[self.ranks[node].to_native() as usize + 1].to_native() as usize
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct FastGraphEdge {
    // todo: the base_node is 'redundant' for the routing query so to say, but makes the implementation easier for now
    // and can still be removed at a later time, we definitely need this information on original
    // edges for shortcut unpacking. a possible hack is storing it in the (for non-shortcuts)
    // unused replaced_in/out_edge field.
    pub base_node: NodeId,
    pub adj_node: NodeId,
    pub weight: Weight,
    pub replaced_in_edge: EdgeId,
    pub replaced_out_edge: EdgeId,
}

impl FastGraphEdge {
    pub fn new(
        base_node: NodeId,
        adj_node: NodeId,
        weight: Weight,
        replaced_edge1: EdgeId,
        replaced_edge2: EdgeId,
    ) -> Self {
        FastGraphEdge {
            base_node,
            adj_node,
            weight,
            replaced_in_edge: replaced_edge1,
            replaced_out_edge: replaced_edge2,
        }
    }
}

pub trait FastGraphEdgeLike {
    fn is_shortcut(&self) -> bool;
    fn base_node(&self) -> NodeId;
    fn adj_node(&self) -> NodeId;
    fn weight(&self) -> Weight;
    fn replaced_in_edge(&self) -> EdgeId;
    fn replaced_out_edge(&self) -> EdgeId;
}

impl FastGraphEdgeLike for FastGraphEdge {
    fn is_shortcut(&self) -> bool {
        assert!(
            (self.replaced_in_edge == INVALID_EDGE && self.replaced_out_edge == INVALID_EDGE)
                || (self.replaced_in_edge != INVALID_EDGE
                && self.replaced_out_edge != INVALID_EDGE)
        );
        self.replaced_in_edge != INVALID_EDGE
    }

    fn base_node(&self) -> NodeId {
        self.base_node
    }

    fn adj_node(&self) -> NodeId {
        self.adj_node
    }

    fn weight(&self) -> Weight {
        self.weight
    }

    fn replaced_in_edge(&self) -> EdgeId {
        self.replaced_in_edge
    }

    fn replaced_out_edge(&self) -> EdgeId {
        self.replaced_out_edge
    }
}

#[cfg(feature = "rkyv")]
impl FastGraphEdgeLike for ArchivedFastGraphEdge {
    fn is_shortcut(&self) -> bool {
        let replaced_in_edge = self.replaced_in_edge.to_native() as usize;
        let replaced_out_edge = self.replaced_out_edge.to_native() as usize;
        assert!(
            (replaced_in_edge == INVALID_EDGE && replaced_out_edge == INVALID_EDGE)
                || (replaced_in_edge != INVALID_EDGE
                && replaced_out_edge != INVALID_EDGE)
        );
        replaced_in_edge != INVALID_EDGE
    }

    fn base_node(&self) -> NodeId {
        self.base_node.to_native() as usize
    }

    fn adj_node(&self) -> NodeId {
        self.adj_node.to_native() as usize
    }

    fn weight(&self) -> Weight {
        self.weight.to_native() as usize
    }

    fn replaced_in_edge(&self) -> EdgeId {
        self.replaced_in_edge.to_native() as usize
    }

    fn replaced_out_edge(&self) -> EdgeId {
        self.replaced_out_edge.to_native() as usize
    }
}
