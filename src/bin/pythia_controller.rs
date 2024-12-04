/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

#[macro_use]
extern crate lazy_static;

use std::cmp::Ordering;
// use std::collections::HashSet;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
// use std::slice::range;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use futures::Stream;
use itertools::{Itertools, max};
use petgraph::csr::EdgeIndex;
use petgraph::data::DataMap;
use petgraph::visit::IntoEdges;
use pythia_common::{OSPRequestType, RequestType};
use stats::{mean, median, stddev, variance};

use threadpool::ThreadPool;

use pythia::budget::BudgetManager;
use pythia::candidate::CandidateManager;
use pythia::controller::controller_from_settings;
use pythia::controller::Controller;
use pythia::critical::CriticalPath;
use pythia::critical::Path;
use pythia::grouping::{Group, GroupEdge, GroupManager};
use pythia::manifest::Manifest;
use pythia::reader::reader_from_settings;
use pythia::search::get_strategy;
use pythia::settings::{ApplicationType, Settings};
use pythia::spantrace::{Feature, Feature2, Span, SpanTrace};
use pythia::trace::{DAGEdge, Event, IDType, Trace, TraceNode, TracepointID};

// // use keccak_hash::keccak256;
// use sha3;

// These are static because search strategy expects static references.
lazy_static! {
    static ref SETTINGS: Settings = Settings::read();
    static ref CONTROLLER: Box<dyn Controller> = controller_from_settings(&SETTINGS);
    static ref MANIFEST: Manifest = Manifest::from_file(&SETTINGS.manifest_file.as_path())
        .expect("Couldn't read manifest from cache");
}

fn reset_reader() {
    let mut reader = reader_from_settings(&SETTINGS);
    reader.reset_state();
}

// Search for:
// -- nova/usr/local/lib/python3.6/dist-packages/nova/virt/libvirt/driver.py:5486:nova.virt.libvirt.driver.LibvirtDriver._get_guest_xml
// to
// -- nova/usr/local/lib/python3.6/dist-packages/nova/hooks.py:2046:nova.compute.manager.ComputeManager._do_build_and_run_instance

// More specifically:
// -- nova/usr/local/lib/python3.6/dist-packages/nova/compute/manager.py:1972:_locked_do_build_and_run_instance:_build_semaphore._waiters
// to
// -- nova/usr/local/lib/python3.6/dist-packages/nova/hooks.py:2046:nova.compute.manager.ComputeManager._do_build_and_run_instance

fn cacti_no_loop() -> bool {
    true
}

