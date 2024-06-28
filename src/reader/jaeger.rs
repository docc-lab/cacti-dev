/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

use std::collections::HashMap;
use std::error::Error;
use futures::Sink;
use hyper::http;
use crate::reader::Reader;
use crate::{Settings, Trace};
use crate::spantrace::Span;
use serde::{Serialize, Deserialize};

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
            reqwest::blocking::get("http://45.56.102.188:16686/api/traces/19c8d9a240f56031")
                .unwrap();

        let resp_obj: JaegerPayload =
            serde_json::from_str(
                (resp.text().unwrap() as String).as_str()).unwrap();

        eprintln!("RESPONSE = {:?}", resp_obj);

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