/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

//! Span-based trace implementation
//!

use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::ptr::null;
use std::time::Duration;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Span {
    pub span_id: String,
    pub parent: String,
    pub service: String, // Maybe make this a service ID type at some point
    pub host: String, // Maybe make this a special host type at some point
    pub operation: String,
    pub start: NaiveDateTime,
    pub duration: Duration,
    // pub children: Vec<*Span>
}

impl Span {
    pub fn from_data(
        sid: String,
        service: String,
        host: String,
        oper: String,
        start: NaiveDateTime
    ) -> Span {
        return Span {
            span_id: sid,
            parent: "".to_string(),
            service,
            host,
            operation: oper,
            start,
            duration: Default::default(),
            // children: Vec::new()
        }
    }

    pub fn add_parent(&mut self, parent: String) {
        self.parent = parent;
    }

    // pub fn add_child(&mut self, mut to_add: &Span) {
    //     self.children.push(to_add);
    //     to_add.add_parent(self);
    // }

    // pub fn add_child_ret(&mut self, mut to_add: &Span) -> &Span {
    //     self.children.push(to_add);
    //     to_add.add_parent(self);
    //     return to_add
    // }

    pub fn overlaps(&self, with: &Span) -> bool {
        let self_start = self.start.timestamp_nanos();
        let self_end = self_start + (self.duration.as_nanos() as i64);

        let with_start = with.start.timestamp_nanos();
        let with_end = with_start + (with.duration.as_nanos() as i64);

        return (with_start < self_end) && (with_end > self_start);
    }
}

// pub struct SpanTrace {
//     pub endpoint_type: String, // maybe change this to a special RequestType implementation later
//     pub root_span: Span,
//     pub req_id: String,
// }

pub struct SpanTrace {
    pub endpoint_type: String, // maybe change this to a special RequestType implementation later
    pub req_id: String,
    pub root_span_id: String,
    pub spans: HashMap<String, Span>,
    pub children: HashMap<String, Vec<Span>>,
}

impl SpanTrace {
    fn add_span(&mut self, to_add: Span, parent: String) {
        self.spans.insert(to_add.span_id, to_add.clone());
        // If no parent entry present, insert one and begin populating
        if !parent.is_empty() {
            match self.children.get_mut(parent.as_str()) {
                Some(mut v) => {
                    v.push(to_add.clone())
                }
                None => {
                    self.children.insert(parent, Vec::new());
                }
            }
        }
        // If self is not member of parent/child map, add entry
        match self.children.get(to_add.clone().span_id.as_str()) {
            None => {
                self.children.insert(to_add.clone().span_id, Vec::new());
            }
            _ => {}
        }
    }

    fn get_backtrace(&self, from: String) -> Vec<Span> {
        let mut to_return = Vec::new();
        let mut cur_id = from;
        loop {
            if cur_id.is_empty() {
                break;
            }

            let cur_span = self.spans.get(cur_id.as_str()).unwrap();
            let cur_parent = cur_span.parent.clone();
            to_return.push(cur_span.clone());
            cur_id = cur_parent;
        }
        return to_return;
    }
}

pub struct SpanCache {
    // pub span_refs: Vec<Span>,
    pub span_times: Vec<(i64, i64)>,
    pub span_refs: Vec<(String, String)>
}

impl SpanCache {
    pub fn init_cache() -> SpanCache {
        return SpanCache {
            // span_refs: Vec::new()
            span_times: Vec::new(),
            span_refs: Vec::new()
        };
    }

    // "context" variable represents a trace ID
    pub fn add_span(&mut self, to_add: Span, context: String) {
        let start_time = to_add.start.timestamp_nanos();
        let end_time = start_time + (to_add.duration.as_nanos() as i64);
        self.span_times.push((start_time.clone(), end_time));
        self.span_refs.push((context, to_add.span_id));
    }

    // pub fn add_spans(&mut self, to_add: Vec<Span>) {
    //     for &span in to_add.iter() {
    //         self.span_refs.push(span);
    //     }
    // }

    pub fn find_overlaps(&self, target: &Span) -> Vec<(String, String)> {
        let mut to_return = Vec::new();
        let mut i = 0;
        for &time in self.span_times.iter() {
            // if span.overlaps(target) {
            //     to_return.push(span)
            // }
            let target_start = target.start.timestamp_nanos();
            let target_end = target_start + (target.duration.as_nanos() as i64);

            if (target_end > time.0) && (target_start < time.1) {
                to_return.push(self.span_refs[i.clone()].clone())
            }
            i += 1;
        }
        return to_return;
    }


}

