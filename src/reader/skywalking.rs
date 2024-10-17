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
            operation: self.endpointName.clone(),
            start: DateTime::from_timestamp_millis(self.startTime as i64).unwrap().naive_utc(),
            duration: Duration::from_micros(self.endTime - self.startTime)
        }
    }
}

////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
struct SWBasicSpan {
    key: String,
    endpointNames: Vec<String>,
    duration: u64,
    start: u64,
    isError: bool,
    traceIds: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct SWBSInner {
    traces: Vec<SWBasicSpan>,
    total: u64
}

#[derive(Debug, Serialize, Deserialize)]
struct SWBSOuter {
    data: SWBSInner
}

#[derive(Debug, Serialize, Deserialize)]
struct SWBSPayload {
    data: SWBSOuter
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

pub struct SWReader {
    // connection: JaegerConnection // TODO: implement this
    fetch_url: String,
    problem_type: RequestType,
    fetch_all: bool,
    for_searchspace: bool,
    cycle_lookback: u128,
    span_cache: SpanCache
    // current_traces: HashMap<String, i64>
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
        let start_time = cur_date - Duration::from_secs(60*20);
        let end_time = cur_date + Duration::from_secs(60*20);

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

        let spans_query_str = resp.text().unwrap();

        // let spans_query_str = format!(
        //     "{{ \"query\": \"query queryTraces($condition: TraceQueryCondition) {{ data: queryBasicTraces(condition: $condition) {{ traces {{ key: segmentId endpointNames duration start isError traceIds }} total }} }}\", \"variables\": {{ \"condition\": {{ \"queryDuration\": {{ \"start\": \"{}-{}-{} {}{}\", \"end\": \"{}-{}-{} {}{}\", \"step\": \"DAY\"}}, \"traceState\": \"ALL\", \"paging\": {{ \"pageNum\": 1, \"pageSize\": 15, \"needTotal\": true }}, \"queryOrder\": \"BY_DURATION\" }} }} }}",
        //     start_time.year(), start_time.month(), start_time.day(),
        //     fmt_two_digit(start_time.hour()), fmt_two_digit(start_time.minute()),
        //     end_time.year(), end_time.month(), end_time.day(),
        //     fmt_two_digit(end_time.hour()), fmt_two_digit(end_time.minute()),
        // );

        // client = reqwest::blocking::Client::new();

        resp = client.post("http://localhost:12800/graphql")
            .json(&spans_query_str)
            .send().unwrap();

        let mut resp_text = resp.text().unwrap();

        println!();
        println!();
        println!("SPAN QUERY STRING:\n{}", spans_query_str.clone());
        println!();
        println!("SPAN QUERY RESPONSE TEXT:\n{}", resp_text);
        println!();
        println!();

        let resp_obj: SWBSPayload =
            serde_json::from_str(resp_text.as_str()).unwrap();

        let trace_ids = resp_obj.data.data.traces.into_iter()
            .map(|bs| bs.traceIds[0].clone()).collect::<Vec<String>>();

        #[derive(Serialize)]
        struct TraceQueryPayload {
            traceIds: Vec<String>
        }

        // let traces_query_str = serde_json::to_string(&TraceQueryPayload{
        //     traceIds: trace_ids
        // }).unwrap();

        client = reqwest::blocking::Client::new();

        resp = client.post("http://localhost:3000/traces")
            .json(&TraceQueryPayload{
                traceIds: trace_ids.clone()
            })
            // .json(traces_query_str)
            .send().unwrap();

        resp_text = resp.text().unwrap();

        let traces_payload: SWPayload =
            serde_json::from_str(resp_text.as_str()).unwrap();

        let traces = traces_payload.data;

        let mut to_return = Vec::new();

        for trace in traces {
            let spans = trace.spans.into_iter()
                .map(|s| s.to_span()).collect::<Vec<Span>>();

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
                    trace_id_parts.push(root_id_parts[i]);
                }
            }

            let trace_id = trace_id_parts.iter().join(".");

            for span in &spans {
                self.span_cache.add_span(span.clone(), trace_id.clone());
            }

            to_return.push(
                SpanTrace::from_span_list(
                    spans, root_span.operation, root_span.span_id,
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
        let fmt_two_digit = |i: u32| {
            if i < 10 {
                return format!("0{}", i);
            }

            return format!("{}", i);
        };

        // if self.fetch_all {
        let cur_date = chrono::Utc::now();
        let start_time = cur_date - Duration::from_secs(60*60*24);
        let end_time = cur_date + Duration::from_secs(60*60*24);

        let spans_query_str = format!(
                "{{ \"query\": \"query queryTraces($condition: TraceQueryCondition) {{ data: queryBasicTraces(condition: $condition) {{ traces {{ key: segmentId endpointNames duration start isError traceIds }} total }} }}\", \"variables\": {{ \"condition\": {{ \"queryDuration\": {{ \"start\": \"{}-{}-{} {}{}\", \"end\": \"{}-{}-{} {}{}\", \"step\": \"DAY\"}}, \"traceState\": \"ALL\", \"paging\": {{ \"pageNum\": 1, \"pageSize\": 15, \"needTotal\": true }}, \"queryOrder\": \"BY_DURATION\" }} }} }}",
                start_time.year(), start_time.month(), start_time.day(),
                fmt_two_digit(start_time.hour()), fmt_two_digit(start_time.minute()),
                end_time.year(), end_time.month(), end_time.day(),
                fmt_two_digit(end_time.hour()), fmt_two_digit(end_time.minute()),
            );

        let mut client = reqwest::blocking::Client::new();

        let mut resp: reqwest::blocking::Response = client.post("http://localhost:12800/graphql")
            .body(spans_query_str)
            .send().unwrap();

        let mut resp_text = resp.text().unwrap();

        let resp_obj: SWBSPayload =
            serde_json::from_str(resp_text.as_str()).unwrap();

        let trace_ids = resp_obj.data.data.traces.into_iter()
            .map(|bs| bs.traceIds[0].clone()).collect::<Vec<String>>();

        #[derive(Serialize)]
        struct TraceQueryPayload {
            traceIds: Vec<String>
        }

        let traces_query_str = serde_json::to_string(&TraceQueryPayload{
            traceIds: trace_ids
        }).unwrap();

        client = reqwest::blocking::Client::new();

        resp = client.post("http://localhost:3000/traces")
            .body(traces_query_str)
            .send().unwrap();

        resp_text = resp.text().unwrap();

        let traces_payload: SWPayload =
            serde_json::from_str(resp_text.as_str()).unwrap();

        let traces = traces_payload.data;

        let mut rt_set = HashSet::new();

        for trace in traces {
            for sw_span in trace.spans {
                let span = sw_span.to_span();

                if span.parent.is_empty() {
                    rt_set.insert(span.service + ":" + span.operation.as_str());
                }
            }
        }

        rt_set.into_iter().map(|rt_str: String| RequestType::SW(SWRequestType{ rt: rt_str }))
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
        return SWReader{
            fetch_url: settings.skywalking_url.clone(),
            problem_type: settings.problem_type.clone(),
            for_searchspace: false,
            fetch_all: false,
            cycle_lookback: settings.cycle_lookback,
            span_cache: SpanCache::init_cache()
        }
    }
}

