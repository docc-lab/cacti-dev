/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

//! Candidate aggressor + pattern grouping

use std::collections::HashMap;
use std::error::Error;
// use std::iter::zip;
use std::time::Duration;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use genawaiter::{rc::gen, yield_};
use petgraph::visit::EdgeRef;
use petgraph::{dot::Dot, graph::NodeIndex, Direction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use pythia_common::RequestType;

use crate::trace::DAGEdge;
use crate::trace::EdgeType;
use crate::trace::Event;
use crate::trace::EventType;
use crate::trace::Trace;
use crate::trace::TracepointID;
use crate::PythiaError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TraceEdge {
    pub g: Trace,
    pub start: Option<Event>,
    pub end: Option<Event>,
    pub edge: Option<DAGEdge>,
}

impl TraceEdge {
    pub fn edges_from_trace(dag: &Trace) -> Result<Vec<TraceEdge>, Box<dyn Error>> {
        let mut all_edges: Vec<TraceEdge> = Vec::new();

        let mut cur_nodes: Vec<NodeIndex> = Vec::new();
        cur_nodes.push(dag.start_node);

        loop {
            if cur_nodes.is_empty() {
                break;
            }

            let cur_node: NodeIndex = match cur_nodes.pop() {
                Some(nidx) => nidx,
                None => continue
            };

            let next_nodes = dag.g
                .neighbors_directed(cur_node, Direction::Outgoing).collect::<Vec<_>>();

            for &nidx in next_nodes.iter() {
                let mut new_edge = TraceEdge {
                    g: Trace::new(&dag.base_id),
                    start: None,
                    end: None,
                    edge: None
                };

                new_edge.g.g.add_edge(
                    cur_node,
                    nidx,
                    dag.g[dag.g.find_edge(cur_node, nidx).unwrap()].clone(),
                );

                new_edge.start = match new_edge.g.g.node_weight(cur_node) {
                    Some(e) => {
                        let mut new_e = Event{
                            trace_id: e.trace_id,
                            tracepoint_id: e.tracepoint_id,
                            timestamp: e.timestamp,
                            is_synthetic: e.is_synthetic.into(),
                            variant: e.variant,
                            key_value_pair: HashMap::new()
                            // key_value_pair: HashMap::from(
                            //     zip(e.key_value_pair.into_keys().into(), e.key_value_pair.into_values()).into())
                        };

                        for key in e.key_value_pair.keys() {
                            new_e.key_value_pair.insert(
                                key.clone(),e.key_value_pair[&key.clone()].clone());
                        }

                        Some(new_e)
                    },
                    None => return Err(Box::new(PythiaError(
                        format!("Error extracting edges {}", new_edge.g).into(),
                    )))
                };

                new_edge.end = match new_edge.g.g.node_weight(nidx) {
                    Some(e) => {
                        let mut new_e = Event{
                            trace_id: e.trace_id,
                            tracepoint_id: e.tracepoint_id,
                            timestamp: e.timestamp,
                            is_synthetic: e.is_synthetic.into(),
                            variant: e.variant,
                            key_value_pair: HashMap::new()
                            // key_value_pair: HashMap::from(
                            //     zip(e.key_value_pair.into_keys().into(), e.key_value_pair.into_values()).into())
                        };

                        for key in e.key_value_pair.keys() {
                            new_e.key_value_pair.insert(
                                key.clone(),e.key_value_pair[&key.clone()].clone());
                        }

                        Some(new_e)
                    },
                    None => return Err(Box::new(PythiaError(
                        format!("Error extracting edges {}", new_edge.g).into(),
                    )))
                };

                let edge_index = match new_edge.g.g.find_edge(cur_node, nidx) {
                    Some(ei) => ei,
                    None => return Err(Box::new(PythiaError(
                        format!("Error extracting edges {}", new_edge.g).into(),
                    )))
                };
                new_edge.edge = match new_edge.g.g.edge_weight(edge_index) {
                    Some(de) => Some(de.clone()),
                    None => return Err(Box::new(PythiaError(
                        format!("Error extracting edges {}", new_edge.g).into(),
                    )))
                };

                all_edges.push(new_edge);

                cur_nodes.push(nidx)
            }
        }

        Ok(all_edges)
    }

    pub fn check_overlap(&self, e: &TraceEdge) -> bool {
        let mut self_start = match &self.start {
            Some(start) => start,
            None => return false
        };

        let mut self_end = match &self.end {
            Some(end) => end,
            None => return false
        };

        let mut e_start = match &e.start {
            Some(start) => start,
            None => return false
        };

        let mut e_end = match &e.end {
            Some(end) => end,
            None => return false
        };

        if e_start.timestamp.timestamp() < self_end.timestamp.timestamp() {
            if e_end.timestamp.timestamp() > self_start.timestamp.timestamp() {
                return true
            }
        }

        false
    }

    pub fn get_candidates(
        dag: &Trace,
        victim: &TraceEdge
    ) -> Result<Vec<TraceEdge>, Box<dyn Error>> {
        let mut candidates: Vec<TraceEdge> = Vec::new();

        let mut all_edges: Vec<TraceEdge> = match TraceEdge::edges_from_trace(dag) {
            Ok(es) => es,
            _ => return Err(Box::new(PythiaError(
                format!("Error extracting candidates {}", dag).into(),
            )))
        };

        while all_edges.len() > 0 {
            let cur_edge = all_edges.pop().unwrap();

            if victim.check_overlap(&cur_edge) {
                candidates.push(cur_edge)
            }
        }

        Ok(candidates)
    }
}