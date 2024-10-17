/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

//! This file contains all the hard-coded settings and parsing code for the toml file.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Serialize, Deserialize};

use config::{Config, File, FileFormat};
use pythia_common::{OSPRequestType, RequestType, REQUEST_TYPES};
use reqwest::get;
use crate::reader::reader_from_settings;

use crate::search::SearchStrategyType;

const SETTINGS_PATH: &str = "./etc/pythia/controller.toml";
const DECISION_EPOCH: Duration = Duration::from_secs(120);
// const PYTHIA_JIFFY: Duration = Duration::from_secs(20);
const PYTHIA_JIFFY: Duration = Duration::from_secs(10);
const GC_EPOCH: Duration = Duration::from_secs(120);
const GC_KEEP_DURATION: Duration = Duration::from_secs(360);
const TRACEPOINTS_PER_EPOCH: usize = 10;
const DISABLE_RATIO: f32 = 0.1;
const TRACE_SIZE_LIMIT: u32 = 100000000;
const N_WORKERS: usize = 4;
const FREE_KEYS: bool = false;

#[derive(Debug)]
pub struct Settings {
    pub application: ApplicationType,
    pub manifest_file: PathBuf,
    pub pythia_clients: Vec<String>,
    pub redis_url: String,
    pub zipkin_url: String,
    pub jaeger_url: String,
    pub skywalking_url: String,
    pub xtrace_url: String,
    pub uber_trace_dir: PathBuf,
    pub DEATHSTAR_trace_dir: PathBuf,
    pub hdfs_control_file: PathBuf,
    pub deathstar_control_file: PathBuf,
    pub emit_events: bool,
    // pub problem_type: OSPRequestType,
    pub problem_type: RequestType,

    pub search_strategy: SearchStrategyType,
    pub jiffy: Duration,
    pub decision_epoch: Duration,
    pub gc_epoch: Duration,
    pub gc_keep_duration: Duration,
    pub tracepoints_per_epoch: usize,
    pub disable_ratio: f32,
    pub trace_size_limit: u32,
    pub n_workers: usize,
    pub free_keys: bool,

