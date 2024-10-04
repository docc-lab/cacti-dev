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
use std::hash::Hash;
use std::ptr::null;
use std::time::Duration;

use chrono::{DateTime, NaiveDateTime};
use pythia_common::jaeger::JaegerRequestType;
use pythia_common::RequestType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{IDType, Trace};
use crate::trace::{DAGEdge, EdgeType, Event, EventType, TracepointID};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Feature {
    pub name: String,
    pub value: String,
}

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

    pub fn end(&self) -> i64 {
        // eprintln!("START = {}", self.start.and_utc().timestamp_nanos_opt().unwrap());
        // eprintln!("DURATION = {}", self.duration.as_nanos() as i64);
        return self.start.and_utc().timestamp_nanos_opt().unwrap() + (self.duration.as_nanos() as i64);
    }

    pub fn overlaps(&self, with: &Span) -> bool {
        let self_start = self.start.and_utc().timestamp_nanos_opt().unwrap();
        let self_end = self_start + (self.duration.as_nanos() as i64);

        let with_start = with.start.and_utc().timestamp_nanos_opt().unwrap();
        let with_end = with_start + (with.duration.as_nanos() as i64);

        return (with_start < self_end) && (with_end > self_start);
    }

    pub fn to_critical_path(
        &self, st: &SpanTrace,
        res: &mut Trace,
        par_host: String,
        par_serv: String
    ) {
        let mut sorted_children: Vec<Span> = st.children.get(self.span_id.as_str()).unwrap().clone();
        sorted_children.sort_by(
            |a, b| b.end().partial_cmp(&a.end()).unwrap());

        // let mut to_ret_graph: StableGraph<Event<String>, DAGEdge> = StableGraph::new();
        // let end_nidx = to_ret_graph.add_node(Event {
        //     trace_id: st.req_id.clone(),
        //     tracepoint_id: TracepointID::from_str((self.span_id.clone() + "_end").as_str()),
        //     timestamp: NaiveDateTime::from_timestamp(
        //         self.end()/1000,
        //         ((self.end()%1000)*1000) as u32
        //     ),
        //     variant: EventType::Entry,
        //     is_synthetic: false,
        //     key_value_pair: HashMap::new(),
        // });

        // let mut to_ret_trace: Trace<String> = Trace::new(st.req_id.clone());
        // to_ret_trace.g = to_ret_graph;
        // to_ret_trace.end_node = end_nidx;

        if res.g.node_count() == 0 {
            res.end_node = res.g.add_node(Event {
                // trace_id: IDType::STRING(st.req_id.clone()),
                // trace_id: IDType::STRING(self.span_id.clone()),
                trace_id: IDType::STRING(self.service.clone() + ":" + self.operation.as_str()),
                // tracepoint_id: TracepointID::from_str((self.span_id.clone() + "_end").as_str()),
                tracepoint_id: TracepointID::from_str((self.service.clone() + ":" + self.operation.as_str() + "_end").as_str()),
                timestamp: DateTime::from_timestamp_nanos(self.end()).naive_utc(),
                variant: EventType::Exit,
                is_synthetic: false,
                key_value_pair: HashMap::new(),
            });
            res.start_node = res.end_node.clone();
        } else {
            let connect_node = res.start_node.clone();
            res.start_node = res.g.add_node(
                Event {
                    // trace_id: IDType::STRING(st.req_id.clone()),
                    // trace_id: IDType::STRING(self.span_id.clone()),
                    trace_id: IDType::STRING(self.service.clone() + ":" + self.operation.as_str()),
                    // tracepoint_id: TracepointID::from_str((self.span_id.clone() + "_end").as_str()),
                    tracepoint_id: TracepointID::from_str((self.service.clone() + ":" + self.operation.as_str() + "_end").as_str()),
                    // timestamp: NaiveDateTime::from_timestamp(
                    //     self.end()/1000,
                    //     ((self.end()%1000)*1000) as u32
                    // ),
                    timestamp: DateTime::from_timestamp_nanos(self.end()).naive_utc(),
                    variant: EventType::Exit,
                    is_synthetic: false,
                    key_value_pair: HashMap::new(),
                }
            );
            let edge_duration = res.g.node_weight(connect_node).unwrap().timestamp.timestamp_nanos() - self.end();
            res.g.add_edge(
                res.start_node.clone(),
                connect_node.clone(),
                DAGEdge {
                    duration: Duration::from_nanos(edge_duration as u64),
                    variant: EdgeType::ChildOf,
                    host: Some(par_host),
                    service: Some(par_serv)
                }
            );
        }

        if sorted_children.len() > 0 {
            let mut cur_span: Span = Span::from_data(
                "".to_string(), "".to_string(),
                "".to_string(), "".to_string(),
                DateTime::from_timestamp_nanos(
                    sorted_children[0].end() + 1).naive_utc());
            for c in sorted_children.into_iter() {
                if c.end() < cur_span.start.and_utc().timestamp_nanos_opt().unwrap() {
                    cur_span = c.clone();
                    // let child_trace = cur_span.to_critical_path(st);
                    cur_span.to_critical_path(
                        st, res, self.host.clone(), self.service.clone());
                }
            }
        }

        let connect_node = res.start_node.clone();
        res.start_node = res.g.add_node(
            Event {
                // trace_id: IDType::STRING(st.req_id.clone()),
                // trace_id: IDType::STRING(self.span_id.clone()),
                trace_id: IDType::STRING(self.service.clone() + ":" + self.operation.as_str()),
                // tracepoint_id: TracepointID::from_str((self.span_id.clone() + "_end").as_str()),
                tracepoint_id: TracepointID::from_str((self.service.clone() + ":" + self.operation.as_str() + "_start").as_str()),
                timestamp: self.start,
                variant: EventType::Entry,
                is_synthetic: false,
                key_value_pair: HashMap::new(),
            }
        );
        let edge_duration = res.g.node_weight(connect_node).unwrap().timestamp.timestamp_nanos() - self.start.timestamp_nanos();
        res.g.add_edge(
            res.start_node.clone(),
            connect_node.clone(),
            DAGEdge {
                duration: Duration::from_nanos(edge_duration as u64),
                variant: EdgeType::ChildOf,
                host: Some(self.host.clone()),
                service: Some(self.service.clone())
            }
        );
    }
    
    pub fn get_features(&self) -> Vec<Feature> {
        let mut to_return = Vec::new();
        
        to_return.push(Feature{
            name: "service".to_string(),
            value: self.service.clone(),
        });
        to_return.push(Feature{
            name: "endpoint".to_string(),
            value: self.operation.clone(),
        });
        
        to_return
    }
}

