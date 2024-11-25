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
use chrono::{DateTime, NaiveDate, NaiveDateTime, Datelike, Timelike};
use futures::Sink;
use hyper::http;
use itertools::Itertools;
use petgraph::Graph;
use pythia_common::skywalking::SWRequestType;
use pythia_common::RequestType;
use crate::reader::Reader;
use crate::{Settings, Trace};
use crate::spantrace::{Span, SpanCache, SpanTrace};
use serde::{Serialize, Deserialize};
use crate::trace::Event;
use url::form_urlencoded;
use crate::reader::jaeger::JaegerReader;

#[derive(Debug, Serialize, Deserialize)]
struct SWRef {
    traceId: String,
    parentSegmentId: String,
    parentSpanId: i64,
    refType: String
}

#[derive(Debug, Serialize, Deserialize)]
struct SWSpan {
    traceId: String,
    segmentId: String,
    spanId: i64,
    parentSpanId: i64,
    serviceCode: String,
    startTime: u64,
    endTime: u64,
    endpointName: String,
    spanType: String,
    peer: String,
    component: String,
    isError: bool,
    layer: String,
    refs: Vec<SWRef>
}

#[derive(Debug, Serialize, Deserialize)]
struct SWResult {
    spans: Vec<SWSpan>
}

#[derive(Debug, Serialize, Deserialize)]
struct SWPayload {
    success: bool,
    data: Vec<SWResult>,
    message: String
}

impl SWSpan {
    pub fn to_span(&self) -> Span {
        println!("SPAN ID = {}.{}", self.segmentId, self.spanId);
        return Span{
            span_id: format!("{}.{}", self.segmentId, self.spanId),
            parent: match self.refs.len() {
                0 => match self.parentSpanId {
                    -1 => "".to_string(),
                    _ => format!("{}.{}", self.segmentId, self.parentSpanId)
                },
                _ => format!("{}.{}", self.refs[0].parentSegmentId, self.refs[0].parentSpanId)
            },
            service: self.serviceCode.clone(),
            host: self.serviceCode.clone(),
            operation: (|s: String| {
                let mut parts = s.split("/").collect::<Vec<&str>>();

                parts = parts.into_iter().filter(|p| {
                    let p_string = p.to_string();
                    !(
                        p_string.contains("-") ||
                            p_string.contains(" ") ||
                            p_string.contains("{") ||
                            p_string.contains(".")
                    )
                }).collect::<Vec<&str>>();

                parts.join("/")
            })(self.endpointName.clone()),
            start: DateTime::from_timestamp_millis(self.startTime as i64).unwrap().naive_utc(),
            duration: Duration::from_millis(self.endTime - self.startTime)
        }
    }
}

////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
struct SWBasicSpan {
    key: String,
    endpointNames: Vec<String>,
    duration: u64,
    start: String,
    isError: bool,
    traceIds: Vec<String>
}

// #[derive(Debug, Serialize, Deserialize)]
// struct SWBSInner {
//     traces: Vec<SWBasicSpan>,
//     total: u64
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// struct SWBSOuter {
//     data: SWBSInner
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// struct SWBSPayload {
//     data: SWBSOuter
// }

#[derive(Debug, Serialize, Deserialize)]
struct SWBSPayload {
    success: bool,
    traces: Vec<SWBasicSpan>,
    total: u64,
    message: String,
}

