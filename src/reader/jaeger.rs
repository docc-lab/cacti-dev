/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::slice::SplitN;
use std::time::Duration;
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use futures::Sink;
use hyper::http;
use itertools::Itertools;
use crate::reader::Reader;
use crate::{Settings, Trace};
use crate::spantrace::{Span, SpanTrace};
use serde::{Serialize, Deserialize};
use crate::trace::Event;

#[derive(Debug, Serialize, Deserialize)]
struct JaegerReference {
    refType: String,
    traceID: String,
    spanID: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerSpan {
    traceID: String,
    spanID: String,
    flags: i32,
    operationName: String,
    startTime: i64,
    duration: i64,
    references: Vec<JaegerReference>,
    processID: String,
}

impl JaegerSpan {
    pub fn to_span(&self, processes: &HashMap<String, JaegerProcess>) -> Span {
        let process_tags = &processes.get(self.processID.as_str()).unwrap().tags;
        let mut host_name = "".to_string();
        for tag in process_tags {
            if tag.key == "hostname".to_string() {
                host_name = tag.value.clone();
            }
        }
        return Span{
            span_id: self.spanID.clone(),
            parent: match self.references.len() {
                0 => "".to_string(),
                _ => self.references[0].spanID.clone()
            },
            service: processes.get(self.processID.as_str()).unwrap().serviceName.clone(),
            host: host_name,
            operation: self.operationName.clone(),
            start: DateTime::from_timestamp_nanos(
                self.startTime*1000).naive_utc(),
            duration: Duration::from_micros(self.duration as u64)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JPTag {
    key: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerProcess {
    serviceName: String,
    tags: Vec<JPTag>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerTrace {
    traceID: String,
    spans: Vec<JaegerSpan>,
    processes: HashMap<String, JaegerProcess>
}

impl JaegerTrace {
    pub fn to_trace(&self) -> SpanTrace {
        let spans: Vec<Span> = self.spans.iter().map(|span| span.to_span(&self.processes)).collect();
        let root_span: &JaegerSpan = self.spans.iter().filter(|&span| span.traceID == span.spanID).collect::<Vec<_>>()[0];
        // let mut span_parents: HashMap<String, String> = HashMap::new();
        // for span in &self.spans {
        //     span_parents.insert(span.spanID, span.)
        // }

        // let mut to_ret_spans: HashMap<String, Span> = HashMap::new();
        // for span in &spans {
        //     to_ret_spans.insert(span.span_id.clone(), span);
        // }

        // return SpanTrace{
        //     endpoint_type: root_span.operationName.clone(),
        //     req_id: self.traceID.clone(),
        //     root_span_id: root_span.spanID.clone(),
        //     spans: to_ret_spans,
        //     children: Default::default()
        // }
        return SpanTrace::from_span_list(
            spans, root_span.operationName.clone(),
            root_span.spanID.clone(), self.traceID.clone());
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerPayload {
    data: Vec<JaegerTrace>,
}

pub struct JaegerReader {
    // connection: JaegerConnection // TODO: implement this
    fetch_url: String
}

impl Reader for JaegerReader {
    fn read_file(&mut self, filename: &str) -> Trace {
        todo!()
    }

    fn read_dir(&mut self, foldername: &str) -> Vec<Trace> {
        todo!()
    }

    fn get_trace_from_base_id(&mut self, id: &str) -> Result<Trace, Box<dyn Error>> {
        // eprintln!("Working on {}", id);
        // let mut result = match Uuid::parse_str(id) {
        //     Ok(uuid) => {
        //         let event_list = self.get_all_matches(&uuid);
        //         if event_list.len() == 0 {
        //             return Err(Box::new(PythiaError(
        //                 format!("No traces match the uuid {}", uuid).into(),
        //             )));
        //         }
        //         let dag = self.from_event_list(Uuid::parse_str(id).unwrap(), event_list)?;
        //         dag
        //     }
        //     Err(_) => {
        //         panic!("Malformed UUID received as base ID: {}", id);
        //     }
        // };
        // if result.request_type == RequestType::Unknown {
        //     eprintln!("Warning: couldn't get type for request {}", id);
        // }
        // result.duration = (result.g[result.end_node].timestamp
        //     - result.g[result.start_node].timestamp)
        //     .to_std()
        //     .unwrap();
        // Ok(result)
        todo!()
    }

    // #[tokio:main]
    // fn get_recent_traces(&mut self) -> Vec<Trace> {
    //     // let mut ids = Vec::new();
    //
    //     let mut traces: HashMap<String, Vec<Span>> = HashMap::new();
    //
    //     // let resp: reqwest::blocking::Response = reqwest::blocking::get("https://httpbin.org/ip").unwrap();
    //     let resp: reqwest::blocking::Response =
    //         reqwest::blocking::get(self.fetch_url.clone() + "/api/traces?service=nginx-web-server&limit=10")
    //             .unwrap();
    //
    //     // match resp.text() {
    //     //     Ok(res) => {
    //     //         eprintln!("RESPONSE = {:?}", resp.text());
    //     //     }
    //     // }
    //
    //     let resp_text = resp.text();
    //
    //     let resp_obj: JaegerPayload =
    //         serde_json::from_str(
    //             (resp_text.unwrap() as String).as_str()).unwrap();
    //
    //     eprintln!("RESPONSE = {:?}", resp_obj);
    //
    //     return Vec::new();
    // }
    fn get_recent_traces(&mut self) -> Vec<Trace> {
        // let mut ids = Vec::new();

        let mut traces: HashMap<String, Vec<Span>> = HashMap::new();

        // let resp: reqwest::blocking::Response = reqwest::blocking::get("https://httpbin.org/ip").unwrap();
        // let resp: reqwest::blocking::Response =
        //     reqwest::blocking::get(self.fetch_url.clone() + "/api/traces?service=nginx-web-server&limit=10")
        //         .unwrap();

        let resp: reqwest::blocking::Response =
            // reqwest::blocking::get("http://45.56.102.188:16686/api/traces/19c8d9a240f56031")
            reqwest::blocking::get("http://45.56.102.188:16686/api/traces/01f2aa083a69e203")
                .unwrap();

        let message: String = fs::read_to_string(
            "/usr/local/pythia/test/test_trace_nested_concurrent_case.json").unwrap();

        // let resp_obj: JaegerPayload =
        //     serde_json::from_str(
        //         (resp.text().unwrap() as String).as_str()).unwrap();

        let resp_obj: JaegerPayload =
            serde_json::from_str(message.as_str()).unwrap();

        eprintln!("RESPONSE = {:?}", resp_obj);

        eprintln!("\n");
        eprintln!("\n");
        eprintln!("\n");

        let jt = &resp_obj.data[0];

        eprintln!("JAEGER TRACE TO SPAN TRACE:");
        eprintln!("{:?}", jt.to_trace());
        eprintln!("\n");
        eprintln!("\n");
        eprintln!("\n");

        let st = jt.to_trace();
        let res_cp = st.to_critical_path();

        let mut cp_events_sorted = res_cp.g.node_weights().collect::<Vec<&Event>>();
        cp_events_sorted.sort_by(|&a, &b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

        eprintln!("SPAN TRACE TO CRITICAL PATH:");
        eprintln!("{:?}", res_cp);
        eprintln!("\n");
        eprintln!("\n");
        eprintln!("\n");
        for e in cp_events_sorted {
            eprintln!("{:?}", e);
        }
        eprintln!("\n");
        eprintln!("\n");
        eprintln!("\n");

        return Vec::new();
    }

    fn reset_state(&mut self) {
        // TODO
        return
    }

    fn for_searchspace(&mut self) {
        // TODO
        return
    }
}

impl JaegerReader {
    pub fn from_settings(settings: &Settings) -> JaegerReader {
        return JaegerReader{
            fetch_url: settings.jaeger_url.clone(),
        }
    }
}