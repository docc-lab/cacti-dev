/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

//! Control plane messages for enabling/disabling tracepoints

use petgraph::visit::Control;
use crate::trace::TracepointID;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ControlMessage {
    pub enable: Vec<TracepointID>,
    pub disable: Vec<TracepointID>
}

impl ControlMessage {
    pub fn from_tracepoints(
        to_enable: Vec<TracepointID>,
        to_disable: Vec<TracepointID>
    ) -> ControlMessage {
        let mut to_return = ControlMessage {
            enable: to_enable,
            disable: to_disable
        };

        return to_return
    }
}