////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
struct TFTraceItem {
    key: String,
    endpointNames: Vec<String>,
    duration: u64,
    start: u64,
    isError: bool,
    traceIds: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct TFTraceData {
    traces: Vec<TFTraceItem>,
    total: u64
}

#[derive(Debug, Serialize, Deserialize)]
struct SWTFInner {
    traceData: TFTraceData
}

#[derive(Debug, Serialize, Deserialize)]
struct SWTimedFetch {
    data: SWTFInner
}

////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
struct SWService {
    key: String,
    label: String,
    group: String
}

#[derive(Debug, Serialize, Deserialize)]
struct SWSInner {
    services: Vec<SWService>
}

#[derive(Debug, Serialize, Deserialize)]
struct SWServices {
    data: SWSInner
}

////////////////////////////////////////

// struct QueryPartNode {
//     part: String,
//     count: u64,
//     children: Vec<String>,
// }
// 
// impl QueryPartNode {
//     pub fn 
// }
// 
// struct QueryPartTree {
//     base_part: QueryPartNode,
//     query_parts: HashMap<String, QueryPartNode>,
// }
// 
// impl QueryPartTree {
//     
// }

////////////////////////////////////////

pub struct SWReader {
    // connection: JaegerConnection // TODO: implement this
    fetch_url: String,
    problem_type: RequestType,
    fetch_all: bool,
    for_searchspace: bool,
    cycle_lookback: u128,
    span_cache: SpanCache,
    // current_traces: HashMap<String, i64>
    operations: Vec<String>,
    op_prefixes: Vec<(String, String)>
}

impl Reader for SWReader {
    fn read_file(&mut self, filename: &str) -> Trace {
        todo!()
    }

    fn read_dir(&mut self, foldername: &str) -> Vec<Trace> {
        todo!()
    }

    fn get_trace_from_base_id(&mut self, id: &str) -> Result<Trace, Box<dyn Error>> {
        todo!()
    }

    fn get_recent_traces(&mut self) -> Vec<Trace> {
        return Vec::new();
    }