// pub struct SpanTrace {
//     pub endpoint_type: String, // maybe change this to a special RequestType implementation later
//     pub root_span: Span,
//     pub req_id: String,
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpanTrace {
    pub endpoint_type: String, // maybe change this to a special RequestType implementation later
    pub req_id: String,
    pub root_span_id: String,
    pub spans: HashMap<String, Span>,
    pub children: HashMap<String, Vec<Span>>,
}

impl SpanTrace {
    pub fn from_span_list(
        spans: Vec<Span>,
        // parents: HashMap<String, String>,
        oper_name: String,
        root_span_id: String,
        trace_id: String
    ) -> SpanTrace {
        let mut to_ret_trace = SpanTrace{
            endpoint_type: oper_name,
            req_id: trace_id,
            root_span_id,
            spans: HashMap::new(),
            children: HashMap::new()
        };

        // let span_parents: HashMap<String, String> = HashMap::new();
        // for span in spans {
        //
        // }

        for span in &spans {
            // to_ret_trace.add_span(span.clone(), parents.get(span.span_id.as_str()).unwrap().clone());
            to_ret_trace.add_span(span.clone(), span.parent.clone());
        }

        to_ret_trace
    }

    fn add_span(&mut self, to_add: Span, parent: String) {
        self.spans.insert(to_add.clone().span_id, to_add.clone());
        // If no parent entry present, insert one and begin populating
        if !parent.is_empty() {
            // match self.children.get_mut(parent.as_str()) {
            //     Some(v) => {
            //         v.push(to_add.clone())
            //     }
            //     None => {
            //         self.children.insert(parent, Vec::new());
            //     }
            // }
            match self.children.get(parent.as_str()) {
                Some(v) => {
                    let mut new_v = v.clone();
                    new_v.push(to_add.clone());
                    self.children.remove(parent.as_str());
                    self.children.insert(parent, new_v);
                }
                None => {
                    self.children.insert(parent, vec![to_add.clone()]);
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

    pub fn to_critical_path(&self) -> Trace {
        let mut to_ret_trace = Trace::new(&IDType::STRING(self.req_id.clone()));
        // TODO: Change the attribute "endpoint_type" to a RequestType type later
        to_ret_trace.request_type = RequestType::Jaeger(JaegerRequestType{
            rt: self.endpoint_type.clone()
        });
        self.spans.get(self.root_span_id.as_str()).unwrap().to_critical_path(
            self, &mut to_ret_trace, "".to_string(), "".to_string());

        // if to_ret_trace.g.fre

        return to_ret_trace;
    }

    pub fn get_backtrace(&self, from: String) -> Vec<Span> {
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
    
    pub fn backtrace_features(&self, from: String) -> Vec<Feature> {
        self.get_backtrace(from).into_iter()
            .map(|s| s.get_features()).collect::<Vec<Vec<Feature>>>()
            .into_iter().flatten().collect()
    }
}

pub struct SpanCache {
    // pub span_refs: Vec<Span>,
    pub span_times: HashMap<String, Vec<(u64, u64)>>,
    pub span_refs: HashMap<String, Vec<(String, String)>>
}

impl SpanCache {
    pub fn init_cache() -> SpanCache {
        return SpanCache {
            // span_refs: Vec::new()
            span_times: HashMap::new(),
            span_refs: HashMap::new()
        };
    }

    pub fn add_trace(&mut self, to_add: SpanTrace) {
        for (_, span) in to_add.spans {
            self.add_span(span, to_add.req_id.clone());
        }
    }

    // "context" variable represents a trace ID
    pub fn add_span(&mut self, to_add: Span, trace_id: String) {
        let start_time = to_add.start.timestamp_nanos() as u64;
        let end_time = start_time + (to_add.duration.as_nanos() as u64);
        // self.span_times.push((start_time.clone(), end_time));
        match self.span_refs.get_mut(to_add.host.as_str()) {
            Some (v) => {
                v.push((trace_id, to_add.span_id));
                self.span_times.get_mut(to_add.host.as_str()).unwrap()
                    .push((start_time.clone(), end_time));
            },
            None => {
                self.span_refs.insert(to_add.host.clone(), vec![(trace_id, to_add.span_id)]);
                self.span_times.insert(to_add.host, vec![(start_time.clone(), end_time)]);
            }
        }
    }

    // pub fn add_spans(&mut self, to_add: Vec<Span>) {
    //     for &span in to_add.iter() {
    //         self.span_refs.push(span);
    //     }
    // }

    pub fn find_overlaps(&self, target: &Span) -> Vec<(String, String)> {
        let mut to_return = Vec::new();
        let mut i = 0;
        match self.span_times.get(target.host.as_str()) {
            Some(v) => {
                let refs = self.span_refs.get(target.host.as_str()).unwrap();
                for &time in v.iter() {
                    // if span.overlaps(target) {
                    //     to_return.push(span)
                    // }
                    let target_start = target.start.and_utc().timestamp_nanos_opt().unwrap() as u64;
                    let target_end = target_start + (target.duration.as_nanos() as u64);

                    if (target_end > time.0) && (target_start < time.1) {
                        to_return.push(refs[i.clone()].clone())
                    }
                    i += 1;
                }
                to_return
            },
            None => Vec::new()
        }
        // for &time in self.span_times.iter() {
        //     // if span.overlaps(target) {
        //     //     to_return.push(span)
        //     // }
        //     let target_start = target.start.timestamp_nanos();
        //     let target_end = target_start + (target.duration.as_nanos() as i64);
        //
        //     if (target_end > time.0) && (target_start < time.1) {
        //         to_return.push(self.span_refs[i.clone()].clone())
        //     }
        //     i += 1;
        // }
        // return to_return;
    }

    pub fn find_overlaps_raw(&self, target_start: u64, target_end: u64, host: String) -> Vec<(String, String)> {
        let mut to_return = Vec::new();
        let mut i = 0;
        match self.span_times.get(host.as_str()) {
            Some(v) => {
                let refs = self.span_refs.get(host.as_str()).unwrap();
                for &time in v.iter() {
                    if (target_end > time.0) && (target_start < time.1) {
                        to_return.push(refs[i.clone()].clone())
                    }
                    i += 1;
                }
                to_return
            },
            None => Vec::new()
        }
        // for &time in self.span_times.iter() {
        //     // if span.overlaps(target) {
        //     //     to_return.push(span)
        //     // }
        //     let target_start = target_start;
        //
        //     if (target_end > time.0) && (target_start < time.1) {
        //         to_return.push(self.span_refs[i.clone()].clone())
        //     }
        //     i += 1;
        // }
        // return to_return;
    }
}

