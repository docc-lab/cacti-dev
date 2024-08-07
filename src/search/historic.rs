/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

use std::collections::HashMap;
use std::collections::HashSet;

use petgraph::graph::EdgeIndex;
use rand::seq::IteratorRandom;

use pythia_common::{OSPRequestType, RequestType};

use crate::controller::Controller;
use crate::grouping::Group;
use crate::manifest::Manifest;
use crate::search::SearchStrategy;
use crate::settings::Settings;
use crate::trace::TracepointID;

/// This strategy returns a random selection of trace points to enable
pub struct HistoricSearch {
    controller: &'static Box<dyn Controller>,
    // per_request_types: HashMap<OSPRequestType, HashSet<TracepointID>>,
    per_request_types: HashMap<RequestType, HashSet<TracepointID>>,
}

impl SearchStrategy for HistoricSearch {
    fn search(&self, group: &Group, _edge: EdgeIndex, budget: usize) -> Vec<TracepointID> {
        let mut rng = rand::thread_rng();
        self.per_request_types
            .get(&group.request_type)
            .unwrap()
            .iter()
            .filter(|&tp| !self.controller.is_enabled(
                &(*tp, Some(group.request_type.clone()))))
            .cloned()
            .choose_multiple(&mut rng, budget)
    }
}

impl HistoricSearch {
    pub fn new(_s: &Settings, m: &'static Manifest, c: &'static Box<dyn Controller>) -> Self {
        HistoricSearch {
            controller: c,
            per_request_types: m.get_per_request_types(),
        }
    }
}
