/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::{fs, vec};
use std::slice::SplitN;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use futures::Sink;
use hyper::http;
use itertools::Itertools;
use pythia_common::jaeger::JaegerRequestType;
use pythia_common::RequestType;
use crate::reader::Reader;
use crate::{Settings, Trace};
use crate::spantrace::{Span, SpanCache, SpanTrace};
use serde::{Serialize, Deserialize};
use crate::trace::Event;
use url::form_urlencoded;

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
    flags: Option<i32>,
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
    pub fn to_trace(&self, cache: &mut SpanCache) -> Result<SpanTrace, String> {
        let spans: Vec<Span> = self.spans.iter().map(|span| span.to_span(&self.processes)).collect();
        // if self.spans.len() == 0 {
        //     println!("{}", self.traceID);
        // }
        // let root_span: &JaegerSpan = self.spans.iter().filter(|&span| span.traceID == span.spanID).collect::<Vec<_>>()[0];
        let root_span_list: Vec<&JaegerSpan> = self.spans.iter()
            .filter(|&span| span.references.len() == 0).collect::<Vec<_>>();

        if root_span_list.len() == 0 {
            // println!("TRACE WITH NO ROOT:");
            // println!("{:?}", self.spans);
            println!("Could not find a root span!");

            return Err("Could not find a root span!".to_string());
        }
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

        for span in &spans {
            cache.add_span(span.clone(), self.traceID.clone())
        }

        let root_span = root_span_list[0];

        Ok(SpanTrace::from_span_list(
            spans, self.processes.get(
                root_span.processID.as_str()).unwrap().serviceName.clone() + ":" +
                root_span.operationName.as_str(),
            root_span.spanID.clone(), self.traceID.clone()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerPayload {
    data: Vec<JaegerTrace>,
}

pub struct JaegerReader {
    // connection: JaegerConnection // TODO: implement this
    fetch_url: String,
    problem_type: RequestType,
    fetch_all: bool,
    for_searchspace: bool,
    cycle_lookback: u128,
    span_cache: SpanCache
    // current_traces: HashMap<String, i64>
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

    fn get_recent_traces(&mut self) -> Vec<Trace> {
        return Vec::new();
    }

    // fn get_recent_span_traces(&mut self) -> Vec<SpanTrace> {
    //     let cur_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();
    //     println!("CUR TIME: {}", cur_time);
    //     println!("http://45.56.102.188:16686/api/traces?end={}&limit=20&maxDuration&minDuration&service=compose-post-service&start={}", cur_time, cur_time - 1*60*1000000);
    //     // "http://45.56.102.188:16686/api/traces?end=1720720913512000&limit=200&lookback=1h&maxDuration&minDuration&service=compose-post-service&start=1720717313512001"
    //     // "http://45.56.102.188:16686/api/traces?end=1721107686694029&limit=200&lookback=1h&maxDuration&minDuration&service=compose-post-service&start=1721107086694029"
    //     // "http://45.56.102.188:16686/api/traces?end=1721108216857049&limit=200&lookback=custom&maxDuration&minDuration&service=compose-post-service&start=1721107616857049"
    //
    //     let query_str = format!("http://localhost:16686/api/traces?end={}&limit=1000000000&lookback=custom&maxDuration&minDuration&service=compose-post-service&start={}", cur_time, cur_time - 1*60*1000000);
    //
    //     // let resp: reqwest::blocking::Response =
    //     //     reqwest::blocking::get(self.fetch_url.clone() + ":16686/api/traces/?end=1720719924151000&\
    //     //     limit=20&lookback=1h&maxDuration&minDuration&service=compose-post-service&start=1720716324151000")
    //     //         .unwrap();
    //
    //     // let resp: reqwest::blocking::Response =
    //     //     reqwest::blocking::get("http://45.56.102.188:16686/api/traces?end=".to_string() +
    //     //         cur_time.to_string().as_str() + "&limit=20&maxDuration&minDuration&" +
    //     //         "service=compose-post-service&start=" + (cur_time - 10*60*1000000).to_string().as_str())
    //     //         .unwrap();
    //
    //     println!("{}", query_str);
    //
    //     let resp: reqwest::blocking::Response =
    //         reqwest::blocking::get(query_str).unwrap();
    //
    //     let resp_obj: JaegerPayload =
    //         serde_json::from_str(
    //             (resp.text().unwrap() as String).as_str()).unwrap();
    //
    //     // println!("{:?}", resp_obj);
    //
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!("NUM TRACES = {}", resp_obj.data.len());
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //     println!();
    //
    //     let mut to_ret_traces = Vec::new();
    //
    //     for jt in resp_obj.data {
    //         to_ret_traces.push(jt.to_trace());
    //     }
    //
    //     return to_ret_traces
    // }

    fn get_recent_span_traces(&mut self) -> Vec<SpanTrace> {
        if self.fetch_all {
            let mut seen = HashSet::new();

            let mut to_return = Vec::new();

            println!("Calling all_operations() - jaeger.rs:263");
            for service in self.all_operations() {
                for tr in self.get_span_traces(service.to_string(), None, self.cycle_lookback) {
                    println!("TRACE TRACE TRACE: [{:?}]", tr);
                    if !seen.contains(tr.req_id.as_str()) {
                        seen.insert(tr.req_id.clone());
                        to_return.push(tr);
                    }
                }
            }

            to_return
        } else {
            self.get_span_traces(
                self.problem_type.to_string().as_str().split(":")
                    .collect::<Vec<&str>>()[0].to_string(),
                Some(self.problem_type.to_string().as_str().split(":")
                    .collect::<Vec<&str>>()[1].to_string()),
                self.cycle_lookback)
        }
    }

    fn reset_state(&mut self) {
        // TODO
        return
    }

    fn for_searchspace(&mut self) {
        self.for_searchspace = true;
    }

    fn all_operations(&mut self) -> Vec<RequestType> {
        let mut to_set_types = HashSet::new();

        let resp: reqwest::blocking::Response =
            reqwest::blocking::get(
                format!("{}/api/services", self.fetch_url)).unwrap();

        let resp_obj: JaegerServicesPayload =
            serde_json::from_str(
                (resp.text().unwrap() as String).as_str()).unwrap();

        // println!("Services:");
        // println!("{:?}", resp_obj.data.clone());

        for service in resp_obj.data {
            // let traces = self.get_span_traces(service, 60*60*1000000);
            println!("Service: {}", service);
            for trace in self.get_span_traces(service, None, 60000000) {
                for (_, span) in trace.spans {
                    if span.parent.is_empty() {
                        to_set_types.insert(
                            RequestType::Jaeger(JaegerRequestType{
                                rt: span.service + ":" + span.operation.as_str()
                            }));
                    }
                }
            }
        }

        to_set_types.into_iter().collect()
    }

    fn set_fetch_all(&mut self) {
        self.fetch_all = true;
    }

    fn get_candidate_events(&self, start: u64, end: u64, host: String) -> Vec<(String, String)> {
        // let overlap_refs = self.span_cache.find_overlaps_raw(start, end, host);
        //
        // let to_return = Vec::new();
        //
        // for (tid, sid) in overlap_refs {
        //
        // }
        //
        // to_return

        self.span_cache.find_overlaps_raw(start, end, host)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerServicesPayload {
    data: Vec<String>,
}

impl JaegerReader {
    pub fn from_settings(settings: &Settings) -> JaegerReader {
        return JaegerReader{
            fetch_url: settings.jaeger_url.clone(),
            problem_type: settings.problem_type.clone(),
            for_searchspace: false,
            fetch_all: false,
            cycle_lookback: settings.cycle_lookback,
            span_cache: SpanCache::init_cache()
        }
    }

    fn get_span_traces(&mut self, service: String, operation: Option<String>, lookback: u128) -> Vec<SpanTrace> {
        let cur_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();

        // let mut to_ret_map = HashMap::new();

        // let mut looked_back = 0;

        // loop {
        //     println!("Querying with lookback = {}", lookback);
        //     if looked_back == lookback {
        //         break;
        //     }
        //
        //     let query_str = format!(
        //         "http://localhost:16686/api/traces?end={}\
        //     &limit=1000000000&lookback=custom&maxDuration&minDuration{}&service={}&start={}",
        //         cur_time - looked_back, match &operation {
        //             Some(s) => format!("&operation={}", form_urlencoded::byte_serialize(
        //                 s.as_bytes()).collect::<String>().as_str()),
        //             None => "".to_string()
        //         }, service, cur_time - 10000000 - looked_back
        //     );
        //
        //     println!("Query String:");
        //     println!("{}", query_str);
        //
        //     let resp: reqwest::blocking::Response =
        //         reqwest::blocking::get(query_str).unwrap();
        //
        //     let resp_text = resp.text().unwrap() as String;
        //
        //     // println!("{}", resp_text.as_str());
        //
        //     let resp_obj: JaegerPayload =
        //         serde_json::from_str(resp_text.as_str()).unwrap();
        //
        //     // resp_obj.data.into_iter()
        //     //     .map(|jt| jt.to_trace()).collect()
        //     let traces = resp_obj.data.into_iter()
        //         .map(|jt| jt.to_trace(&mut self.span_cache))
        //         .filter(|tt_res| match tt_res {
        //             Ok(_) => true,
        //             _ => false
        //         })
        //         .map(|st_ok| st_ok.unwrap()).collect::<Vec<SpanTrace>>();
        //
        //     for trace in traces {
        //         if !to_ret_map.contains_key(trace.req_id.as_str()) {
        //             to_ret_map.insert(trace.req_id.clone(), trace);
        //         }
        //     }
        //
        //     looked_back += 5000000;
        // }
        //
        // to_ret_map.into_iter().map(|(_, v)| v).collect::<Vec<SpanTrace>>()

        let query_str = format!(
            "http://localhost:16686/api/traces?end={}\
            &limit=1000000000&lookback=custom&maxDuration&minDuration{}&service={}&start={}",
            cur_time, match operation {
                Some(s) => format!("&operation={}", form_urlencoded::byte_serialize(
                    s.as_bytes()).collect::<String>().as_str()),
                None => "".to_string()
            }, service, cur_time - lookback
        );

        println!("Query String:");
        println!("{}", query_str);

        let resp: reqwest::blocking::Response =
            reqwest::blocking::get(query_str).unwrap();

        let resp_text = resp.text().unwrap() as String;

        // println!("{}", resp_text.as_str());

        let resp_obj: JaegerPayload =
            serde_json::from_str(resp_text.as_str()).unwrap();

        println!("resp data len = {}", resp_obj.data.len());

        // resp_obj.data.into_iter()
        //     .map(|jt| jt.to_trace()).collect()
        resp_obj.data.into_iter()
            .map(|jt| jt.to_trace(&mut self.span_cache))
            .filter(|tt_res| match tt_res {
                Ok(_) => true,
                _ => false
            })
            .map(|st_ok| st_ok.unwrap()).collect()
    }
}