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
    layer: String
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

}

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