    fn get_recent_span_traces(&mut self) -> Vec<SpanTrace> {
        let fmt_two_digit = |i: u32| {
            if i < 10 {
                return format!("0{}", i);
            }

            return format!("{}", i);
        };

        // if self.fetch_all {
        let cur_date = chrono::Utc::now();
        // let start_time = cur_date - Duration::from_secs(60*20);
        let start_time = cur_date - Duration::from_secs(60*15);
        // let end_time = cur_date + Duration::from_secs(60*20);
        // let end_time = cur_date + Duration::from_secs(60*60*24);
        let end_time = cur_date;

        #[derive(Serialize)]
        struct SpanQueryFormatReq {
            start_year: i32,
            start_month: String,
            start_day: String,
            start_hour: String,
            start_minute: String,
            end_year: i32,
            end_month: String,
            end_day: String,
            end_hour: String,
            end_minute: String,
            page_num: u64,
        }

        let mut client = reqwest::blocking::Client::new();

        let mut resp: reqwest::blocking::Response;

        let mut resp_text: String;

        let mut resp_obj: SWBSPayload;

        let mut all_trace_ids: Vec<String> = Vec::new();

        let mut page_num = 1u64;
        let mut start_time_iter = end_time - Duration::from_secs(30);
        let mut end_time_iter = end_time;

        let mut next_page = true;
        let mut print_iter = 1;
        loop {
            println!("SpanQuery Retrieval Loop #{}", print_iter);
            print_iter += 1;

            resp = client.post("http://localhost:3000/spanquery")
                .json(&SpanQueryFormatReq{
                    start_year: start_time_iter.naive_local().year(),
                    start_month: fmt_two_digit(start_time_iter.naive_local().month()),
                    start_day: fmt_two_digit(start_time_iter.naive_local().day()),
                    start_hour: fmt_two_digit(start_time_iter.naive_local().hour()),
                    start_minute: fmt_two_digit(start_time_iter.naive_local().minute()),
                    end_year: end_time_iter.naive_local().year(),
                    end_month: fmt_two_digit(end_time_iter.naive_local().month()),
                    end_day: fmt_two_digit(end_time_iter.naive_local().day()),
                    end_hour: fmt_two_digit(end_time_iter.naive_local().hour()),
                    end_minute: fmt_two_digit(end_time_iter.naive_local().minute()),
                    page_num,
                }).send().unwrap();

            resp_text = resp.text().unwrap();

            resp_obj = serde_json::from_str(resp_text.as_str()).unwrap();

            let mut trace_ids = resp_obj.traces.into_iter()
                .map(|bs| bs.traceIds[0].clone()).collect::<Vec<String>>();

            // if trace_ids.len() < 10000 {
            //     to_break = true;
            // }
            if trace_ids.len() == 0 {
                next_page = false;
            }

            all_trace_ids.append(&mut trace_ids);

            if next_page {
                page_num += 1;
            } else {
                if start_time_iter == start_time {
                    break;
                } else {
                    start_time_iter = start_time_iter - Duration::from_secs(30);
                    end_time_iter = end_time_iter - Duration::from_secs(30);
                    page_num = 1;
                }
            }
        }

        let mut trace_ids_set = HashSet::new();
        for trace_id in all_trace_ids {
            trace_ids_set.insert(trace_id.clone());
        }

        all_trace_ids = trace_ids_set.into_iter().collect::<Vec<String>>();

        #[derive(Serialize)]
        struct TraceQueryPayload {
            traceIds: Vec<String>
        }

        // trace_ids = trace_ids.drain(..100).collect::<Vec<String>>();

        client = reqwest::blocking::Client::new();

        let mut traces: Vec<SWResult> = Vec::new();

        let loop_iters = ((all_trace_ids.len() as f64)/1000.0).ceil() as u64;
        let mut i = 0;

        println!();
        println!();
        println!("GENERIC OPERATIONS:");
        for (op_p, op) in &self.op_prefixes {
            println!("[ Prefix: {} ] ==== [ Operation: {} ]", op_p, op);
        }
        println!();
        println!();

        loop {
            println!("Trace retrieval loop {}/{}", i, loop_iters);
            i += 1;
            
            if all_trace_ids.len() == 0 {
                break;
            }

            let mut min_range_end = 1000;
            if all_trace_ids.len() < 1000 {
                min_range_end = all_trace_ids.len();
            }
            let cur_trace_ids = all_trace_ids.drain(..min_range_end).collect::<Vec<String>>();

            resp = client.post("http://localhost:3000/traces")
                .json(&TraceQueryPayload{
                    traceIds: cur_trace_ids
                })
                .timeout(Duration::from_secs(180))
                .send().unwrap();

            resp_text = resp.text().unwrap();

            let mut traces_payload: SWPayload =
                serde_json::from_str(resp_text.as_str()).unwrap();

            traces.append(&mut (traces_payload.data));
        }

        // let traces = traces_payload.data;

        let mut to_return = Vec::new();

        // let mut generics = HashSet::new();
        // for operation in &self.operations {
        //     let yeet = operation.split("/{").collect::<Vec<&str>>()[0].to_string();
        //     generics.insert((
        //         yeet,
        //         operation.clone()
        //     ));
        // }

        for trace in traces {
            // let mut spans = trace.spans.into_iter()
            //     .map(|s| s.to_span()).collect::<Vec<Span>>();
            let spans = trace.spans.into_iter()
                .map(|s| s.to_span()).collect::<Vec<Span>>();
            
            // spans = spans.into_iter().map(|s| {
            //     let mut to_return = s;
            //
            //     for (op_p, op) in &self.op_prefixes {
            //         if to_return.operation.contains(op_p.as_str()) {
            //             to_return.operation = op.clone();
            //             break;
            //         }
            //     }
            //
            //     to_return
            // }).collect::<Vec<Span>>();

            let mut root_span = spans[0].clone();
            for span in &spans {
                if span.parent.is_empty() {
                    root_span = span.clone();
                }
            }

            let root_id_parts = root_span.span_id.split(".").collect::<Vec<&str>>();
            let mut trace_id_parts = Vec::new();
            let mut i = 1;
            loop {
                if i == root_id_parts.len() {
                    break;
                } else {
                    trace_id_parts.push(root_id_parts[i-1]);
                }
                
                i += 1;
            }

            let trace_id = trace_id_parts.iter().join(".");

            for span in &spans {
                self.span_cache.add_span(span.clone(), trace_id.clone());
            }
            
            println!("skywalking.rs - get_recent_span_traces - root_span.operation = {}", root_span.operation);

            to_return.push(
                SpanTrace::from_span_list(
                    spans,
                    format!("{}:{}", root_span.service, root_span.operation),
                    root_span.span_id,
                    trace_id
                )
            );
        }

        to_return
        // } else {
        //     let cur_date = chrono::Utc::now();
        //     let start_time = cur_date - Duration::from_secs(60*20);
        //     let end_time = cur_date + Duration::from_secs(60*20);
        //
        //     let services_query_str = format!(
        //         "{{ \"query\": \"query queryServices($duration: Duration!,$keyword: String!) {{ services: getAllServices(duration: $duration, group: $keyword) {{ key: id label: name group }} }}\", \"variables\": {{ \"duration\": {{ \"start\": \"{}-{}-{} {}{}\",\"end\": \"{}-{}-{} {}{}\", \"step\": \"MINUTE\" }}, \"keyword\":\"\" }} }}",
        //         start_time.year(), start_time.month(), start_time.day(),
        //         fmt_two_digit(start_time.hour()), fmt_two_digit(start_time.minute()),
        //         end_time.year(), end_time.month(), end_time.day()
        //         fmt_two_digit(end_time.hour()), fmt_two_digit(end_time.minute()),
        //     );
        //
        //     let client = reqwest::blocking::Client::new();
        //
        //     let resp: reqwest::blocking::Response = client.post("http://localhost:12800/graphql")
        //         .body(services_query_str)
        //         .send().unwrap();
        //
        //     let resp_text = resp.text().unwrap();
        //
        //     let resp_obj: SWServices =
        //         serde_json::from_str(resp_text.as_str()).unwrap();
        //
        //     let service_names = resp_obj.data.services.into_iter().map(
        //         |service| service.label
        //     ).collect::<Vec<String>>();
        //
        //     Vec::new()
        // }
    }