/// Main Pythia function that runs in a loop and makes decisions
fn main() {
    if cacti_no_loop() {
        let mut quit_in = -1;
        let mut targets = HashSet::new();
        // The targets are set here. Any typos, and Pythia won't stop.
        // targets.insert(TracepointID::from_str("nova/usr/local/lib/python3.6/dist-packages/nova/compute/manager.py:1859:nova.compute.manager.ComputeManager._update_scheduler_instance_info"));
        targets.insert("");
        eprintln!("Targets are {:?}", targets);

        let filename = std::env::args().nth(1).unwrap();
        eprintln!("Printing results to {}", filename);
        // eprintln!("All args = [{:?}]", std::env::args().into_iter().collect::<Vec<String>>());
        let mut output_file = File::create(filename).unwrap();
        writeln!(output_file, "{:?}", *SETTINGS).ok();
        writeln!(output_file, "Targets: {:?}", targets).ok();

        // let problem_type = std::env::args().nth(2).unwrap();

        // let mut temp_settings = Settings::read();
        // temp_settings.problem_type = RequestType::from_str(
        //     problem_type.as_str(), &SETTINGS.application.as_str()).unwrap();

        let mut reader = reader_from_settings(&SETTINGS);

        // let problem_traces = reader.get_recent_span_traces();
        //
        // let problem_path_traces = problem_traces.iter().map(
        //     |st| st.to_critical_path()).collect::<Vec<Trace>>();
        //
        // println!("EXAMPLE PATH TRACE:");
        // println!("{:?}", problem_path_traces[0]);
        // println!();
        // println!();
        // println!();
        // println!();
        // println!();
        //
        // let problem_paths = problem_path_traces
        //     .iter().map(|ppt| {
        //     let mut cp = CriticalPath::from_trace(ppt).unwrap();
        //     cp.request_type = ppt.request_type.clone();
        //     cp.start_node = ppt.start_node;
        //     cp.end_node = ppt.end_node;
        //     cp
        // })
        //     .collect::<Vec<CriticalPath>>();
        //
        // let groups = Group::from_critical_paths(problem_paths.clone()).into_iter()
        //     .filter(|g| g.traces.len() > 1).collect::<Vec<Group>>();
        //
        // println!("SAMPLE GROUP:");
        // println!("{:?}", groups[0]);
        //
        // println!();
        // println!();
        // println!();
        // println!();
        // println!();
        //
        // println!("PROBLEM EDGES:");
        // println!("{:?}", groups[0].problem_edges());

        // let mut group_manager = GroupManager::new();
        // group_manager.update(&problem_paths);
        //
        // println!();
        // println!();
        // println!();
        // println!();
        // println!();
        //
        // println!("TOTAL GROUPS --- {}", group_manager.all_groups().len());
        // println!("PROBLEM GROUPS CV --- {}", group_manager.problem_groups_cv(0.05).len());
        // println!("SLOW GROUPS --- {}", group_manager.problem_groups_slow(90.0).len());
        //
        // let sample_problem_group = group_manager
        //     .problem_groups_cv(0.05)[0];
        // let sample_problem_edges = sample_problem_group
        //     .problem_edges()[..5].into_iter()
        //     .map(|&ei| sample_problem_group.g.edge_weight(ei).unwrap().clone())
        //     .collect::<Vec<GroupEdge>>();
        //
        // println!();
        // println!();
        // println!();
        //
        // println!("SAMPLE PROBLEM EDGES:");
        // for edge in sample_problem_edges {
        //     println!("{:?}", edge);
        // }
        //
        // let sample_problem_edge_endpoints = sample_problem_group
        //     .problem_edges()[..5].into_iter()
        //     .map(|&ei| {
        //         let ee = sample_problem_group.g
        //             .edge_endpoints(ei).unwrap().clone();
        //         let ee_start = sample_problem_group.g
        //             .node_weight(ee.0).unwrap().clone();
        //         let ee_end = sample_problem_group.g
        //             .node_weight(ee.1).unwrap().clone();
        //
        //         (ee_start, ee_end)
        //     })
        //     .collect::<Vec<(TraceNode, TraceNode)>>();
        //
        // println!();
        // println!();
        // println!();
        //
        // println!("SAMPLE PROBLEM EDGES (RAW):");
        // // for (i, edge) in sample_problem_edges_dag.into_iter().enumerate() {
        // for (i, edge) in sample_problem_edge_endpoints.into_iter().enumerate() {
        //     // println!("{:?}", sample_problem_edge_endpoints[i].0);
        //     // println!("{:?}", edge);
        //     // println!("{:?}", sample_problem_edge_endpoints[i].1);
        //     println!("{:?}", edge.0);
        //     println!("{:?}", edge.1);
        //     println!();
        // }
        //
        // println!();
        // println!();
        // println!();
        //
        // println!("SAMPLE LONG EDGE RID:");
        // println!(
        //     "{:?}",
        //     sample_problem_group.traces.clone().into_iter().map(|tr| tr.request_id).collect::<Vec<IDType>>()
        // );

        // let problem_groups = group_manager.problem_groups_cv(0.05);
        // // let top_problem_edges = problem_groups.into_iter()
        // //     .map(|g| {
        // //         let ee = g.g.edge_endpoints(g.problem_edges()[0]).unwrap();
        // //         let ee_start = sample_problem_group.g
        // //             .node_weight(ee.0).unwrap().clone();
        // //         let ee_end = sample_problem_group.g
        // //             .node_weight(ee.1).unwrap().clone();
        // //
        // //         (ee_start, ee_end)
        // //     })
        // //     .collect::<Vec<(TraceNode, TraceNode)>>();

        // // TODO: Make the # of groups selected a configurable parameter
        // let top_problem_groups = group_manager.problem_groups_cv(0.05)[..10]
        //     .into_iter().map(|&g| g.clone()).collect::<Vec<Group>>();
        //
        // let mut top_problem_edges: HashMap<String, (TraceNode, TraceNode)> = HashMap::new();
        // for g in top_problem_groups {
        //     let ee = g.g.edge_endpoints(g.problem_edges()[0]).unwrap();
        //     let ee_start = sample_problem_group.g
        //         .node_weight(ee.0).unwrap().clone();
        //     let ee_end = sample_problem_group.g
        //         .node_weight(ee.1).unwrap().clone();
        //
        //     top_problem_edges.insert(g.hash().to_string(), (ee_start, ee_end));
        // }
        //
        // sleep(Duration::from_micros(SETTINGS.cycle_lookback as u64));
        //
        // reader.set_fetch_all();
        //
        // let off_pl_traces = reader.get_recent_span_traces();
        //
        // let mut problem_type_traces = Vec::new();
        // // let mut non_problem_traces = Vec::new();
        // let mut non_problem_traces = HashMap::new();
        //
        // for tr in off_pl_traces {
        //     if RequestType::from_str(
        //         tr.endpoint_type.as_str(),
        //         SETTINGS.application.as_str()
        //     ).unwrap() == SETTINGS.problem_type.clone() {
        //         problem_type_traces.push(tr);
        //     }
        //     else {
        //         // non_problem_traces.push(tr);
        //         non_problem_traces.insert(tr.req_id.clone(), tr);
        //     }
        // }
        //
        // let pt_traces = problem_traces.iter().map(
        //     |st| st.to_critical_path()).collect::<Vec<Trace>>();
        //
        // let pt_crits = pt_traces
        //     .iter().map(|ppt| {
        //     let mut cp = CriticalPath::from_trace(ppt).unwrap();
        //     cp.request_type = ppt.request_type.clone();
        //     cp.start_node = ppt.start_node;
        //     cp.end_node = ppt.end_node;
        //     cp
        // })
        //     .collect::<Vec<CriticalPath>>();
        //
        // // for s in top_problem_edges.into_iter() {
        // //
        // // }
        //
        // println!();
        // println!();
        // println!();
        // println!();
        // println!();

        // for cp in pt_crits {
        //     match top_problem_edges.get(cp.hash()) {
        //         Some ((tns, tne)) => {
        //             let (ts, te, edge) = cp.get_by_tracepoints(
        //                 tns.tracepoint_id, tne.tracepoint_id);
        //
        //             let overlaps = reader.get_candidate_events(
        //                 ts.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64,
        //                 te.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64,
        //                 edge.host.unwrap()
        //             );
        //
        //             println!("OVERLAPPING EDGES:");
        //             for o in overlaps {
        //                 println!(
        //                     "{:?}",
        //                     non_problem_traces.get(o.0.as_str()).unwrap()
        //                         .spans.get(o.1.as_str()).unwrap()
        //                 )
        //             }
        //         },
        //         None => continue
        //     }
        // }

        ////////////////////////////////////////

        // println!();
        // println!();
        // println!("PHASE 1");
        // println!();
        // println!();
        // let problem_traces = reader.get_recent_span_traces();
        // 
        // println!();
        // println!("PT Len = {}", problem_traces.len());
        // println!();
        // 
        // let problem_path_traces = problem_traces.iter().map(
        //     |st| st.to_critical_path()).collect::<Vec<Trace>>();
        // 
        // println!();
        // println!("PPT Len = {}", problem_traces.len());
        // println!();
        // 
        // let problem_paths = problem_path_traces
        //     .iter().map(|ppt| {
        //     let mut cp = CriticalPath::from_trace(ppt).unwrap();
        //     cp.request_type = ppt.request_type.clone();
        //     cp.start_node = ppt.start_node;
        //     cp.end_node = ppt.end_node;
        //     cp
        // })
        //     .collect::<Vec<CriticalPath>>();
        // 
        // println!("# problem traces = {}", problem_paths.len());
        // 
        // // let groups = Group::from_critical_paths(problem_paths.clone()).into_iter()
        // //     .filter(|g| g.traces.len() > 1).collect::<Vec<Group>>();
        // 
        // let mut group_manager = GroupManager::new();
        // group_manager.update(&problem_paths);
        // 
        // // let problem_groups = group_manager.problem_groups_cv(0.05);
        // 
        // // TODO: Make the # of groups selected a configurable parameter
        // let problem_groups = group_manager.problem_groups_cv(0.05);
        // let top_problem_groups = problem_groups[..std::cmp::min(10, problem_groups.len())]
        //     .into_iter().map(|&g| g.clone()).collect::<Vec<Group>>();
        // 
        // let mut top_problem_edges: HashMap<String, (TraceNode, TraceNode)> = HashMap::new();
        // for g in top_problem_groups {
        //     let ee = g.g.edge_endpoints(g.problem_edges()[0]).unwrap();
        //     let ee_start = g.g
        //         .node_weight(ee.0).unwrap().clone();
        //     let ee_end = g.g
        //         .node_weight(ee.1).unwrap().clone();
        // 
        //     top_problem_edges.insert(g.hash().to_string(), (ee_start, ee_end));
        // }

        // sleep(Duration::from_micros(SETTINGS.cycle_lookback as u64));
        println!();
        println!();
        println!("PHASE 2");
        println!();
        println!();

        // println!();
        // println!("Sleep start");
        // // sleep(Duration::from_millis(60000));
        // sleep(Duration::from_millis(6000));
        // println!("Sleep end");
        // println!();
        // println!();

        reader.set_fetch_all();

        let off_pl_traces = reader.get_recent_span_traces();
        
        println!("off_pl_traces.len() = {}", off_pl_traces.len());

        let mut problem_type_traces = Vec::new();
        // let mut non_problem_traces = Vec::new();
        let mut non_problem_traces = HashMap::new();

        println!();
        println!();
        for tr in off_pl_traces {
            // println!();
            // println!("INSERTING TRACE: [[{:?}]]", tr.clone());
            // println!();

            // TODO: Re-Enable this to log trace types for all requests
            println!("TRACE TYPE = {}", RequestType::from_str(
                tr.endpoint_type.as_str(),
                SETTINGS.application.as_str()
            ).unwrap());
            println!("TRACE SPAN COUNT = {}", tr.spans.len());
            println!();

            if RequestType::from_str(
                tr.endpoint_type.as_str(),
                SETTINGS.application.as_str()
            ).unwrap() == SETTINGS.problem_type.clone() {
                problem_type_traces.push(tr.clone());
            }
            // else {
            //     // non_problem_traces.push(tr);
            //     non_problem_traces.insert(tr.req_id.clone(), tr);
            // }
            non_problem_traces.insert(tr.req_id.clone(), tr);
        }
        println!();
        println!();

        let pt_traces = problem_type_traces.iter().map(
            |st| st.to_critical_path()).collect::<Vec<Trace>>();

        let pt_crits = pt_traces
            .iter().map(|ppt| {
            // let mut cp = CriticalPath::from_trace(ppt).unwrap();
            let mut cp = CriticalPath::from_cp_trace(ppt);
            cp.request_type = ppt.request_type.clone();
            cp.start_node = ppt.start_node;
            cp.end_node = ppt.end_node;
            cp
        })
            .collect::<Vec<CriticalPath>>();

        println!("pt_crits.len() = {}", pt_crits.len());
        
        let mut victim_req_ids = Vec::new();
        for ptc in &pt_crits {
            victim_req_ids.push((ptc.request_id.clone(), ptc.duration));
        }
        victim_req_ids.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap()
        });
        
        // let survivor_req_ids = victim_req_ids
        //     .drain(..((0.9*(victim_req_ids.len() as f64)) as usize))
        //     .collect::<Vec<(IDType, Duration)>>();
        
        let _ = victim_req_ids
            .drain(..((0.9*(victim_req_ids.len() as f64)) as usize));
        
        let mut victim_rid_set = HashSet::new();
        // let mut survivor_rid_set = HashSet::new();

        for (vid, _) in victim_req_ids {
            victim_rid_set.insert(vid);
        }
        // for (sid, _) in survivor_req_ids {
        //     survivor_rid_set.insert(sid);
        // }

        // println!();
        // println!();
        // println!();
        // println!();
        // println!();
        // println!("{:?}", non_problem_traces);
        // println!();
        // println!();
        // println!();
        // println!();
        // println!();

        println!("PHASE 2.1");

        /*~
         * Edge grouping code:
         * Extracts all edges from all problem-trace critical paths and puts them into groups
         * Gathers summary stats for each edge group, as well as PCC with
        ~*/

        // Contains identifying information about a particular edge (by ID)
        #[derive(Clone, Debug)]
        struct EdgeGroup {
            pub ts: String,
            pub te: String,
            pub mean: u64,
            pub var: u64,
            pub cov: f64,
            pub pcc: f64,
            pub vs_diff: f64,
            pub latencies: Vec<(IDType, u64, u64)>
        }

        impl EdgeGroup {
            pub fn new(edge: &DAGEdge, ts: &Event, te: &Event, parent_lat: u64, req_id: &IDType) -> EdgeGroup {
                let mut to_return = EdgeGroup{
                    ts: ts.tracepoint_id.to_string(),
                    te: te.tracepoint_id.to_string(),
                    mean: 0,
                    var: 0,
                    cov: 0.0,
                    pcc: 0.0,
                    vs_diff: 0.0,
                    latencies: vec![],
                };

                // TODO: fix this hacky stuff soon
                if (edge.duration.as_nanos() as u64) < 10000000000000000000 {
                    to_return.latencies.push(
                        (req_id.clone(), edge.duration.as_nanos() as u64, parent_lat));
                    // to_return.mean = edge.duration.as_nanos() as u64;
                }

                // return EdgeGroup{
                //     ts: ts.tracepoint_id.to_string(),
                //     te: te.tracepoint_id.to_string(),
                //     mean: edge.duration.as_nanos() as u64,
                //     var: 0,
                //     pcc: 0,
                //     latencies: vec![(ts.trace_id.clone(), edge.duration.as_nanos() as u64, parent_lat)],
                // }
                return to_return;
            }

            pub fn add_edge(&mut self, edge: &DAGEdge, trace_id: &IDType, trace_lat: u64) {
                // TODO: fix this hacky stuff soon
                if (edge.duration.as_nanos() as u64) < 10000000000000000000 {
                    self.latencies.push(
                        (trace_id.clone(), edge.duration.as_nanos() as u64, trace_lat));
                }
            }

            pub fn compute_stats(&mut self, victim_ids: &HashSet<IDType>) {
                let latencies_iter = self.latencies.clone().into_iter()
                    .map(|e| e.1).collect::<Vec<u64>>()
                    .into_iter();
                let resp_times_iter = self.latencies.clone().into_iter()
                    .map(|e| e.2).collect::<Vec<u64>>()
                    .into_iter();
                
                self.var = variance(latencies_iter.clone()) as u64;
                self.mean = mean(latencies_iter.clone()) as u64;
                let rt_mean = mean(resp_times_iter.clone()) as u64;

                let mut pcc_num = 0i128;
                for (_, ed, rt) in &self.latencies {
                    pcc_num += (*ed as i128 - self.mean as i128)*(*rt as i128 - rt_mean as i128);
                }

                self.cov = (pcc_num as f64)/((self.latencies.len() as f64));

                self.pcc = (pcc_num as f64)/
                    ((self.latencies.len() as f64)*
                        (stddev(latencies_iter.clone())*
                            stddev(resp_times_iter.clone())));
                
                let mut victims = Vec::new();
                let mut survivors = Vec::new();
                
                for (tid, lat, _) in &self.latencies {
                    if victim_ids.contains(tid) {
                        victims.push(lat.clone());
                    } else {
                        survivors.push(lat.clone());
                    }
                }
                
                self.vs_diff = mean(victims.into_iter()) - mean(survivors.into_iter());
            }
            
            pub fn get_median(&self) -> u64 {
                median(
                    self.latencies.clone().into_iter()
                        .map(|e| e.1).collect::<Vec<u64>>()
                        .into_iter()
                ).unwrap() as u64
            }
            
            pub fn slow_med_diff(&self) -> u64 {
                let mut lats_sorted= self.latencies.clone().into_iter()
                    .map(|e| e.1).collect::<Vec<u64>>();
                
                lats_sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());

                // println!("ls0 = {}; sgm = {}", lats_sorted[0], self.get_median());
                
                lats_sorted[((lats_sorted.len() as f64)*0.01) as usize] - self.get_median()
            }
        }

        println!("PHASE 2.2");

        let mut edge_groups: HashMap<String, EdgeGroup> = HashMap::new();
        let mut eg_keys = HashSet::new();
        for cp in &pt_crits {
            for ei in cp.g.g.edge_indices() {
                let edge = cp.g.g.edge_weight(ei).unwrap();
                let (epi_s, epi_e) = cp.g.g.edge_endpoints(ei).unwrap();
                let t_start = cp.g.g.node_weight(epi_s).unwrap();
                let t_end = cp.g.g.node_weight(epi_e).unwrap();

                let key = t_start.tracepoint_id.to_string() + "::" + t_end.tracepoint_id.to_string().as_str();
                eg_keys.insert(key.clone());
                match edge_groups.get_mut(key.as_str()) {
                    Some(eg) => {
                        // eg.add_edge(edge, &t_start.trace_id, cp.duration.as_nanos() as u64);
                        eg.add_edge(edge, &cp.request_id, cp.duration.as_nanos() as u64);
                    }
                    None => {
                        edge_groups.insert(
                            key,
                            EdgeGroup::new(edge, t_start, t_end, cp.duration.as_nanos() as u64, &cp.request_id)
                        );
                    }
                }
            }
        }

        for k in &eg_keys {
            // if edge_groups.get(k.as_str()).unwrap().latencies.len() <= 1 {
            // if edge_groups.get(k.as_str()).unwrap().latencies.len() <= 3 {
            if edge_groups.get(k.as_str()).unwrap().latencies.len() <= 10 {
                edge_groups.remove(k.as_str());
            }
        }

        println!("# of edge groups: {}", eg_keys.len());

        let mut eg_keys = Vec::new();
        let mut f_eg_keys = Vec::new();

        for (k, e) in edge_groups.iter_mut() {
            eg_keys.push(k.clone());
            e.compute_stats(&victim_rid_set);
        }

        let mut filtered_eg: HashMap<String, EdgeGroup> = HashMap::new();

        for k in &eg_keys {
            let eg_mean = edge_groups[k.as_str()].mean;
            let eg_cov = edge_groups[k.as_str()].cov;
            // if eg_mean == 0 || eg_cov == 0.0 {
            //     edge_groups.remove(k.as_str());
            // }

            if eg_mean != 0 && eg_cov != 0.0 {
                filtered_eg.insert(k.clone(), edge_groups[k.as_str()].clone());
            }

            f_eg_keys.push(k.clone());
        }

        println!("# of edge groups (post-filter): {}", filtered_eg.keys().collect::<Vec<&String>>().len());

        // let mut eg_var_sorted: Vec<(String, EdgeGroup)> = Vec::new();
        // for k in &eg_keys {
        //     eg_var_sorted.push((k.clone(), edge_groups.get(k.as_str()).unwrap().clone()));
        // }
        // eg_var_sorted.sort_by(
        //     |a, b| b.1.var.partial_cmp(&a.1.var).unwrap()
        // );

        let mut eg_pcc_sorted: Vec<(String, EdgeGroup)> = Vec::new();
        for k in &f_eg_keys {
            match filtered_eg.get(k.as_str()) {
                Some(eg) => {
                    eg_pcc_sorted.push((k.clone(), eg.clone()));
                }
                _ => {}
            }
        }
        eg_pcc_sorted.sort_by(
            |a, b| {
                if b.1.pcc.is_nan() {
                    println!("NaN Edge Group Len: {}", b.1.latencies.len());
                    println!("NaN Edge Group Edge: ({})", b.0.clone());
                    println!("NaN Edge Group Latencies:\n{:?}", b.1.clone())
                }
                if a.1.pcc.is_nan() {
                    println!("NaN Edge Group Len: {}", a.1.latencies.len());
                    println!("NaN Edge Group Edge: ({})", a.0.clone());
                    println!("NaN Edge Group Latencies:\n{:?}", a.1.clone())
                }
                println!("{} : {}", b.1.pcc, &a.1.pcc);
                b.1.pcc.partial_cmp(&a.1.pcc).unwrap()
            }
        );

        let mut eg_cov_sorted: Vec<(String, EdgeGroup)> = Vec::new();
        for k in &f_eg_keys {
            match filtered_eg.get(k.as_str()) {
                Some(eg) => {
                    eg_cov_sorted.push((k.clone(), eg.clone()));
                }
                _ => {}
            }
        }
        eg_cov_sorted.sort_by(
            |a, b| {
                if b.1.cov.is_nan() {
                    println!("NaN Edge Group Len: {}", b.1.latencies.len());
                    println!("NaN Edge Group Edge: ({})", b.0.clone());
                    println!("NaN Edge Group Latencies:\n{:?}", b.1.clone())
                }
                if a.1.cov.is_nan() {
                    println!("NaN Edge Group Len: {}", a.1.latencies.len());
                    println!("NaN Edge Group Edge: ({})", a.0.clone());
                    println!("NaN Edge Group Latencies:\n{:?}", a.1.clone())
                }
                println!("{} : {}", b.1.cov, &a.1.cov);
                b.1.cov.partial_cmp(&a.1.cov).unwrap()
            }
        );

        let mut eg_diff_sorted: Vec<(String, EdgeGroup)> = Vec::new();
        // for k in &eg_keys {
        //     eg_diff_sorted.push((k.clone(), edge_groups.get(k.as_str()).unwrap().clone()));
        // }
        for k in &eg_keys {
            match edge_groups.get(k.as_str()) {
                Some(eg) => {
                    eg_diff_sorted.push((k.clone(), eg.clone()));
                }
                _ => {}
            }
        }
        eg_diff_sorted.sort_by(
            |a, b| b.1.slow_med_diff().partial_cmp(&a.1.slow_med_diff()).unwrap()
        );

        let mut eg_vs_sorted: Vec<(String, EdgeGroup)> = Vec::new();
        // for k in &eg_keys {
        //     eg_diff_sorted.push((k.clone(), edge_groups.get(k.as_str()).unwrap().clone()));
        // }
        for k in &eg_keys {
            match edge_groups.get(k.as_str()) {
                Some(eg) => {
                    eg_vs_sorted.push((k.clone(), eg.clone()));
                }
                _ => {}
            }
        }
        eg_vs_sorted.sort_by(
            |a, b| b.1.vs_diff.partial_cmp(&a.1.vs_diff).unwrap()
        );
        

        println!("HHE Metric (PCC) = {}", eg_pcc_sorted[0].1.pcc);
        println!("HHE Winner Len (PCC) = {}", eg_pcc_sorted[0].1.latencies.len());
        let hhe_parts_pcc = eg_pcc_sorted[0].0.split("::").collect::<Vec<&str>>();
        let (hhe_start_pcc, hhe_end_pcc) = (hhe_parts_pcc[0].to_string(), hhe_parts_pcc[1].to_string());
        println!();
        println!();
        println!("HHE (PCC) = ({}, {})", hhe_start_pcc, hhe_end_pcc);
        println!();
        println!("HHE List (PCC) = [");
        let mut hhe_pcc_ctr = 0;
        loop {
            if hhe_pcc_ctr >= eg_pcc_sorted.len() {
                break;
            }

            let pcc_parts = eg_pcc_sorted[hhe_pcc_ctr].0.split("::").collect::<Vec<&str>>();
            let (pcc_start, pcc_end) = (pcc_parts[0].to_string(), pcc_parts[1].to_string());

            println!("HHE (PCC) Pos {} = ({}, {}) --- Score = [ {} ]", hhe_pcc_ctr, pcc_start, pcc_end, eg_pcc_sorted[hhe_pcc_ctr].1.pcc);

            hhe_pcc_ctr += 1;
        }
        println!("]");
        println!();
        println!();

        let mut latencies_sorted = eg_pcc_sorted[0].1.latencies.clone();
        latencies_sorted.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap()
        });
        // println!("{:?}", latencies_sorted);

        println!();
        println!();
        println!();
        println!();

        println!("HHE Metric (Cov) = {}", eg_cov_sorted[0].1.cov);
        println!("HHE Winner Len (Cov) = {}", eg_cov_sorted[0].1.latencies.len());
        let hhe_parts_cov = eg_cov_sorted[0].0.split("::").collect::<Vec<&str>>();
        let (hhe_start_cov, hhe_end_cov) = (hhe_parts_cov[0].to_string(), hhe_parts_cov[1].to_string());
        println!();
        println!();
        println!("HHE (Cov) = ({}, {})", hhe_start_cov, hhe_end_cov);
        println!();
        println!("HHE List (Cov) = [");
        let mut hhe_cov_ctr = 0;
        loop {
            if hhe_cov_ctr >= eg_cov_sorted.len() {
                break;
            }

            let cov_parts = eg_cov_sorted[hhe_cov_ctr].0.split("::").collect::<Vec<&str>>();
            let (cov_start, cov_end) = (cov_parts[0].to_string(), cov_parts[1].to_string());

            println!("HHE (Cov) Pos {} = ({}, {}) --- Score = [ {} ]", hhe_cov_ctr, cov_start, cov_end, eg_cov_sorted[hhe_cov_ctr].1.cov);

            hhe_cov_ctr += 1;
        }
        println!("]");
        println!();
        println!();

        latencies_sorted = eg_cov_sorted[0].1.latencies.clone();
        latencies_sorted.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap()
        });
        // println!("{:?}", latencies_sorted);

        println!();
        println!();
        println!();
        println!();

        /*~ End edge grouping code ~*/

        let mut hhe_index = 0;
        loop {
            if eg_diff_sorted[hhe_index].1.slow_med_diff() < 10000000000000000000 {
                break;
            }

            hhe_index += 1;
        }

        println!("HHE Metric (Diff) = {}", eg_diff_sorted[hhe_index].1.slow_med_diff());
        println!("HHE Winner Len (Diff) = {}", eg_diff_sorted[0].1.latencies.len());
        // println!("HHE Metric = {}", eg_diff_sorted[1].1.slow_med_diff());
        // println!("HHE Metric = {}", eg_diff_sorted[2].1.slow_med_diff());
        // let mut hhe_parts = eg_diff_sorted[0].0.split("::").collect::<Vec<&str>>();
        // let (hhe_start, hhe_end) = (hhe_parts[0].to_string(), hhe_parts[1].to_string());
        //
        // println!();
        // println!();
        // println!("HHE = ({}, {})", hhe_start, hhe_end);
        //
        // let hhe_parts_2 = eg_diff_sorted[1].0.split("::").collect::<Vec<&str>>();
        // let (hhe_start_2, hhe_end_2) = (hhe_parts_2[0].to_string(), hhe_parts_2[1].to_string());
        // println!("HHE = ({}, {})", hhe_start_2, hhe_end_2);
        //
        // let hhe_parts_3 = eg_diff_sorted[2].0.split("::").collect::<Vec<&str>>();
        // let (hhe_start_3, hhe_end_3) = (hhe_parts_3[0].to_string(), hhe_parts_3[1].to_string());
        // println!("HHE = ({}, {})", hhe_start_3, hhe_end_3);
        // println!();
        // println!();

        let hhe_parts_diff = eg_diff_sorted[hhe_index].0.split("::").collect::<Vec<&str>>();
        let (hhe_start_diff, hhe_end_diff) = (hhe_parts_diff[0].to_string(), hhe_parts_diff[1].to_string());
        println!();
        println!();
        println!("HHE (Diff) = ({}, {})", hhe_start_diff, hhe_end_diff);
        println!();
        println!("HHE List (Diff) = [");
        let mut hhe_diff_ctr = hhe_index;
        loop {
            if hhe_diff_ctr >= eg_diff_sorted.len() {
                break;
            }

            let diff_parts = eg_diff_sorted[hhe_diff_ctr].0.split("::").collect::<Vec<&str>>();
            let (diff_start, diff_end) = (diff_parts[0].to_string(), diff_parts[1].to_string());

            println!("HHE (Diff) Pos {} = ({}, {}) --- Score = [ {} ]", hhe_diff_ctr-hhe_index, diff_start, diff_end, eg_diff_sorted[hhe_diff_ctr].1.slow_med_diff());

            hhe_diff_ctr += 1;
        }
        println!("]");
        println!();
        println!();

        latencies_sorted = eg_diff_sorted[hhe_index].1.latencies.clone();
        latencies_sorted.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap()
        });
        // println!("{:?}", latencies_sorted);
        println!();
        println!();
        println!();
        println!();

        println!("HHE Metric (V/S) = {}", eg_vs_sorted[0].1.pcc);
        println!("HHE Winner Len (V/S) = {}", eg_vs_sorted[0].1.latencies.len());
        let hhe_parts_vs = eg_vs_sorted[0].0.split("::").collect::<Vec<&str>>();
        let (hhe_start_vs, hhe_end_vs) = (hhe_parts_vs[0].to_string(), hhe_parts_vs[1].to_string());
        println!();
        println!();
        println!("HHE (V/S) = ({}, {})", hhe_start_vs, hhe_end_vs);
        println!();
        println!("HHE List (V/S) = [");
        let mut hhe_vs_ctr = 0;
        loop {
            if hhe_vs_ctr >= eg_vs_sorted.len() {
                break;
            }

            let vs_parts = eg_vs_sorted[hhe_vs_ctr].0.split("::").collect::<Vec<&str>>();
            let (vs_start, vs_end) = (vs_parts[0].to_string(), vs_parts[1].to_string());

            println!("HHE (V/S) Pos {} = ({}, {}) --- Score = [ {} ]", hhe_vs_ctr, vs_start, vs_end, eg_vs_sorted[hhe_vs_ctr].1.vs_diff);

            hhe_vs_ctr += 1;
        }
        println!("]");
        println!();
        println!();

        let mut latencies_sorted = eg_vs_sorted[0].1.latencies.clone();
        latencies_sorted.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap()
        });
        // println!("{:?}", latencies_sorted);
        println!();
        println!();
        println!();
        println!();

        let mut pcc_index = 0i64;
        let mut cov_index = 0i64;
        let mut diff_index = 0i64;
        let mut vs_index = 0i64;

        loop {
            if (pcc_index as usize) == eg_pcc_sorted.len() {
                pcc_index = -1;
                break;
            }

            let hhe_parts_pcc = eg_pcc_sorted[pcc_index as usize].0.split("::").collect::<Vec<&str>>();
            let (hhe_start_pcc, hhe_end_pcc) = (hhe_parts_pcc[0].to_string(), hhe_parts_pcc[1].to_string());

            // if hhe_start_pcc.contains("ts-order-service") && hhe_end_pcc.contains("ts-order-service") {
            // if hhe_start_pcc.contains("ts-route-service") || hhe_end_pcc.contains("ts-route-service") {
            // if hhe_start_pcc.contains("ts-price-service") || hhe_end_pcc.contains("ts-price-service") {
            if hhe_start_pcc.contains("ts-station-service") || hhe_end_pcc.contains("ts-station-service") {
                break;
            }

            pcc_index += 1;
        }

        loop {
            if (cov_index as usize) == eg_cov_sorted.len() {
                cov_index = -1;
                break;
            }

            let hhe_parts_cov = eg_cov_sorted[cov_index as usize].0.split("::").collect::<Vec<&str>>();
            let (hhe_start_cov, hhe_end_cov) = (hhe_parts_cov[0].to_string(), hhe_parts_cov[1].to_string());

            // if hhe_start_cov.contains("ts-order-service") && hhe_end_cov.contains("ts-order-service") {
            // if hhe_start_cov.contains("ts-route-service") || hhe_end_cov.contains("ts-route-service") {
            // if hhe_start_cov.contains("ts-price-service") || hhe_end_cov.contains("ts-price-service") {
            if hhe_start_cov.contains("ts-station-service") || hhe_end_cov.contains("ts-station-service") {
                break;
            }

            cov_index += 1;
        }
        //
        // loop {
        //     let hhe_parts_pcc = eg_pcc_sorted[pcc_index].0.split("::").collect::<Vec<&str>>();
        //     let (hhe_start_pcc, hhe_end_pcc) = (hhe_parts_pcc[0].to_string(), hhe_parts_pcc[1].to_string());
        //
        //     if hhe_start_pcc.contains("ts-order-service") && hhe_end_pcc.contains("ts-order-service") {
        //         break;
        //     }
        //
        //     pcc_index += 1;
        // }

        loop {
            if (hhe_index + (diff_index as usize)) == eg_pcc_sorted.len() {
                diff_index = -1;
                break;
            }

            let hhe_parts_diff = eg_diff_sorted[hhe_index + (diff_index as usize)].0.split("::").collect::<Vec<&str>>();
            let (hhe_start_diff, hhe_end_diff) = (hhe_parts_diff[0].to_string(), hhe_parts_diff[1].to_string());

            // if hhe_start_diff.contains("ts-order-service") && hhe_end_diff.contains("ts-order-service") {
            // if hhe_start_diff.contains("ts-route-service") || hhe_end_diff.contains("ts-route-service") {
            // if hhe_start_diff.contains("ts-price-service") || hhe_end_diff.contains("ts-price-service") {
            if hhe_start_diff.contains("ts-station-service") || hhe_end_diff.contains("ts-station-service") {
                break;
            }

            diff_index += 1;
        }

        loop {
            if (vs_index as usize) == eg_vs_sorted.len() {
                vs_index = -1;
                break;
            }

            let hhe_parts_vs = eg_vs_sorted[vs_index as usize].0.split("::").collect::<Vec<&str>>();
            let (hhe_start_vs, hhe_end_vs) = (hhe_parts_vs[0].to_string(), hhe_parts_vs[1].to_string());

            // if hhe_start_vs.contains("ts-order-service") && hhe_end_vs.contains("ts-order-service") {
            // if hhe_start_vs.contains("ts-route-service") || hhe_end_vs.contains("ts-route-service") {
            // if hhe_start_vs.contains("ts-price-service") || hhe_end_vs.contains("ts-price-service") {
            if hhe_start_vs.contains("ts-station-service") || hhe_end_vs.contains("ts-station-service") {
                break;
            }

            vs_index += 1;
        }

        println!();
        println!();
        println!("Result positions:");
        println!("PCC --- {}", pcc_index);
        println!("Covariance --- {}", cov_index);
        println!("P99 - P50 --- {}", diff_index);
        println!("V/S Mean Diff --- {}", vs_index);
        println!();
        println!();

        // "all_overlaps" maps from critical path hashes to vectors of edges
        // that overlap with the corresponding CP's HHE
        let mut all_overlaps: HashMap<String, Vec<(String, String)>> = HashMap::new();

        // "all_hhe_crits" contains the CPs belonging to HHE-containing traces
        let mut all_hhe_crits: Vec<CriticalPath> = Vec::new();
        let mut cp_hhe_lats: HashMap<String, u64> = HashMap::new();

        for cp in &pt_crits {
            let cp_edge = cp.get_by_tracepoints(
                TracepointID::from_str(hhe_start_vs.as_str()), TracepointID::from_str(hhe_end_vs.as_str())
                // TracepointID::from_str(hhe_start_diff.as_str()), TracepointID::from_str(hhe_end_diff.as_str())
                // TracepointID::from_str(hhe_start_pcc.as_str()), TracepointID::from_str(hhe_end_pcc.as_str())
                // TracepointID::from_str(hhe_start_cov.as_str()), TracepointID::from_str(hhe_end_cov.as_str())
            );

            match cp_edge {
                Some(cpe) => {
                    all_hhe_crits.push(cp.clone());

                    let (ts, te, edge) = cpe;

                    let overlaps = reader.get_candidate_events(
                        ts.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64,
                        te.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64,
                        edge.host.unwrap()
                    );

                    // println!("CP Hash = {}", cp.hash());
                    // println!("TEnd = {}", te.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64);
                    // println!("TStart = {}", ts.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64);

                    // cp_hhe_lats.insert(
                    //     cp.hash().to_string(),
                    //     te.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64 - ts.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64
                    // );

                    // If difference between start and end is somehow negative, make it zero
                    let te_time = te.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64;
                    let ts_time = ts.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64;
                    if ts_time > te_time {
                        cp_hhe_lats.insert(cp.hash().to_string(), 0);
                    } else {
                        cp_hhe_lats.insert(
                            cp.hash().to_string(),
                            te_time - ts_time
                        );
                    }

                    // TODO: Re-enable this to gain more info
                    println!("Num Overlaps = {}", overlaps.len());

                    // println!("OVERLAPPING EDGES:");
                    for o in overlaps {
                        // println!();
                        // println!();
                        // println!("Getting overlaps for: [\n{:?}\n]", cp.get_by_tracepoints(
                        //     ts.tracepoint_id, te.tracepoint_id));
                        // println!();
                        // println!("{:?}", o.0.as_str());
                        // println!(
                        //     "{:?}",
                        //     non_problem_traces.get(o.0.as_str()).unwrap()
                        //         .spans.get(o.1.as_str()).unwrap()
                        // );
                        // println!();
                        // println!();
                        match all_overlaps.get_mut(cp.hash()) {
                            Some(v) => {
                                v.push(o.clone());
                            }
                            None => {
                                all_overlaps.insert(cp.hash().to_string(), vec![o.clone()]);
                            }
                        }
                    }
                }
                None => continue
            }
        }

        println!();
        println!();
        for (cph, ov) in all_overlaps.clone() {
            println!("CPH = {}   |||   # OL = {}", cph, ov.len());
        }
        println!();
        println!();

        all_hhe_crits.sort_by(|a, b| {
            a.duration.partial_cmp(&b.duration).unwrap()
        });

        let victim_crits: Vec<CriticalPath> = all_hhe_crits
            .drain((((all_hhe_crits.len() as f64)*0.9) as usize)..).collect();
        let survivor_crits: Vec<CriticalPath> = all_hhe_crits.drain(..).collect();

        let victim_hashes = victim_crits.into_iter()
            .map(|cp| cp.hash().to_string()).collect::<Vec<String>>();
        let survivor_hashes = survivor_crits.into_iter()
            .map(|cp| cp.hash().to_string()).collect::<Vec<String>>();

        // // "backtraces" maps from trace ID to a set of backtraces
        // let mut backtraces: HashMap<String, Vec<Vec<Span>>> = HashMap::new();
        // // "k" is a CP hash; "v" is a vector of (traceID, spanID) pairs
        // for (k, v) in all_overlaps {
        //     for o in v {
        //         // Get problem trace based on overlap's trace ID
        //         let overlapping_trace = non_problem_traces
        //             .get(o.0.as_str()).unwrap();
        // 
        //         //** If there is a backtrace list corresponding to "k", add a backtrace
        //         //** to it, otherwise create the list
        //         // match backtraces.get_mut(o.0.as_str()) {
        //         match backtraces.get_mut(k.as_str()) {
        //             Some(v) => {
        //                 v.push(overlapping_trace.get_backtrace(o.1))
        //             }
        //             None => {
        //                 backtraces.insert(k.clone(), vec![overlapping_trace.get_backtrace(o.1)]);
        //             }
        //         }
        //     }
        // }
        // 
        // let mut backtrace_features = HashSet::new();
        // for (_, v) in backtraces.clone().into_iter() {
        //     for backtrace in v {
        //         for span in backtrace {
        //             let features = span.get_features();
        //             for feature in features {
        //                 backtrace_features.insert(feature);
        //             }
        //         }
        //     }
        // }

        // "backtraces" maps from trace ID to a set of backtraces
        // let mut backtraces: HashMap<String, Vec<Vec<Span>>> = HashMap::new();
        let mut backtraces: HashMap<String, Vec<Vec<Feature2>>> = HashMap::new();
        let mut backtrace_features = HashSet::new();
        // "k" is a CP hash; "v" is a vector of (traceID, spanID) pairs
        for (k, v) in all_overlaps {
            for o in v {
                // Get problem trace based on overlap's trace ID
                let overlapping_trace = non_problem_traces
                    .get(o.0.as_str()).unwrap();

                //** If there is a backtrace list corresponding to "k", add a backtrace
                //** to it, otherwise create the list
                // match backtraces.get_mut(o.0.as_str()) {
                
                let ot_bf2 = overlapping_trace.backtrace_features_2(o.1);
                
                match backtraces.get_mut(k.as_str()) {
                    Some(v) => {
                        v.push(ot_bf2.clone())
                    }
                    None => {
                        backtraces.insert(k.clone(), vec![ot_bf2.clone()]);
                    }
                }
                
                for bf in &ot_bf2 {
                    backtrace_features.insert(bf.clone());
                }
            }
        }

        println!();
        println!();
        for (cph, bv) in backtraces.clone() {
            println!("CPH = {}   |||   # Backtraces = {}", cph, bv.len());
        }
        println!();
        println!();
        
        // for (_, v) in backtraces.clone().into_iter() {
        //     for backtrace in v {
        //         for span in backtrace {
        //             let features = span.get_features();
        //             for feature in features {
        //                 backtrace_features.insert(feature);
        //             }
        //         }
        //     }
        // }

        println!();
        println!();
        println!("# Backtrace Features = {}", backtrace_features.len());
        println!();
        for feature in backtrace_features.clone().into_iter() {
            println!("FEATURE:\n{:?}", feature);
        }
        println!();
        println!();

        // let mut feature_occupancy_dists: HashMap<Feature, (Vec<u64>, Vec<u64>)> = HashMap::new();
        let mut feature_occupancy_dists: HashMap<Feature2, (Vec<u64>, Vec<u64>)> = HashMap::new();
        // let mut feature_correlations: HashMap<Feature, (Vec<u64>, Vec<u64>)> = HashMap::new();
        let mut feature_correlations: HashMap<Feature2, (Vec<u64>, Vec<u64>)> = HashMap::new();

        let total_problem_count = victim_hashes.len() + survivor_hashes.len();

        for feature in &backtrace_features {
            println!();
            println!("Starting analysis for:\n{:?}", feature);
            println!();
            let mut victim_occupancy_counts = Vec::new();
            let mut survivor_occupancy_counts = Vec::new();
            let mut hhe_latencies = Vec::new();

            let mut counter = 1;
            for vh in &victim_hashes {
                let mut occurrences = 0u64;

                // match backtraces.get(vh.as_str()) {
                //     Some(bts) => {
                //         for backtrace in bts {
                //             for span in backtrace {
                //                 if span.has_feature(feature.clone()) {
                //                     occurrences += 1;
                //                     break;
                //                 }
                //             }
                //         }
                //     },
                //     _ => {}
                // }

                match backtraces.get(vh.as_str()) {
                    Some(bts) => {
                        for backtrace in bts {
                            if backtrace.contains(feature) {
                                occurrences += 1;
                                break;
                            }
                        }
                    },
                    _ => {}
                }

                println!("{} of {}   (V)   |||    {} occurrences", counter, total_problem_count, occurrences);
                
                victim_occupancy_counts.push(occurrences);
                hhe_latencies.push(cp_hhe_lats.get(vh.as_str()).unwrap().clone());

                counter += 1;
            }

            for sh in &survivor_hashes {
                let mut occurrences = 0u64;

                // match backtraces.get(sh.as_str()) {
                //     Some(bts) => {
                //         for backtrace in bts {
                //             for span in backtrace {
                //                 if span.has_feature(feature.clone()) {
                //                     occurrences += 1;
                //                     break;
                //                 }
                //             }
                //         }
                //     },
                //     _ => {}
                // }

                match backtraces.get(sh.as_str()) {
                    Some(bts) => {
                        for backtrace in bts {
                            if backtrace.contains(feature) {
                                occurrences += 1;
                                break;
                            }
                        }
                    },
                    _ => {}
                }

                println!("{} of {}   (S)   |||    {} occurrences", counter, total_problem_count, occurrences);
                
                survivor_occupancy_counts.push(occurrences);
                hhe_latencies.push(cp_hhe_lats.get(sh.as_str()).unwrap().clone());

                counter += 1;
            }
            
            feature_occupancy_dists.insert(
                feature.clone(), (victim_occupancy_counts.clone(), survivor_occupancy_counts.clone()));

            let mut all_occupancies = victim_occupancy_counts;
            all_occupancies.append(&mut survivor_occupancy_counts);

            feature_correlations.insert(
                feature.clone(), (all_occupancies, hhe_latencies));
        }

        println!();
        println!();
        println!();
        println!();
        println!();
        for feature in &backtrace_features {
            println!("Feature:\n{:?}", feature);
            println!();
            let dists = feature_occupancy_dists.get(feature).unwrap();
            println!("Victim Occupancies:\n{:?}", dists.0);
            println!();
            println!("Survivor Occupancies:\n{:?}", dists.1);
            println!();
            println!();
            println!();
        }
        println!();
        println!();
        println!();
        println!();
        println!();

        let mut results = Vec::new();

        for feature in &backtrace_features {
            let dists = feature_occupancy_dists.get(feature).unwrap();
            let victim_mean = mean(dists.0.clone().into_iter());
            let survivor_mean = mean(dists.1.clone().into_iter());
            results.push((feature.clone(), victim_mean - survivor_mean));
        }

        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap()
        });

        println!();
        println!();
        println!();
        println!();
        println!("RESULTS (MEAN DIFF):");
        println!("++++++++++++++++++++++++++++++++++++++++");
        for result in results {
            println!("{:?} === {}", result.0, result.1);
        }
        println!("++++++++++++++++++++++++++++++++++++++++");
        println!();
        println!();
        println!();
        println!();

        results = Vec::new();

        for feature in &backtrace_features {
            let dists = feature_occupancy_dists.get(feature).unwrap();
            let victim_mean = mean(dists.0.clone().into_iter());
            results.push((feature.clone(), victim_mean));
        }

        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap()
        });

        println!();
        println!();
        println!();
        println!();
        println!("RESULTS (VICTIM MEAN):");
        println!("++++++++++++++++++++++++++++++++++++++++");
        for result in results {
            println!("{:?} === {}", result.0, result.1);
        }
        println!("++++++++++++++++++++++++++++++++++++++++");
        println!();
        println!();
        println!();
        println!();

        results = Vec::new();
        for feature in &backtrace_features {
            let dists = feature_correlations.get(feature).unwrap();

            let mut pcc_num = 0i128;
            // for (_, ed, rt) in &dists.1 {
            let mut i = 0;
            let lat_mean = mean(dists.1.clone().into_iter());
            let occ_mean = mean(dists.0.clone().into_iter());
            loop {
                if i == dists.1.len() {
                    break;
                }

                let lat = dists.1[i];
                let occ = dists.0[i];

                pcc_num += (lat as i128 - lat_mean as i128)*(occ as i128 - occ_mean as i128);

                i += 1;
            }

            let pcc = (pcc_num as f64)/
                ((dists.1.len() as f64)*
                    (stddev(dists.1.clone().into_iter())*
                        stddev(dists.0.clone().into_iter())));

            results.push((feature.clone(), pcc));
        }

        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap()
        });

        println!();
        println!();
        println!();
        println!();
        println!();
        println!("RESULTS (CORRELATION):");
        println!("++++++++++++++++++++++++++++++++++++++++");
        for result in results {
            println!("{:?} === {}", result.0, result.1);
        }
        println!("++++++++++++++++++++++++++++++++++++++++");
        println!();
        println!();
        println!();
        println!();

        results = Vec::new();
        for feature in &backtrace_features {
            let dists = feature_correlations.get(feature).unwrap();

            let mut pcc_num = 0i128;
            // for (_, ed, rt) in &dists.1 {
            let mut i = 0;
            let lat_mean = mean(dists.1.clone().into_iter());
            let occ_mean = mean(dists.0.clone().into_iter());
            loop {
                if i == dists.1.len() {
                    break;
                }

                let lat = dists.1[i];
                let occ = dists.0[i];

                pcc_num += (lat as i128 - lat_mean as i128)*(occ as i128 - occ_mean as i128);

                i += 1;
            }

            let cov = (pcc_num as f64)/
                ((dists.1.len() as f64));

            results.push((feature.clone(), cov));
        }

        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap()
        });

        println!();
        println!();
        println!();
        println!();
        println!();
        println!("RESULTS (COVARIANCE):");
        println!("++++++++++++++++++++++++++++++++++++++++");
        for result in results {
            println!("{:?} === {}", result.0, result.1);
        }
        println!("++++++++++++++++++++++++++++++++++++++++");
        println!();
        println!();
        println!();
        println!();

        // for cp in &pt_crits {
        //     match top_problem_edges.get(cp.hash()) {
        //         Some ((tns, tne)) => {
        //             let (ts, te, edge) = cp.get_by_tracepoints(
        //                 tns.tracepoint_id, tne.tracepoint_id);
        //
        //             let overlaps = reader.get_candidate_events(
        //                 ts.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64,
        //                 te.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64,
        //                 edge.host.unwrap()
        //             );
        //
        //             println!("OVERLAPPING EDGES:");
        //             for o in overlaps {
        //                 println!();
        //                 println!();
        //                 println!("Getting overlaps for: [\n{:?}\n]", cp.get_by_tracepoints(
        //                 tns.tracepoint_id, tne.tracepoint_id));
        //                 println!();
        //                 println!("{:?}", o.0.as_str());
        //                 println!(
        //                     "{:?}",
        //                     non_problem_traces.get(o.0.as_str()).unwrap()
        //                         .spans.get(o.1.as_str()).unwrap()
        //                 );
        //                 println!();
        //                 println!();
        //             }
        //         },
        //         None => continue
        //     }
        // }
        
        println!("END PHASE 2");
    } else {
        // Initialize search strategies and group management
        let now = Instant::now();
        // let strategy = get_strategy(&SETTINGS, &MANIFEST, &CONTROLLER);
        let mut budget_manager = BudgetManager::from_settings(&SETTINGS);
        // let mut groups = GroupManager::new();
        let mut last_decision = Instant::now();
        let mut last_gc = Instant::now();

        let mut quit_in = -1;
        let mut targets = HashSet::new();
        // The targets are set here. Any typos, and Pythia won't stop.
        targets.insert(TracepointID::from_str("nova/usr/local/lib/python3.6/dist-packages/nova/compute/manager.py:1859:nova.compute.manager.ComputeManager._update_scheduler_instance_info"));
        eprintln!("Targets are {:?}", targets);

        let victim_segment = (
            "nova/usr/local/lib/python3.6/dist-packages/nova/compute/manager.py:1972:_locked_do_build_and_run_instance:_build_semaphore._waiters",
            "nova/usr/local/lib/python3.6/dist-packages/nova/hooks.py:2046:nova.compute.manager.ComputeManager._do_build_and_run_instance"
        );

        let filename = std::env::args().nth(1).unwrap();
        eprintln!("Printing results to {}", filename);
        let mut output_file = File::create(filename).unwrap();
        writeln!(output_file, "{:?}", *SETTINGS).ok();
        writeln!(output_file, "Targets: {:?}", targets).ok();

        // Enable skeleton/minimal always-on tracepoints
        CONTROLLER.disable_all();
        let to_enable = MANIFEST
            .skeleton()
            .iter()
            .map(|a| {
                if !targets.get(a).is_none() {
                    targets.remove(a);
                    if targets.len() == 0 {
                        panic!("Targets are in the skeleton");
                    }
                    a
                } else {
                    a
                }
            })
            .map(|&a| (a.clone(), None))
            .collect();
        CONTROLLER.enable(&to_enable);
        writeln!(output_file, "Enabled {}", to_enable.len()).ok();
        writeln!(output_file, "Enabled {:?}", to_enable).ok();
        reset_reader();

        println!("Enabled following tracepoints: {:?}", to_enable);

        let pool = ThreadPool::new(SETTINGS.n_workers + 2);
        let (tx_in, rx_in) = channel();
        for _ in 0..SETTINGS.n_workers {
            let tx = tx_in.clone();
            // Asynchronously loop and continuously fetch recent traces, and then send them to "rx"
            // in order to be able to read later on in "Main pythia loop" section
            pool.execute(move || {
                let mut reader = reader_from_settings(&SETTINGS);
                loop {
                    // let recent_traces = match SETTINGS.application {
                    //     ApplicationType::Jaeger | ApplicationType::Zipkin => reader.get_recent_span_traces(),
                    //     _ => reader.get_recent_traces()
                    // };
                    // for trace in reader.get_recent_traces() {
                    //     println!("==========\nTrace:");
                    //     println!("{}", trace.base_id);
                    //     println!("{}", trace);
                    //     println!("{}", trace.request_type);
                    //     println!("\n==========\n\n\n");
                    //
                    //     tx.send(CriticalPath::from_trace(&trace).unwrap())
                    //         .expect("channel will be there waiting for the pool");
                    //
                    //     // tx.send(trace)
                    //     //     .expect("channel will be there waiting for the pool");
                    // }

                    match SETTINGS.application {
                        ApplicationType::Jaeger | ApplicationType::Zipkin => {
                            for trace in reader.get_recent_span_traces() {
                                tx.send(CriticalPath::from_trace(&trace.to_critical_path()).unwrap())
                                    .expect("channel will be there waiting for the pool");
                            }
                        }
                        _ => {
                            for trace in reader.get_recent_traces() {
                                tx.send(CriticalPath::from_trace(&trace).unwrap())
                                    .expect("channel will be there waiting for the pool");
                            }
                        }
                    }

                    sleep(SETTINGS.jiffy);
                }
            });
        }

        // let (tx_across, rx_across) = channel();

        let mut final_break = false;

        if true {
            // Main pythia loop
            // Loop infinitely, making tracepoint enabling decisions in each iteration
            let mut jiffy_no = 0;
            pool.execute(move || {
                let strategy = get_strategy(&SETTINGS, &MANIFEST, &CONTROLLER);
                let mut groups = GroupManager::new();
                let mut used_groups_archive : Vec<Group> = Vec::new();

                loop {
                    println!();
                    println!();
                    println!();
                    println!();
                    println!();
                    println!();
                    println!();
                    println!();
                    println!();
                    println!();
                    println!("NEW ITERATION!!");
                    println!();
                    println!();
                    println!();
                    println!();
                    writeln!(output_file, "Jiffy {}, {:?}", jiffy_no, Instant::now()).ok();
                    budget_manager.read_stats();
                    budget_manager.print_stats();
                    budget_manager.write_stats(&mut output_file);
                    let over_budget = budget_manager.overrun();

                    // Collect traces, add traces to groups
                    let critical_paths: Vec<CriticalPath> = rx_in.try_iter().collect::<Vec<CriticalPath>>().into_iter().filter(
                        | cp: &CriticalPath | cp.request_type == SETTINGS.problem_type
                    ).collect();

                    // // TODO: use critical_paths to get edge IDs of problematic edge types and send via tx_across
                    // for cp in critical_paths.iter() {
                    //     tx_across.send(/*some stuff in here to get the problematic edge*/"placeholder")
                    //         .expect("CACTI will be receiving on a channel");
                    // }

                    groups.update(&critical_paths);
                    budget_manager.update_new_paths(&critical_paths);
                    println!(
                        "Got {} paths of duration {:?} at time {}us",
                        critical_paths.len(),
                        critical_paths
                            .iter()
                            .map(|p| p.duration)
                            .collect::<Vec<Duration>>(),
                        now.elapsed().as_micros()
                    );
                    println!();
                    println!();
                    println!("Groups: {}", groups);
                    println!();
                    println!();
                    writeln!(output_file, "New traces: {}", critical_paths.len()).ok();
                    writeln!(
                        output_file,
                        "New tracepoints: {}",
                        critical_paths
                            .iter()
                            .map(|p| p.g.g.node_count())
                            .sum::<usize>()
                    )
                        .ok();

                    // if over_budget || last_gc.elapsed() > SETTINGS.gc_epoch {
                    // Run garbage collection
                    // if over_budget {
                    // eprintln!("Over budget, would disable but it's not implemented");
                    // let enabled_tracepoints: HashSet<_> =
                    //     CONTROLLER.enabled_tracepoints().drain(..).collect();
                    // let keep_count =
                    //     (enabled_tracepoints.len() as f32 * (1.0 - SETTINGS.disable_ratio)) as usize;
                    // let mut to_keep = HashSet::new();
                    // for g in groups.problem_groups() {
                    //     let mut nidx = g.start_node;
                    //     while nidx != g.end_node {
                    //         if enabled_tracepoints
                    //             .get(&(g.at(nidx), Some(g.request_type)))
                    //             .is_none()
                    //         {
                    //             eprintln!(
                    //                 "{} is not enabled for {} but we got it",
                    //                 g.at(nidx),
                    //                 g.request_type
                    //             );
                    //         } else {
                    //             to_keep.insert((g.at(nidx), Some(g.request_type)));
                    //             if to_keep.len() > keep_count {
                    //                 break;
                    //             }
                    //         }
                    //         nidx = g.next_node(nidx).unwrap();
                    //     }

                    //     if to_keep.len() > keep_count {
                    //         break;
                    //     }
                    // }
                    // let mut to_disable = Vec::new();
                    // for tp in enabled_tracepoints {
                    //     if to_keep.get(&tp).is_none() {
                    //         to_disable.push(tp);
                    //     }
                    // }
                    // CONTROLLER.disable(&to_disable);
                    // writeln!(output_file, "Disabled {}", to_disable.len()).ok();
                    // writeln!(output_file, "Disabled {:?}", to_disable).ok();
                    // }
                    // Disable tracepoints not observed in critical paths
                    //     let to_disable = budget_manager.old_tracepoints();
                    //     CONTROLLER.disable(&to_disable);
                    //     writeln!(output_file, "Disabled {}", to_disable.len()).ok();
                    //     writeln!(output_file, "Disabled {:?}", to_disable).ok();

                    //     last_gc = Instant::now();
                    // }

                    println!("BEFORE IF CHECK - {:?} - {:?}", last_decision.elapsed(), SETTINGS.decision_epoch);
                    // TODO: Ignoring budget for now
                    // if !over_budget && last_decision.elapsed() > SETTINGS.decision_epoch {
                    if last_decision.elapsed() > SETTINGS.decision_epoch {
                        println!("IF CHECK SUCCEEDED");

                        let enabled_tracepoints: HashSet<_> =
                            CONTROLLER.enabled_tracepoints().drain(..).collect();


                        // Make decision
                        let mut budget = SETTINGS.tracepoints_per_epoch;
                        // let problem_groups = groups.problem_groups();

                        // Extract problematic groups from the group manager based on a particular CV (0.05 here)
                        let problem_groups = groups.problem_groups_cv(0.05); // tsl: problem groups takes now
                        // let all_groups = groups.all_groups();
                        // println!("*CV Groups: {:?}", problem_groups);

                        //comment-in below line for consistently slow analysis
                        // let problem_groups_slow = groups.problem_groups_slow(95.0); // tsl: problem groups takes now
                        // println!("*SLOW Groups: {:?}", problem_groups_slow);

                        let mut used_groups = Vec::new();

                        //tsl ; get problematic group types to disable tps for non-problematic ones
                        let mut problematic_req_types = Vec::new();

                        // TODO: Why are we specifically taking 10 problematic groups?
                        println!("Making decision. Top 10 problem groups:");
                        for g in problem_groups.iter().take(10) {
                            println!("{}", g);
                            // for enabled in &g.enabled_tps{
                            //     println!("Enabled: {:?} ", enabled);
                            // }
                        }

                        //comment-in below line for consistently slow analysis
                        // println!("Making decision. Top 10 slow problem groups:");
                        // for g in problem_groups_slow.iter().take(10) {
                        //     println!("{}", g);
                        //     // for enabled in &g.enabled_tps{
                        //     //     println!("Enabled: {:?} ", enabled);
                        //     // }
                        // }

                        // Iterate through selected problematic groups and make decisions to
                        // enable/disable tracepoints based on search strategy
                        println!();
                        println!();
                        println!("Number of distinct problem groups: {}", problem_groups.len());
                        println!();
                        println!("Problem groups: {:?}", problem_groups);
                        println!();
                        println!();
                        println!("Problematic req types before: ");
                        println!("{:?}, ", problematic_req_types);
                        println!();
                        for g in problem_groups {
                            println!();
                            println!("Problem group iteration with type [{}]", g.request_type);
                            println!();
                            problematic_req_types.push(g.request_type.clone());

                            let problem_edges = g.problem_edges();

                            // Grab 10 top edges from group; TODO: why 10 specifically?
                            println!("Top 10 edges of group {}:", g);
                            for edge in problem_edges.iter().take(10) {
                                let endpoints = g.g.edge_endpoints(*edge).unwrap();
                                println!(
                                    "({} -> {}): {}",
                                    g.g[endpoints.0], g.g[endpoints.1], g.g[*edge]
                                );
                            }
                            // Iterate through edges, making decisions and enabling further
                            // tracepoints based on those decisions
                            let mut pgp = false;
                            for &edge in problem_edges.iter() {
                                // TODO: Ignoring budget for now
                                // if budget <= 0 {
                                //     break;
                                // }
                                let endpoints = g.g.edge_endpoints(edge).unwrap();
                                println!(
                                    "Searching ({} -> {}): {}",
                                    g.g[endpoints.0], g.g[endpoints.1], g.g[edge]
                                );
                                let decisions = strategy
                                    .search(g, edge, budget)
                                    .iter()
                                    .take(budget)
                                    .map(|&t| (t, Some(g.request_type.clone())))
                                    .collect::<Vec<_>>();
                                budget -= decisions.len();
                                for d in &decisions {
                                    if !targets.get(&d.0).is_none() {
                                        targets.remove(&d.0);
                                        if targets.len() == 0 {
                                            eprintln!("Found the target");
                                            quit_in = 20;
                                        } else {
                                            eprintln!("Found one target");
                                        }
                                    }
                                }
                                CONTROLLER.enable(&decisions);
                                writeln!(output_file, "Enabled {}", decisions.len()).ok();
                                writeln!(output_file, "Enabled {:?}", decisions).ok();
                                if decisions.len() > 0 {
                                    used_groups.push(g.hash().to_string());
                                    // let gv = *g;
                                    used_groups_archive.push(g.clone());
                                }
                                // // tsl: record enabled tracepoints per group
                                // g.update_enabled_tracepoints(&decisions);

                                if !pgp {
                                    let reeeee = g.g.edge_indices();
                                    // let haaaan = g.traces[0].g.g.edge_indices();

                                    let mut edge_durations_indices = Vec::new();

                                    for yeet in reeeee {
                                        let edge_duration = &g.g.edge_weight(yeet).unwrap().duration;
                                        let edge_endpoints = g.g.edge_endpoints(yeet.clone()).unwrap();
                                        edge_durations_indices.push((edge_duration, edge_endpoints))
                                    }

                                    edge_durations_indices.sort_by(| di1, di2 | -> Ordering {
                                        let max1 = max(di1.0.iter().map(|&x| x.as_nanos())).unwrap();
                                        let max2 = max(di2.0.iter().map(|&x| x.as_nanos())).unwrap();
                                        return max2.cmp(&max1);
                                    });

                                    // for yeet in reeeee {
                                    //     println!();
                                    //     println!("EDGE IS:");
                                    //     let edge_duration = &g.g.edge_weight(yeet).unwrap().duration;
                                    //     let edge_variance = variance(edge_duration.iter().map(|&x| x.as_nanos()));
                                    //     let edge_mean = mean(edge_duration.iter().map(|&x| x.as_nanos()));
                                    //     println!("{:?} ; {}", edge_duration, (edge_variance/edge_mean)/1000000000.0);
                                    //     // println!("{:?}", g.traces[0].g.g.edge_endpoints(yeet));
                                    //     println!();
                                    // }

                                    for di in edge_durations_indices {
                                        println!();
                                        println!("EDGE IS:");
                                        let edge_duration = di.0;
                                        let edge_variance = variance(edge_duration.iter().map(|&x| x.as_nanos()));
                                        let edge_variance = variance(edge_duration.iter().map(|&x| x.as_nanos()));
                                        let edge_mean = mean(edge_duration.iter().map(|&x| x.as_nanos()));
                                        println!("{:?} ; {}", edge_duration, (edge_variance/edge_mean)/1000000000.0);
                                        println!();
                                    }

                                    pgp = true;
                                }
                            }
                            // TODO: Ignoring budget for now
                            // if budget <= 0 {
                            //     break;
                            // }
                        }
                        println!();
                        println!("Problematic req types after: ");
                        println!("{:?}, ", problematic_req_types);
                        println!();
                        // for item in problematic_req_types{
                        //     println!("{:?}, ", item)
                        // }

                        // TODO: Reactivate used groups at some point
                        // for g in used_groups {
                        //     groups.used(&g);
                        // }

                        if false {
                            println!();
                            println!();
                            println!("USED GROUPS:");
                            println!();
                            for ug in &used_groups_archive {
                                println!("{}", ug);
                                println!();
                            }
                            println!();
                            println!();
                        }

                        //tsl : for groups that stopped being problematic; just disable tracepoints, which are enabled so far

                        // let mut to_disable = Vec::new();
                        for tp in enabled_tracepoints {
                            println!("{:?}, ", tp.1);

                            // if g.request_type == tp. && to_keep.get(&tp).is_none() {
                            //     to_disable.push(tp);
                            // }

                        }
                        // CONTROLLER.disable(&to_disable);



                        last_decision = Instant::now();
                    }
                    else {
                        if false {
                            println!();
                            println!();
                            println!();
                            println!("ALL GROUPS:");
                            for group in groups.all_groups() {
                                println!();
                                println!();
                                println!("GROUP:");
                                let mut group_vec = Vec::new();
                                group_vec.push(group);
                                println!("{:?}", group_vec);

                                let reeeee = group.g.edge_indices();
                                // let haaaan = g.traces[0].g.g.edge_indices();
                                for yeet in reeeee {
                                    println!();
                                    println!("EDGE IS:");
                                    let edge_duration = &group.g.edge_weight(yeet).unwrap().duration;
                                    let edge_variance = variance(edge_duration.iter().map(|&x| x.as_nanos()));
                                    let edge_mean = mean(edge_duration.iter().map(|&x| x.as_nanos()));
                                    println!("{:?} ; {}", edge_duration, (edge_variance/edge_mean)/1000000000.0);
                                    // println!("{:?}", g.traces[0].g.g.edge_endpoints(yeet));
                                    println!();
                                }
                            }
                            println!();
                            println!();
                        }
                    }
                    quit_in -= 1;
                    if quit_in == 0 {
                        eprintln!("Quitting");
                        final_break = true;
                        return;
                    }

                    jiffy_no += 1;
                    sleep(SETTINGS.jiffy);
                }
            });
        }

        // pool.execute(move || {
        //     // let mut cached_traces = HashMap::new();
        //     // let mut mapped_victims = Vec::new();
        //     // let mut cur_victims = Vec::new();
        //     // let mut cur_edges = Vec::new();
        //     //
        //     // let latest_traces: Vec<Trace> = rx_in.try_iter().collect::<Vec<Trace>>().into_iter().collect();
        //     // let victim_traces: Vec<CriticalPath> = latest_traces.iter().map(| t | -> CriticalPath {
        //     //     return CriticalPath::from_trace(t).unwrap();
        //     // }).filter(
        //     //     | cp: &CriticalPath | cp.request_type == SETTINGS.problem_type
        //     // ).collect();
        //
        //     let mut candidates = CandidateManager::from_settings(
        //         &SETTINGS, victim_segment);
        //
        //     loop {
        //         let latest_traces: Vec<Trace> = rx_in.try_iter().collect::<Vec<Trace>>().into_iter().collect();
        //         candidates.add_traces(latest_traces);
        //         candidates.process_victims();
        //         candidates.process_non_victims();
        //
        //         candidates.find_candidates();
        //         candidates.flush_old_victims();
        //         candidates.flush_old_non_victims();
        //
        //         println!();
        //         println!();
        //         println!();
        //         println!("OVERLAPS START");
        //         println!();
        //         for overlap in &candidates.victim_overlaps {
        //             println!();
        //             println!("{:?}", (overlap.1).0);
        //             println!("[");
        //             for te in &(overlap.1).1 {
        //                 println!("{:?}", te);
        //             }
        //             println!("]");
        //             println!();
        //         }
        //         println!("OVERLAPS END");
        //         println!();
        //         println!();
        //
        //         sleep(SETTINGS.jiffy);
        //     }
        // });
        //
        let mut i = 0;
        loop {
            if final_break {
                break;
            }

            println!("i = ${}", i);
            sleep(Duration::from_millis(1000));

            i += 1;
        }

        // // Main CACTI Loop
        // pool.execute(move || {
        //     loop {
        //         // Collect non-victim traces from second channel
        //         // Requires us to set up another channel, (tx2, rx2)
        //         let non_victim_traces = rx2.try_iter().collect::<Vec<_>>();
        //
        //         // Collect victim edges
        //         let victim_edges = rx_across.try_iter().collect::<Vec<_>>();
        //
        //         // Update "non-victim groups" - TODO: implement this once design finalized
        //         nv_groups.update(non_victim_traces);
        //
        //         if !over_budget && last_decision.elapsed() > SETTINGS.decision_epoch {
        //             // Get non-victim enabled tracepoints
        //             // TODO: implement this for non-victim traces, on a per-request-type basis
        //             let nv_enabled_tracepoints: HashSet<_> =
        //                 CONTROLLER.nv_enabled_tracepoints().drain(..).collect();
        //
        //             // Get problematic edges and search non-victim groups for overlapping tracepoints
        //             for g in problem_groups {
        //                 let problem_edges = g.problem_edges();
        //
        //                 for &edge in problem_edges.iter() {
        //                     // let endpoints = g.g.edge_endpoints(edge).unwrap();
        //
        //                     let overlaps = cacti_strategy.search(edge)
        //                         .iter()
        //                         .map(|&t| (t, Some(g.request_type)))
        //                         .collect::<Vec<_>>();
        //
        //                     for &overlap in overlaps.iter() {
        //                         // TODO: group overlaps by same edge ID/other attributes
        //
        //                         // TODO: add information to groups
        //                     }
        //
        //                     // TODO: Iterate through groups and make decisions for group(s)
        //                 }
        //             }
        //         }
        //     }
        // });
    }
}
