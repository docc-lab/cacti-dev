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
use itertools::max;
use petgraph::csr::EdgeIndex;
use petgraph::data::DataMap;
use petgraph::visit::IntoEdges;
use pythia_common::{OSPRequestType, RequestType};
use stats::{mean, variance};

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
use pythia::spantrace::SpanTrace;
use pythia::trace::{DAGEdge, Event, IDType, Trace, TraceNode, TracepointID};

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

        println!();
        println!();
        println!("PHASE 1");
        println!();
        println!();
        let problem_traces = reader.get_recent_span_traces();

        let problem_path_traces = problem_traces.iter().map(
            |st| st.to_critical_path()).collect::<Vec<Trace>>();

        let problem_paths = problem_path_traces
            .iter().map(|ppt| {
            let mut cp = CriticalPath::from_trace(ppt).unwrap();
            cp.request_type = ppt.request_type.clone();
            cp.start_node = ppt.start_node;
            cp.end_node = ppt.end_node;
            cp
        })
            .collect::<Vec<CriticalPath>>();

        println!("# problem traces = {}", problem_paths.len());

        // let groups = Group::from_critical_paths(problem_paths.clone()).into_iter()
        //     .filter(|g| g.traces.len() > 1).collect::<Vec<Group>>();

        let mut group_manager = GroupManager::new();
        group_manager.update(&problem_paths);

        // let problem_groups = group_manager.problem_groups_cv(0.05);

        // TODO: Make the # of groups selected a configurable parameter
        let problem_groups = group_manager.problem_groups_cv(0.05);
        let top_problem_groups = problem_groups[..std::cmp::min(10, problem_groups.len())]
            .into_iter().map(|&g| g.clone()).collect::<Vec<Group>>();

        let mut top_problem_edges: HashMap<String, (TraceNode, TraceNode)> = HashMap::new();
        for g in top_problem_groups {
            let ee = g.g.edge_endpoints(g.problem_edges()[0]).unwrap();
            let ee_start = g.g
                .node_weight(ee.0).unwrap().clone();
            let ee_end = g.g
                .node_weight(ee.1).unwrap().clone();

            top_problem_edges.insert(g.hash().to_string(), (ee_start, ee_end));
        }

        // sleep(Duration::from_micros(SETTINGS.cycle_lookback as u64));
        println!();
        println!();
        println!("PHASE 2");
        println!();
        println!();

        reader.set_fetch_all();

        let off_pl_traces = reader.get_recent_span_traces();
        
        println!("off_pl_traces.len() = {}", off_pl_traces.len());

        let mut problem_type_traces = Vec::new();
        // let mut non_problem_traces = Vec::new();
        let mut non_problem_traces = HashMap::new();

        for tr in off_pl_traces {
            // println!();
            // println!("INSERTING TRACE: [[{:?}]]", tr.clone());
            // println!();
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

        let pt_traces = problem_traces.iter().map(
            |st| st.to_critical_path()).collect::<Vec<Trace>>();

        let pt_crits = pt_traces
            .iter().map(|ppt| {
            let mut cp = CriticalPath::from_trace(ppt).unwrap();
            cp.request_type = ppt.request_type.clone();
            cp.start_node = ppt.start_node;
            cp.end_node = ppt.end_node;
            cp
        })
            .collect::<Vec<CriticalPath>>();

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

        for cp in pt_crits {
            match top_problem_edges.get(cp.hash()) {
                Some ((tns, tne)) => {
                    let (ts, te, edge) = cp.get_by_tracepoints(
                        tns.tracepoint_id, tne.tracepoint_id);

                    let overlaps = reader.get_candidate_events(
                        ts.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64,
                        te.timestamp.and_utc().timestamp_nanos_opt().unwrap() as u64,
                        edge.host.unwrap()
                    );

                    println!("OVERLAPPING EDGES:");
                    for o in overlaps {
                        println!();
                        println!();
                        println!("Getting overlaps for: [\n{:?}\n]", cp.get_by_tracepoints(
                        tns.tracepoint_id, tne.tracepoint_id));
                        println!();
                        println!(
                            "{:?}",
                            non_problem_traces.get(o.0.as_str()).unwrap()
                                .spans.get(o.1.as_str()).unwrap()
                        );
                        println!();
                        println!();
                    }
                },
                None => continue
            }
        }
        
        
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
