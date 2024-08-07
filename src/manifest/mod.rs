/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

//! This module has the searchspace.
//!
//! Manifest has one SearchSpace per request type, and mostly relays functions to the relevant
//! SearchSpace.
mod searchspace;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Display;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use petgraph::visit::IntoNodeReferences;
use petgraph::visit::NodeRef;
use serde::{Deserialize, Serialize};

use pythia_common::{OSPRequestType, RequestType};
use pythia_common::REQUEST_TYPE_REGEXES;

use crate::grouping::Group;
use crate::manifest::searchspace::SearchSpace;
use crate::trace::Trace;
use crate::trace::TracepointID;

pub use crate::manifest::searchspace::HierarchicalCriticalPath;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    // pub per_request_type: HashMap<OSPRequestType, SearchSpace>,
    pub per_request_type: HashMap<RequestType, SearchSpace>,
    pub request_type_tracepoints: Vec<TracepointID>,
}

impl Manifest {
    /// Returns the set of trace points seen for each request type
    // pub fn get_per_request_types(&self) -> HashMap<OSPRequestType, HashSet<TracepointID>> {
    pub fn get_per_request_types(&self) -> HashMap<RequestType, HashSet<TracepointID>> {
        let mut result = HashMap::new();
        for (rt, ss) in self.per_request_type.iter() {
            result.insert(rt.clone(), ss.trace_points());
        }
        result
    }

    pub fn all_tracepoints(&self) -> HashSet<TracepointID> {
        let mut result = HashSet::new();
        for ss in self.per_request_type.values() {
            result.extend(ss.trace_points().iter());
        }
        result
    }

    pub fn find_matches<'a>(&'a self, group: &Group) -> Vec<&'a HierarchicalCriticalPath> {
        let now = Instant::now();
        // let matches = if group.request_type == OSPRequestType::Unknown {
        let matches = if group.request_type == RequestType::Unknown {
            let mut result = Vec::new();
            for ss in self.per_request_type.values() {
                result.extend(ss.find_matches(group, false).iter());
            }
            result
        } else {
            match self.per_request_type.get(&group.request_type) {
                Some(ss) => ss.find_matches(group, false),
                None => {
                    panic!(
                        "Request type {:?} not present in manifest",
                        group.request_type
                    );
                }
            }
        };
        eprintln!(
            "Finding {} matching groups took {}, group size {}",
            matches.len(),
            now.elapsed().as_micros(),
            group.g.node_count()
        );
        if matches.len() == 0 {
            println!(
                "No critical path matches the group {}:\n{}",
                group,
                group.dot()
            );
        }
        matches
    }

    pub fn match_performance(&self, group: &Group) -> Duration {
        // Stats: base_id,trace_len,match_count,duration(us),best_match_len"
        let now = Instant::now();
        let matches = self
            .per_request_type
            .get(&group.request_type)
            .unwrap()
            .find_matches(group, true);
        if matches.len() == 0 {
            panic!(
                "Found no match for {}:\n{}",
                group.traces[0].g.base_id, group
            );
        }
        now.elapsed()
    }

    pub fn new() -> Manifest {
        Manifest {
            per_request_type: HashMap::new(),
            request_type_tracepoints: Vec::new(),
        }
    }

    pub fn from_trace_list(traces: &Vec<Trace>) -> Manifest {
        // let mut map = HashMap::<OSPRequestType, SearchSpace>::new();
        let mut map = HashMap::<RequestType, SearchSpace>::new();
        for trace in traces {
            match map.get_mut(&trace.request_type) {
                Some(space) => {
                    space.add_trace(&trace, false);
                }
                None => {
                    let mut space = SearchSpace::default();
                    space.add_trace(&trace, false);
                    map.insert(trace.request_type.clone(), space);
                }
            }
        }
        let mut result = Manifest {
            per_request_type: map,
            request_type_tracepoints: Vec::new(),
        };
        result.add_request_type_tracepoints(traces);
        result
    }

    fn add_request_type_tracepoints(&mut self, traces: &Vec<Trace>) {
        for trace in traces {
            self.request_type_tracepoints.extend(
                trace
                    .g
                    .node_references()
                    .map(|x| x.weight().tracepoint_id.to_string())
                    .filter(|x: &String| REQUEST_TYPE_REGEXES.is_match(x))
                    .map(|x| TracepointID::from_str(&x)),
            );
        }
    }

    pub fn to_file(&self, file: &Path) {
        let writer = std::fs::File::create(file).unwrap();
        serde_json::to_writer(writer, self).ok();
    }

    pub fn from_file(file: &Path) -> Option<Manifest> {
        let reader = std::fs::File::open(file).unwrap();
        serde_json::from_reader(reader).unwrap()
    }

    /// This is where a skeleton is defined. Adding/removing things to skeleton and
    /// changing the definition of a skeleton is done only from here.
    pub fn skeleton(&self) -> Vec<TracepointID> {
        let mut result = HashSet::new();
        for ss in self.per_request_type.values() {
            for tracepoint in ss.get_entry_points() {
                result.insert(tracepoint);
            }
            for tracepoint in ss.get_synchronization_points() {
                result.insert(tracepoint);
            }
            for tracepoint in ss.get_top_hierarchy() {
                result.insert(tracepoint);
            }
        }
        result.extend(self.request_type_tracepoints.iter());
        result.iter().cloned().collect()
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Manifest:").unwrap();
        for (request_type, inner) in &self.per_request_type {
            write!(f, "{:?} manifest:\n{}", request_type, inner).unwrap();
        }
        Ok(())
    }
}
