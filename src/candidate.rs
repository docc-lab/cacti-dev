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
use itertools::Itertools;
use petgraph::visit::{EdgeRef, FilterNode};
use petgraph::{dot::Dot, graph::NodeIndex, Direction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use pythia_common::{OSPRequestType, RequestType};
use pythia_common::OSPRequestType::ServerCreate;

use crate::trace::{DAGEdge, IDType, TraceEdge};
use crate::trace::EdgeType;
use crate::trace::Event;
use crate::trace::EventType;
use crate::trace::Trace;
use crate::trace::TracepointID;
use crate::{CriticalPath, PythiaError, Settings};

pub struct CandidateManager {
    // pub victim_type: OSPRequestType,
    pub all_types: Vec<RequestType>,
    pub victim_type: RequestType,
    pub victim_start: String,
    pub victim_end: String,
    pub victim_paths: Vec<CriticalPath>,
    pub non_victim_traces: Vec<Trace>,
    // pub victim_segments: Vec<(Uuid, TraceEdge)>,
    // pub non_victim_segments: HashMap<String, (Uuid, TraceEdge, Option<i64>)>,
    // pub non_victim_segments: HashMap<String, (Uuid, TraceEdge, i64)>,
    pub non_victim_segments: HashMap<String, (IDType, TraceEdge, i64)>,
    pub candidate_segments: Vec<TraceEdge>,
    // pub victim_overlaps: HashMap<Uuid, (TraceEdge, Vec<TraceEdge>)>,
    pub victim_overlaps: HashMap<IDType, (TraceEdge, Vec<TraceEdge>)>,
    // pub old_victim_overlaps: HashMap<Uuid, (TraceEdge, Vec<TraceEdge>)>,
    pub old_victim_overlaps: HashMap<IDType, (TraceEdge, Vec<TraceEdge>)>,
    // pub victim_overlap_max_times: HashMap<Uuid, i64>,
    pub victim_overlap_max_times: HashMap<IDType, i64>,
    used_pairs: HashSet<(String, String)>,
    // pub candidate_groups: HashMap<OSPRequestType, Vec<(i32, i64)>>
    pub candidate_groups: HashMap<RequestType, Vec<(i32, i64)>>
}

// pub fn hash_uuid_and_edge(uuid: Uuid, edge: TraceEdge) -> String {
//     let mut hasher = Sha256::new();
//     hasher.input(uuid.as_bytes());
//     hasher.input(edge.tp_start.as_bytes());
//     hasher.input(edge.tp_end.as_bytes());
//     hasher.result_str()
// }

pub fn hash_id_and_edge(id: IDType, edge: TraceEdge) -> String {
    let mut hasher = Sha256::new();
    match id {
        IDType::UUID(u) => hasher.input(u.as_bytes()),
        IDType::STRING(s) => hasher.input(s.as_bytes())
    }
    hasher.input(edge.tp_start.as_bytes());
    hasher.input(edge.tp_end.as_bytes());
    hasher.result_str()
}

impl CandidateManager {
    pub fn from_settings(settings: &Settings, victim_edge: (&str, &str)) -> CandidateManager {
        let mut to_return = CandidateManager {
            all_types: settings.all_request_types.clone(),
            victim_type: settings.problem_type.clone(),
            victim_start: victim_edge.0.to_string(),
            victim_end: victim_edge.1.to_string(),
            victim_paths: Vec::new(),
            non_victim_traces: Vec::new(),
            // victim_segments: Vec::new(),
            non_victim_segments: HashMap::new(),
            candidate_segments: Vec::new(),
            victim_overlaps: HashMap::new(),
            old_victim_overlaps: HashMap::new(),
            victim_overlap_max_times: HashMap::new(),
            used_pairs: HashSet::new(),
            candidate_groups: HashMap::new(),
        };

        // to_return.candidate_groups.insert(ServerCreate, Vec::new());
        // to_return.candidate_groups.insert(OSPRequestType::ServerList, Vec::new());
        // to_return.candidate_groups.insert(OSPRequestType::ServerDelete, Vec::new());
        // to_return.candidate_groups.insert(OSPRequestType::UsageList, Vec::new());
        // to_return.candidate_groups.insert(OSPRequestType::Unknown, Vec::new());

        for rt in settings.all_request_types.clone() {
            to_return.candidate_groups.insert(rt, Vec::new());
        }

        to_return
    }

    pub fn add_traces(&mut self, traces: Vec<Trace>) {
        for trace in &traces {
            if trace.request_type == self.victim_type {
                let mut new_cp = CriticalPath::from_trace(&trace).unwrap();
                new_cp.g.set_req_type(trace.request_type.clone());
                self.victim_paths.push(new_cp);
            }
            // else {
            self.non_victim_traces.push(trace.clone());
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
            // println!("CRITICAL PATH TRACE TYPE = {}", cur_path.g.request_type);

            // let mut i = 0;

            for edge in cp_edges {
                // i += 1;
                // println!();
                // println!("EDGE:");
                // println!("{:?}", edge);
                // println!();
                if (edge.tp_start == self.victim_start) && (edge.tp_end == self.victim_end) {
                    // self.victim_overlaps.push((cur_path.g.base_id, edge))
                    self.victim_overlaps.insert(cur_path.g.base_id.clone(), (edge, Vec::new()));
                    self.victim_overlap_max_times.insert(cur_path.g.base_id.clone(), now_time.clone());
                }
            }

            // println!();
            // println!("There were [[ {} ]] edges in the victim type!", i);
            // println!();
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
                // let mut hasher = Sha256::new();
                // hasher.input(cur_trace.base_id.as_bytes());
                // hasher.input(edge.tp_start.as_bytes());
                // hasher.input(edge.tp_end.as_bytes());
                // self.non_victim_segments.insert(
                //     hasher.result_str(),
                //     (cur_trace.base_id, edge, now_time.clone())
                // );

                self.non_victim_segments.insert(
                    // hash_uuid_and_edge(cur_trace.base_id, edge.clone()),
                    hash_id_and_edge(cur_trace.base_id.clone(), edge.clone()),
                    (cur_trace.base_id.clone(), edge.clone(), now_time.clone())
                );
            }
        }
    }

    pub fn flush_old_victims(&mut self) {
        let victim_overlaps = self.victim_overlaps.clone();
        for overlap_info in victim_overlaps {
            let latest_overlap_time = &self.victim_overlap_max_times.get(&overlap_info.0).unwrap();
            let now_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_nanos() as i64;
            if *latest_overlap_time + 900*1000000000 < now_time {
                let old_victim = (&mut self.victim_overlaps).remove(&overlap_info.0).unwrap();

                // let mut rt_counts: HashMap<RequestType, i32> = HashMap::new();

                let victim_segment = old_victim.0.clone();
                let vs_latency = victim_segment.end - victim_segment.start;

                // for rt in OSPRequestType::all_types() {
                for rt in self.all_types.clone() {
                    let rt_count = &old_victim.1.clone().into_iter().filter(
                        | c: &TraceEdge | -> bool { c.request_type == rt }
                    ).collect_vec().len();

                    let mut rt_data = self.candidate_groups.get(&rt).unwrap().clone();
                    rt_data.push((rt_count.clone() as i32, vs_latency.clone()));
                    (&mut self.candidate_groups).insert(rt, rt_data);
                }

                // for candidate in &old_victim.1 {
                //     // if self.candidate_groups.contains_key(&candidate.request_type) {
                //     //     let mut rt_data = self.candidate_groups.get(&candidate.request_type).unwrap();
                //     //     rt_data.push()
                //     // }
                //     // else {
                //     //
                //     // }
                //     if rt_counts.contains_key(&candidate.request_type) {
                //         rt_counts.insert(
                //             candidate.request_type,
                //             rt_counts.get(&candidate.request_type).unwrap() + 1
                //         );
                //     }
                //     else {
                //         rt_counts.insert(candidate.request_type, 1);
                //     }
                // }
                //
                // for rtc in rt_counts {
                //     if self.candidate_groups.contains_key(&rtc.0) {
                //         let mut rt_data = self.candidate_groups.get(&rtc.0).unwrap().clone();
                //         rt_data.push((rtc.1, vs_latency.clone()));
                //         (&mut self.candidate_groups).insert(rtc.0, rt_data);
                //     }
                //     else {
                //         let mut rt_data = Vec::new();
                //         rt_data.push((rtc.1, vs_latency.clone()));
                //         (&mut self.candidate_groups).insert(rtc.0, rt_data);
                //     }
                // }

                (&mut self.old_victim_overlaps).insert(
                    overlap_info.0.clone(),
                    old_victim.clone(),
                );
                // self.move_victim_overlaps(&overlap_info.0);
                self.victim_overlap_max_times.remove(&overlap_info.0);
            }
        }

        println!();
        println!();
        println!();
        println!("Flushed old non-victims! Printing candidate groups!");
        println!();
        println!("{:?}", self.candidate_groups);
        println!();
        println!();
        println!();
    }

    // pub fn move_victim_overlaps(victim_uuid: Uuid) {
    //
    // }

    pub fn flush_old_non_victims(&mut self) {
        let now_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards!")
            .as_nanos() as i64;

        let non_victim_segments = self.non_victim_segments.clone();

        for non_victim_info in non_victim_segments {
            // match non_victim_info.1.2 {
            //     Some(t) => {
            //         if t + 900*1000000000 < now_time {
            //             self.non_victim_segments.remove(&non_victim_info.0)
            //         }
            //     },
            //     None => (),
            // }
            if (non_victim_info.1).2 + 900*1000000000 < now_time {
                (&mut self.non_victim_segments).remove(&non_victim_info.0);
            }
        }
    }

    pub fn find_candidates(&mut self) {
        // println!();
        // println!();
        // println!();
        // println!("FIND_CANDIDATES START");
        // println!();
        // println!();

        let now_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards!")
            .as_nanos() as i64;

        let victim_overlaps = self.victim_overlaps.clone();
        let non_victim_segments = self.non_victim_segments.clone();

        // println!("Outermost Loop");
        for victim in &victim_overlaps {
            // println!("Inner Loop 1");
            for non_victim in &non_victim_segments {
                // println!("Pre-Uuid check");
                if victim.0.clone() != (non_victim.1).0 {
                    // println!("Post-Uuid check");
                    let hash_to_check = (
                        // hash_uuid_and_edge(victim.0, (victim.1).0.clone()),
                        // hash_uuid_and_edge((non_victim.1).0, (non_victim.1.clone()).1),
                        hash_id_and_edge(victim.0.clone(), (victim.1).0.clone()),
                        hash_id_and_edge((non_victim.1).0.clone(), (non_victim.1.clone()).1),
                    );
                    if !self.used_pairs.contains(&hash_to_check) {
                        // println!("Post-used-pairs check");
                        if (victim.1).0.overlaps_with(&(non_victim.1).1) {
                            // println!("Overlap found!!");
                            let mut new_overlaps = (victim.1).1.clone();
                            new_overlaps.push((non_victim.1.clone()).1);
                            (&mut self.victim_overlaps).insert(
                                victim.0.clone(),
                                ((victim.1).0.clone(), new_overlaps)
                            );
                            (&mut self.victim_overlap_max_times).insert(
                                victim.0.clone(),
                                now_time.clone()
                            );
                            (&mut self.non_victim_segments).insert(
                                non_victim.0.clone(),
                                ((non_victim.1).0.clone(), (non_victim.1).1.clone(), now_time.clone())
                            );
                            ((&mut self.used_pairs).insert(hash_to_check));
                        }
                    }
                }
            }
        }
        // println!("Outermost Loop");
        // println!("FIND_CANDIDATES END");
        // println!();
        // println!();
        // println!();
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