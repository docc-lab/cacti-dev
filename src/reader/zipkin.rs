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
use pythia_common::RequestType;
use crate::reader::Reader;
use crate::{Settings, Trace};
use crate::spantrace::{Span, SpanTrace};

pub struct ZipkinReader {
    // connection: ZipkinConnection // TODO: implement this
    fetch_url: String
}

impl Reader for ZipkinReader {
    fn all_operations(&mut self) -> Vec<RequestType> {
        Vec::new()
    }

    fn set_fetch_all(&mut self) {}

    fn get_recent_span_traces(&mut self) -> Vec<SpanTrace> {
        return Vec::new();
    }

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
    fn get_recent_traces(&mut self) -> Vec<Trace> {
        // let mut ids = Vec::new();

        let mut traces: HashMap<String, Vec<Span>> = HashMap::new();

        // let resp: reqwest::blocking::Response = reqwest::blocking::get("https://httpbin.org/ip").unwrap();
        let resp: reqwest::blocking::Response =
            reqwest::blocking::get(self.fetch_url.clone())
                .unwrap();
        // match resp {
        //     Ok()
        // }

        eprintln!("RESPONSE = {:?}", resp.text());

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

impl ZipkinReader {
    pub fn from_settings(settings: &Settings) -> ZipkinReader {
        return ZipkinReader{
            fetch_url: settings.zipkin_url.clone(),
        }
    }
}