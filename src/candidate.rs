/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

//! Candidate aggressor + pattern grouping

use std::collections::{HashMap, HashSet};
use std::error::Error;
// use std::iter::zip;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use genawaiter::{rc::gen, yield_};
use petgraph::visit::EdgeRef;
use petgraph::{dot::Dot, graph::NodeIndex, Direction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use pythia_common::RequestType;

use crate::trace::{DAGEdge, TraceEdge};
use crate::trace::EdgeType;
use crate::trace::Event;
use crate::trace::EventType;
use crate::trace::Trace;
use crate::trace::TracepointID;
use crate::{CriticalPath, PythiaError, Settings};

pub struct CandidateManager {
    pub victim_type: RequestType,
    pub victim_start: String,
    pub victim_end: String,
    pub victim_paths: Vec<CriticalPath>,
    pub non_victim_traces: Vec<Trace>,
    // pub victim_segments: Vec<(Uuid, TraceEdge)>,
    // pub non_victim_segments: HashMap<String, (Uuid, TraceEdge, Option<i64>)>,
    pub non_victim_segments: HashMap<String, (Uuid, TraceEdge, i64)>,
    pub candidate_segments: Vec<TraceEdge>,
    pub victim_overlaps: HashMap<Uuid, (TraceEdge, Vec<TraceEdge>)>,
    pub victim_overlap_max_times: HashMap<Uuid, i64>,
    used_pairs: HashSet<(String, String)>,
}

impl CandidateManager {
    pub fn from_settings(settings: &Settings, victim_edge: (&str, &str)) -> CandidateManager {
        return CandidateManager {
            victim_type: settings.problem_type,
            victim_start: victim_edge.0.to_string(),
            victim_end: victim_edge.1.to_string(),
            victim_paths: Vec::new(),
            non_victim_traces: Vec::new(),
            // victim_segments: Vec::new(),
            non_victim_segments: HashMap::new(),
            candidate_segments: Vec::new(),
            victim_overlaps: HashMap::new(),
            victim_overlap_max_times: HashMap::new(),
            used_pairs: HashSet::new(),
        }
    }

    pub fn add_traces(&mut self, traces: Vec<Trace>) {
        for trace in traces {
            if trace.request_type == self.victim_type {
                self.victim_paths.push(CriticalPath::from_trace(&trace).unwrap())
            }
            // else {
            self.non_victim_traces.push(trace);
            // }
        }
    }

    pub fn process_victims(&mut self) {
        let now_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards!")
            .as_nanos() as i64;

        loop {
            if self.victim_paths.len() == 0 {
                break;
            }

            let cur_path = self.victim_paths.pop().unwrap();

            let cp_edges = cur_path.g.get_edges();

            for edge in cp_edges {
                if (edge.tp_start == self.victim_start) && (edge.tp_end == self.victim_end) {
                    // self.victim_overlaps.push((cur_path.g.base_id, edge))
                    self.victim_overlaps.insert(cur_path.g.base_id, (edge, Vec::new()));
                    self.victim_overlap_max_times.insert(cur_path.g.base_id, now_time.clone());
                }
            }
        }
    }

    pub fn process_non_victims(&mut self) {
        let now_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards!")
            .as_nanos() as i64;

        loop {
            if self.non_victim_traces.len() == 0 {
                break;
            }

            let cur_trace = self.non_victim_traces.pop().unwrap();

            let ct_edges = cur_trace.get_edges();

            for edge in ct_edges {
                let mut hasher = Sha256::new();
                hasher.input(cur_trace.base_id.as_bytes());
                hasher.input(edge.tp_start.as_bytes());
                hasher.input(edge.tp_end.as_bytes());
                self.non_victim_segments.insert(
                    hasher.result_str(),
                    (cur_trace.base_id, edge, now_time.clone())
                );
            }
        }
    }

    pub fn flush_old_victims(&mut self) {
        for overlap_info in &self.victim_overlaps {
            let latest_overlap_time = &self.victim_overlap_max_times.get(&overlap_info.0).unwrap();
            let now_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_nanos() as i64;
            if *latest_overlap_time + 300*1000000000 < now_time {
                self.victim_overlaps.remove(&overlap_info.0);
                self.victim_overlap_max_times.remove(&overlap_info.0);
            }
        }
    }

    pub fn flush_old_non_victims(&mut self) {
        let now_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards!")
            .as_nanos() as i64;

        for non_victim_info in self.non_victim_segments {
            // match non_victim_info.1.2 {
            //     Some(t) => {
            //         if t + 300*1000000000 < now_time {
            //             self.non_victim_segments.remove(&non_victim_info.0)
            //         }
            //     },
            //     None => (),
            // }
            if (non_victim_info.1).2 + 300*1000000000 < now_time {
                self.non_victim_segments.remove(&non_victim_info.0);
            }
        }
    }

    pub fn find_candidates(&mut self) {
        let now_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards!")
            .as_nanos() as i64;

        for victim in self.victim_overlaps {
            for non_victim in self.non_victim_segments {
                if victim.0 != (non_victim.1).0 {
                    if (victim.1).0.overlaps_with(&(non_victim.1).1) {
                        let mut new_overlaps = (victim.1).1.clone();
                        new_overlaps.push((non_victim.1).1);
                        self.victim_overlaps.insert(
                            victim.0,
                            ((victim.1).0.clone(), new_overlaps)
                        );
                        self.victim_overlap_max_times.insert(
                            victim.0,
                            now_time.clone()
                        );
                        self.non_victim_segments.insert(
                            non_victim.0,
                            ((non_victim.1).0, (non_victim.1).1.clone(), now_time.clone())
                        );
                    }
                }
            }
        }
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct TraceEdge {
//     pub g: Trace,
//     pub start: Option<Event>,
//     pub end: Option<Event>,
//     pub edge: Option<DAGEdge>,
// }
//
// impl TraceEdge {
//     pub fn edges_from_trace(dag: &Trace) -> Result<Vec<TraceEdge>, Box<dyn Error>> {
//         let mut all_edges: Vec<TraceEdge> = Vec::new();
//
//         let mut cur_nodes: Vec<NodeIndex> = Vec::new();
//         cur_nodes.push(dag.start_node);
//
//         loop {
//             if cur_nodes.is_empty() {
//                 break;
//             }
//
//             let cur_node: NodeIndex = match cur_nodes.pop() {
//                 Some(nidx) => nidx,
//                 None => continue
//             };
//
//             let next_nodes = dag.g
//                 .neighbors_directed(cur_node, Direction::Outgoing).collect::<Vec<_>>();
//
//             for &nidx in next_nodes.iter() {
//                 let mut new_edge = TraceEdge {
//                     g: Trace::new(&dag.base_id),
//                     start: None,
//                     end: None,
//                     edge: None
//                 };
//
//                 new_edge.g.g.add_edge(
//                     cur_node,
//                     nidx,
//                     dag.g[dag.g.find_edge(cur_node, nidx).unwrap()].clone(),
//                 );
//
//                 new_edge.start = match new_edge.g.g.node_weight(cur_node) {
//                     Some(e) => {
//                         let mut new_e = Event{
//                             trace_id: e.trace_id,
//                             tracepoint_id: e.tracepoint_id,
//                             timestamp: e.timestamp,
//                             is_synthetic: e.is_synthetic.into(),
//                             variant: e.variant,
//                             key_value_pair: HashMap::new()
//                             // key_value_pair: HashMap::from(
//                             //     zip(e.key_value_pair.into_keys().into(), e.key_value_pair.into_values()).into())
//                         };
//
//                         for key in e.key_value_pair.keys() {
//                             new_e.key_value_pair.insert(
//                                 key.clone(),e.key_value_pair[&key.clone()].clone());
//                         }
//
//                         Some(new_e)
//                     },
//                     None => return Err(Box::new(PythiaError(
//                         format!("Error extracting edges {}", new_edge.g).into(),
//                     )))
//                 };
//
//                 new_edge.end = match new_edge.g.g.node_weight(nidx) {
//                     Some(e) => {
//                         let mut new_e = Event{
//                             trace_id: e.trace_id,
//                             tracepoint_id: e.tracepoint_id,
//                             timestamp: e.timestamp,
//                             is_synthetic: e.is_synthetic.into(),
//                             variant: e.variant,
//                             key_value_pair: HashMap::new()
//                             // key_value_pair: HashMap::from(
//                             //     zip(e.key_value_pair.into_keys().into(), e.key_value_pair.into_values()).into())
//                         };
//
//                         for key in e.key_value_pair.keys() {
//                             new_e.key_value_pair.insert(
//                                 key.clone(),e.key_value_pair[&key.clone()].clone());
//                         }
//
//                         Some(new_e)
//                     },
//                     None => return Err(Box::new(PythiaError(
//                         format!("Error extracting edges {}", new_edge.g).into(),
//                     )))
//                 };
//
//                 let edge_index = match new_edge.g.g.find_edge(cur_node, nidx) {
//                     Some(ei) => ei,
//                     None => return Err(Box::new(PythiaError(
//                         format!("Error extracting edges {}", new_edge.g).into(),
//                     )))
//                 };
//                 new_edge.edge = match new_edge.g.g.edge_weight(edge_index) {
//                     Some(de) => Some(de.clone()),
//                     None => return Err(Box::new(PythiaError(
//                         format!("Error extracting edges {}", new_edge.g).into(),
//                     )))
//                 };
//
//                 all_edges.push(new_edge);
//
//                 cur_nodes.push(nidx)
//             }
//         }
//
//         Ok(all_edges)
//     }
//
//     pub fn check_overlap(&self, e: &TraceEdge) -> bool {
//         let mut self_start = match &self.start {
//             Some(start) => start,
//             None => return false
//         };
//
//         let mut self_end = match &self.end {
//             Some(end) => end,
//             None => return false
//         };
//
//         let mut e_start = match &e.start {
//             Some(start) => start,
//             None => return false
//         };
//
//         let mut e_end = match &e.end {
//             Some(end) => end,
//             None => return false
//         };
//
//         if e_start.timestamp.timestamp() < self_end.timestamp.timestamp() {
//             if e_end.timestamp.timestamp() > self_start.timestamp.timestamp() {
//                 return true
//             }
//         }
//
//         false
//     }
//
//     pub fn get_candidates(
//         dag: &Trace,
//         victim: &TraceEdge
//     ) -> Result<Vec<TraceEdge>, Box<dyn Error>> {
//         let mut candidates: Vec<TraceEdge> = Vec::new();
//
//         let mut all_edges: Vec<TraceEdge> = match TraceEdge::edges_from_trace(dag) {
//             Ok(es) => es,
//             _ => return Err(Box::new(PythiaError(
//                 format!("Error extracting candidates {}", dag).into(),
//             )))
//         };
//
//         while all_edges.len() > 0 {
//             let cur_edge = all_edges.pop().unwrap();
//
//             if victim.check_overlap(&cur_edge) {
//                 candidates.push(cur_edge)
//             }
//         }
//
//         Ok(candidates)
//     }
// }