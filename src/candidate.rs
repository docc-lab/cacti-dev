/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

//! Candidate aggressor + pattern grouping

use std::collections::HashMap;
use std::error::Error;
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
pub struct CandidateAggressor {
    pub g: Trace,
    pub start: Event,
    pub end: Event,
    pub edge: DAGEdge,
}

impl CandidateAggressor {
    pub fn from_trace(dag: &Trace) -> Result<Vec<CandidateAggressor>, Box<dyn Error>> {
        let mut candidates: Vec<CandidateAggressor> = Vec::new();

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
                let mut candidate = CandidateAggressor {
                    g: Trace::new(&dag.base_id),
                    start: Event,
                    end: Event,
                    edge: DAGEdge
                };

                candidate.g.g.add_edge(
                    cur_node,
                    nidx,
                    dag.g[dag.g.find_edge(cur_node, nidx).unwrap()].clone(),
                );

                // candidate.start = candidate.g.g.

                candidates.push(candidate);

                cur_nodes.push(nidx)
            }
        }

        Ok(candidates)
    }
}