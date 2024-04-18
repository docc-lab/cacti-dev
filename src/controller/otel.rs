/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use pythia_common::RequestType;
use crate::controller::Controller;
use crate::Settings;
use crate::trace::TracepointID;

pub struct OTelController {
    client_list: Vec<String>,

    /// This should only be valid after disable_all is called
    enabled_tracepoints: Arc<Mutex<HashSet<(String, Option<String>)>>>,
    // TODO: ^^ Change strings to appropriate types
    // enabled_tracepoints: Arc<Mutex<HashSet<(TracepointID, Option<RequestType>)>>>,
}

impl Controller for OTelController {
    fn enable(&self, points: &Vec<(TracepointID, Option<RequestType>)>) {
        todo!()
    }

    fn disable(&self, points: &Vec<(TracepointID, Option<RequestType>)>) {
        todo!()
    }

    fn is_enabled(&self, point: &(TracepointID, Option<RequestType>)) -> bool {
        todo!()
    }

    fn disable_all(&self) {
        todo!()
    }

    fn enable_all(&self) {
        todo!()
    }

    fn enabled_tracepoints(&self) -> Vec<(TracepointID, Option<RequestType>)> {
        todo!()
    }
}

impl OTelController {
    pub fn from_settings(settings: &Settings) -> OTelController {
        return OTelController {
            client_list: settings.pythia_clients.clone(),
            enabled_tracepoints: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}