    pub all_request_types: Vec<RequestType>,
    pub cycle_lookback: u128,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ApplicationType {
    HDFS,
    OpenStack,
    Uber,
    DEATHSTAR,
    Zipkin,
    Jaeger,
    SkyWalking,
}

impl ApplicationType {
    pub fn as_str(&self) -> &str {
        match self {
            ApplicationType::HDFS => "HDFS",
            ApplicationType::OpenStack => "OpenStack",
            ApplicationType::DEATHSTAR => "DEATHSTAR",
            ApplicationType::Uber => "Uber",
            ApplicationType::Jaeger => "Jaeger",
            ApplicationType::Zipkin => "Zipkin",
            ApplicationType::SkyWalking => "SkyWalking",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerServicesPayload {
    data: Vec<String>,
}

impl Settings {
    pub fn read() -> Settings {
        // let mut settings = Config::default();
        // let mut settings = Config::builder();
        // settings
        //     .merge(File::new(SETTINGS_PATH, FileFormat::Toml))
        //     .unwrap();
        let mut settings_builder = Config::builder()
            .set_default("default", "1").unwrap()
            .add_source(File::new(SETTINGS_PATH, FileFormat::Toml))
            .set_override("override", "1").unwrap();
        let mut settings = settings_builder.build().unwrap();
        let get_setting = |key: &str| { settings.get::<String>(key).unwrap() as String };
        // let results = settings.try_into::<HashMap<String, String>>().unwrap();
        // let manifest_file = PathBuf::from(results.get("manifest_file").unwrap());
        let manifest_file = PathBuf::from(get_setting("manifest_file"));
        // let hdfs_control_file = PathBuf::from(results.get("hdfs_control_file").unwrap());
        let hdfs_control_file = PathBuf::from(get_setting("hdfs_control_file"));
        // let deathstar_control_file = PathBuf::from(results.get("hdfs_control_file").unwrap());
        let deathstar_control_file = PathBuf::from(get_setting("hdfs_control_file"));
        // let pythia_clients = results.get("pythia_clients").unwrap();
        let pythia_clients = get_setting("pythia_clients");
        let pythia_clients = if pythia_clients.len() == 0 {
            Vec::new()
        } else {
            pythia_clients.split(",").map(|x| x.to_string()).collect()
        };
        // let emit_events = match results.get("emit_events").unwrap().as_str() {
        let emit_events = match get_setting("emit_events").as_str() {
            "true" => true,
            _ => false
        };

        let mut to_return = Settings {
            manifest_file,
            hdfs_control_file,
            deathstar_control_file,
            pythia_clients,
            // redis_url: results.get("redis_url").unwrap().to_string(),
            // zipkin_url: results.get("zipkin_url").unwrap().to_string(),
            // jaeger_url: results.get("jaeger_url").unwrap().to_string(),
            // uber_trace_dir: PathBuf::from(results.get("uber_trace_dir").unwrap()),
            // DEATHSTAR_trace_dir: PathBuf::from(results.get("DEATHSTAR_trace_dir").unwrap()),
            // application: match results.get("application").unwrap().as_str() {
            redis_url: get_setting("redis_url"),
            zipkin_url: get_setting("zipkin_url"),
            jaeger_url: get_setting("jaeger_url"),
            skywalking_url: get_setting("skywalking_url"),
            uber_trace_dir: PathBuf::from(get_setting("uber_trace_dir")),
            DEATHSTAR_trace_dir: PathBuf::from(get_setting("DEATHSTAR_trace_dir")),
            application: match get_setting("application").as_str() {
                "OpenStack" => ApplicationType::OpenStack,
                "HDFS" => ApplicationType::HDFS,
                "Uber" => ApplicationType::Uber,
                "DEATHSTAR" => ApplicationType::DEATHSTAR,
                "Zipkin" => ApplicationType::Zipkin,
                "Jaeger" => ApplicationType::Jaeger,
                "SkyWalking" => ApplicationType::SkyWalking,
                _ => panic!("Unknown application type"),
            },
            // xtrace_url: results.get("xtrace_url").unwrap().to_string(),
            xtrace_url: get_setting("xtrace_url"),
            decision_epoch: DECISION_EPOCH,
            // search_strategy: match results.get("search_strategy").unwrap().as_str() {
            search_strategy: match get_setting("search_strategy").as_str() {
                "Flat" => SearchStrategyType::Flat,
                "Hierarchical" => SearchStrategyType::Hierarchical,
                "Historic" => SearchStrategyType::Historic,
                _ => panic!("Unknown search strategy"),
            },
            tracepoints_per_epoch: TRACEPOINTS_PER_EPOCH,
            jiffy: PYTHIA_JIFFY,
            gc_epoch: GC_EPOCH,
            gc_keep_duration: GC_KEEP_DURATION,
            disable_ratio: DISABLE_RATIO,
            trace_size_limit: TRACE_SIZE_LIMIT,
            n_workers: N_WORKERS,
            free_keys: FREE_KEYS,
            emit_events,
            // problem_type: RequestType::from_str(results.get("problem_type").unwrap().as_str()).unwrap()
            // problem_type: OSPRequestType::from_str(get_setting("problem_type").as_str()).unwrap()
            problem_type: RequestType::from_str(
                get_setting("problem_type").as_str(),
                get_setting("application").as_str()).unwrap(),
            all_request_types: Vec::new(),
            cycle_lookback: get_setting("cycle_lookback").parse::<u128>().unwrap()
        };

        to_return.all_request_types = match get_setting("application").as_str() {
            "OpenStack" =>  REQUEST_TYPES.clone().into_iter()
                .map(|rt| RequestType::OSP(rt)).collect(),
            "Jaeger" => {
                println!("Calling all_operations() - settings.rs:187");
                reader_from_settings(&to_return).all_operations()
            },
            _ => Vec::new()
            // _ => panic!("Unknown application type"),
        };

        to_return
    }

    pub fn read_pt(problem_type: String) -> Settings {
        // let mut settings = Config::default();
        // let mut settings = Config::builder();
        // settings
        //     .merge(File::new(SETTINGS_PATH, FileFormat::Toml))
        //     .unwrap();
        let mut settings_builder = Config::builder()
            .set_default("default", "1").unwrap()
            .add_source(File::new(SETTINGS_PATH, FileFormat::Toml))
            .set_override("override", "1").unwrap();
        let mut settings = settings_builder.build().unwrap();
        let get_setting = |key: &str| { settings.get::<String>(key).unwrap() as String };
        // let results = settings.try_into::<HashMap<String, String>>().unwrap();
        // let manifest_file = PathBuf::from(results.get("manifest_file").unwrap());
        let manifest_file = PathBuf::from(get_setting("manifest_file"));
        // let hdfs_control_file = PathBuf::from(results.get("hdfs_control_file").unwrap());
        let hdfs_control_file = PathBuf::from(get_setting("hdfs_control_file"));
        // let deathstar_control_file = PathBuf::from(results.get("hdfs_control_file").unwrap());
        let deathstar_control_file = PathBuf::from(get_setting("hdfs_control_file"));
        // let pythia_clients = results.get("pythia_clients").unwrap();
        let pythia_clients = get_setting("pythia_clients");
        let pythia_clients = if pythia_clients.len() == 0 {
            Vec::new()
        } else {
            pythia_clients.split(",").map(|x| x.to_string()).collect()
        };
        // let emit_events = match results.get("emit_events").unwrap().as_str() {
        let emit_events = match get_setting("emit_events").as_str() {
            "true" => true,
            _ => false
        };

        let mut to_return = Settings {
            manifest_file,
            hdfs_control_file,
            deathstar_control_file,
            pythia_clients,
            // redis_url: results.get("redis_url").unwrap().to_string(),
            // zipkin_url: results.get("zipkin_url").unwrap().to_string(),
            // jaeger_url: results.get("jaeger_url").unwrap().to_string(),
            // uber_trace_dir: PathBuf::from(results.get("uber_trace_dir").unwrap()),
            // DEATHSTAR_trace_dir: PathBuf::from(results.get("DEATHSTAR_trace_dir").unwrap()),
            // application: match results.get("application").unwrap().as_str() {
            redis_url: get_setting("redis_url"),
            zipkin_url: get_setting("zipkin_url"),
            jaeger_url: get_setting("jaeger_url"),
            skywalking_url: get_setting("skywalking_url"),
            uber_trace_dir: PathBuf::from(get_setting("uber_trace_dir")),
            DEATHSTAR_trace_dir: PathBuf::from(get_setting("DEATHSTAR_trace_dir")),
            application: match get_setting("application").as_str() {
                "OpenStack" => ApplicationType::OpenStack,
                "HDFS" => ApplicationType::HDFS,
                "Uber" => ApplicationType::Uber,
                "DEATHSTAR" => ApplicationType::DEATHSTAR,
                "Zipkin" => ApplicationType::Zipkin,
                "Jaeger" => ApplicationType::Jaeger,
                "SkyWalking" => ApplicationType::SkyWalking,
                _ => panic!("Unknown application type"),
            },
            // xtrace_url: results.get("xtrace_url").unwrap().to_string(),
            xtrace_url: get_setting("xtrace_url"),
            decision_epoch: DECISION_EPOCH,
            // search_strategy: match results.get("search_strategy").unwrap().as_str() {
            search_strategy: match get_setting("search_strategy").as_str() {
                "Flat" => SearchStrategyType::Flat,
                "Hierarchical" => SearchStrategyType::Hierarchical,
                "Historic" => SearchStrategyType::Historic,
                _ => panic!("Unknown search strategy"),
            },
            tracepoints_per_epoch: TRACEPOINTS_PER_EPOCH,
            jiffy: PYTHIA_JIFFY,
            gc_epoch: GC_EPOCH,
            gc_keep_duration: GC_KEEP_DURATION,
            disable_ratio: DISABLE_RATIO,
            trace_size_limit: TRACE_SIZE_LIMIT,
            n_workers: N_WORKERS,
            free_keys: FREE_KEYS,
            emit_events,
            // problem_type: RequestType::from_str(results.get("problem_type").unwrap().as_str()).unwrap()
            // problem_type: OSPRequestType::from_str(get_setting("problem_type").as_str()).unwrap()
            problem_type: RequestType::from_str(
                problem_type.as_str(),
                get_setting("application").as_str()).unwrap(),
            all_request_types: Vec::new(),
            cycle_lookback: get_setting("cycle_lookback").parse::<u128>().unwrap()
        };

        to_return.all_request_types = match get_setting("application").as_str() {
            "OpenStack" =>  REQUEST_TYPES.clone().into_iter()
                .map(|rt| RequestType::OSP(rt)).collect(),
            "Jaeger" => {
                println!("Calling all_operations() - settings.rs:286");
                reader_from_settings(&to_return).all_operations()
            },
            _ => Vec::new()
            // _ => panic!("Unknown application type"),
        };

        to_return
    }
}