    fn reset_state(&mut self) {
        // TODO
        return
    }

    fn for_searchspace(&mut self) {
        self.for_searchspace = true;
    }

    fn all_operations(&mut self) -> Vec<RequestType> {
        println!();
        println!();
        println!("==========\nSKYWALKING READER -- GETTING ALL OPERATIONS\n==========");
        println!();
        println!();
        
        let fmt_two_digit = |i: u32| {
            if i < 10 {
                return format!("0{}", i);
            }

            return format!("{}", i);
        };

        // if self.fetch_all {
        let cur_date = chrono::Utc::now();
        // let start_time = cur_date - Duration::from_secs(60*60*24);
        let start_time = cur_date - Duration::from_secs(60*20);
        // let end_time = cur_date + Duration::from_secs(60*60*24);
        let end_time = cur_date + Duration::from_secs(60*60*24);

        #[derive(Serialize)]
        struct SpanQueryFormatReq {
            start_year: i32,
            start_month: u32,
            start_day: u32,
            start_hour: String,
            start_minute: String,
            end_year: i32,
            end_month: u32,
            end_day: u32,
            end_hour: String,
            end_minute: String,
        }

        let mut client = reqwest::blocking::Client::new();

        let mut resp: reqwest::blocking::Response  = client.post("http://localhost:3000/spanquery")
            .json(&SpanQueryFormatReq{
                start_year: start_time.year(),
                start_month: start_time.month(),
                start_day: start_time.day(),
                start_hour: fmt_two_digit(start_time.hour()),
                start_minute: fmt_two_digit(start_time.minute()),
                end_year: end_time.year(),
                end_month: end_time.month(),
                end_day: end_time.day(),
                end_hour: fmt_two_digit(end_time.hour()),
                end_minute: fmt_two_digit(end_time.minute()),
            }).send().unwrap();

        let mut resp_text = resp.text().unwrap();

        let resp_obj: SWBSPayload =
            serde_json::from_str(resp_text.as_str()).unwrap();

        let mut trace_ids = resp_obj.traces.into_iter()
            .map(|bs| bs.traceIds[0].clone()).collect::<Vec<String>>();

        #[derive(Serialize)]
        struct TraceQueryPayload {
            traceIds: Vec<String>
        }

        client = reqwest::blocking::Client::new();

        let mut traces: Vec<SWResult> = Vec::new();
        
        let loop_iters = ((trace_ids.len() as f64)/1000.0).ceil() as u64;
        let mut i = 0;

        loop {
            println!("Trace retrieval loop {}/{}", i, loop_iters);
            i += 1;
            
            if trace_ids.len() == 0 {
                break;
            }

            let mut min_range_end = 1000;
            if trace_ids.len() < 1000 {
                min_range_end = trace_ids.len();
            }
            let cur_trace_ids = trace_ids.drain(..min_range_end).collect::<Vec<String>>();

            resp = client.post("http://localhost:3000/traces")
                .json(&TraceQueryPayload{
                    traceIds: cur_trace_ids
                })
                .timeout(Duration::from_secs(120))
                .send().unwrap();

            resp_text = resp.text().unwrap();

            let mut traces_payload: SWPayload =
                serde_json::from_str(resp_text.as_str()).unwrap();

            traces.append(&mut (traces_payload.data));
        }

        let traces_payload: SWPayload =
            serde_json::from_str(resp_text.as_str()).unwrap();

        let traces = traces_payload.data;

        let mut rt_set = HashSet::new();

        for trace in traces {
            for sw_span in trace.spans {
                let span = sw_span.to_span();

                // if span.parent.is_empty() {
                //     rt_set.insert(span.service + ":" + span.operation.as_str());
                // }
                rt_set.insert(span.service + ":" + span.operation.as_str());
            }
        }

        // rt_set.into_iter().map(|rt_str: String| RequestType::SW(SWRequestType{ rt: rt_str }))
        //     .collect::<Vec<RequestType>>()
        
        let mut with_generics = HashSet::new();
        let mut without_generics = HashSet::new();
        for rt in rt_set {
            let num_generics = (rt.split("/{").collect::<Vec<&str>>().len() - 1) as u64;
            
            if num_generics > 0 {
                with_generics.insert(rt);
            } else {
                without_generics.insert(rt);
            }
        }
        
        let mut to_return = with_generics.clone();
        let mut generic_prefixes = HashSet::new();
        // for rt in &with_generics {
        //     generic_prefixes.insert(
        //         (rt.split("/{").collect::<Vec<&str>>()[0], rt.clone()));
        // }

        for rt in &with_generics {
            generic_prefixes.insert(rt.split("/{").collect::<Vec<&str>>()[0],);
        }
        
        for rt in without_generics {
            let mut contains_generic = false;
            // let mut generic = "".to_string();
            for gp in &generic_prefixes {
                if rt.contains(gp) {
                    contains_generic = true;
                    // generic = g.clone();
                    break;
                }
            }
            // if contains_generic {
            //     to_return.insert(generic);
            // } else {
            //     to_return.insert(rt);
            // }
            if !contains_generic {
                to_return.insert(rt);
            }
        }
        
        self.operations = to_return.clone().into_iter().collect();
        self.op_prefixes = self.operations.clone().into_iter().map(|op| {
            (op.split("/{").collect::<Vec<&str>>()[0].to_string(), op)
        }).collect::<Vec<(String, String)>>();

        to_return.into_iter().map(|rt_str: String| RequestType::SW(SWRequestType{ rt: rt_str }))
            .collect::<Vec<RequestType>>()
    }

    fn set_fetch_all(&mut self) {
        self.fetch_all = true;
    }

    fn get_candidate_events(&self, start: u64, end: u64, host: String) -> Vec<(String, String)> {
        self.span_cache.find_overlaps_raw(start, end, host)
    }
}

impl SWReader {
    pub fn from_settings(settings: &Settings) -> SWReader {
        let mut to_return = SWReader{
            fetch_url: settings.skywalking_url.clone(),
            problem_type: settings.problem_type.clone(),
            for_searchspace: false,
            fetch_all: false,
            cycle_lookback: settings.cycle_lookback,
            span_cache: SpanCache::init_cache(),
            operations: Vec::new(),
            op_prefixes: Vec::new(),
        };
        
        to_return
    }
}

