/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

//! General trace implementation
//!

use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::sync::Mutex;
use std::time::Duration;

use bimap::BiMap;
use chrono::NaiveDateTime;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::StableGraph;
use petgraph::Direction;
use serde::de;
use serde::ser;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;
use stats::variance;
use pythia_common::OSPRequestType;
use pythia_common::RequestType;

use std::collections::HashMap;

//The enum Value contains variants which are added depending on the type of key-value pairs needed
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum Value {
    UnsignedInt(u64),
    Str(String),
    SignedInt(i64),
    //float(f64),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub enum IDType {
    UUID(Uuid),
    STRING(String),
}

impl Display for IDType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            IDType::UUID(u) => write!(f, "{}", u),
            IDType::STRING(s) => write!(f, "{}", s),
        }
    }
}

/// A general-purpose trace which does not contain application-specific things
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trace {
    pub g: StableGraph<Event, DAGEdge>,
    // pub base_id: Uuid,
    pub base_id: IDType,
    pub start_node: NodeIndex,
    pub end_node: NodeIndex,
    // pub request_type: OSPRequestType,
    pub request_type: RequestType,
    pub duration: Duration,
    /// used by osprofiler to find keys to delete from redis
    pub keys: Vec<String>,
}

impl Trace {
    // pub fn new(base_id: &Uuid) -> Self {
    pub fn new(base_id: &IDType) -> Self {
        Trace {
            g: StableGraph::new(),
            base_id: base_id.clone(),
            start_node: NodeIndex::end(),
            end_node: NodeIndex::end(),
            // request_type: OSPRequestType::Unknown,
            request_type: RequestType::Unknown,
            duration: Duration::new(0, 0),
            keys: Vec::new(),
        }
    }

    // pub fn set_req_type(&mut self, r: OSPRequestType) {
    //     self.request_type = r;
    // }
    pub fn set_req_type(&mut self, r: RequestType) {
        self.request_type = r;
    }

    pub fn to_file(&self, file: &Path) {
        let writer = std::fs::File::create(file).unwrap();
        serde_json::to_writer(writer, self).ok();
    }

    /// Does a forward-scan of nodes for the node with the given trace_id
    // pub fn can_reach_from_node(&self, trace_id: Uuid, nidx: NodeIndex) -> bool {
    pub fn can_reach_from_node(&self, trace_id: IDType, nidx: NodeIndex) -> bool {
        let mut cur_nidx = nidx;
        loop {
            if self.g[cur_nidx].trace_id == trace_id {
                return true;
            }
            let next_nids = self
                .g
                .neighbors_directed(cur_nidx, Direction::Outgoing)
                .collect::<Vec<_>>();
            if next_nids.len() == 0 {
                return false;
            } else if next_nids.len() == 1 {
                cur_nidx = next_nids[0];
            } else {
                for next_nidx in next_nids {
                    if self.can_reach_from_node(trace_id.clone(), next_nidx) {
                        return true;
                    }
                }
                return false;
            }
        }
    }

    /// Return nodes with outdegree == 0
    pub fn possible_end_nodes(&self) -> Vec<NodeIndex> {
        let mut result = Vec::new();
        for i in self.g.node_indices() {
            if self.g.neighbors_directed(i, Direction::Outgoing).count() == 0 {
                result.push(i);
            }
        }
        result
    }

    fn _get_start_end_nodes(&self) -> (NodeIndex, NodeIndex) {
        let mut smallest_time =
            NaiveDateTime::parse_from_str("3000/01/01 01:01", "%Y/%m/%d %H:%M").unwrap();
        let mut largest_time =
            NaiveDateTime::parse_from_str("1000/01/01 01:01", "%Y/%m/%d %H:%M").unwrap();
        let mut start = NodeIndex::end();
        let mut end = NodeIndex::end();
        for i in self.g.node_indices() {
            if self.g[i].timestamp > largest_time {
                end = i;
                largest_time = self.g[i].timestamp;
            }
            if self.g[i].timestamp < smallest_time {
                start = i;
                smallest_time = self.g[i].timestamp;
            }
        }
        (start, end)
    }

    /// Remove branches that do not end in the ending node
    pub fn prune(&mut self) {
        let mut removed_count = 0;
        loop {
            let mut iter = self.g.externals(Direction::Outgoing);
            let mut end_node = match iter.next() {
                Some(nidx) => nidx,
                None => {
                    break;
                }
            };
            if end_node == self.end_node {
                end_node = match iter.next() {
                    None => {
                        break;
                    }
                    Some(n) => n,
                };
            }
            let mut cur_nodes = vec![end_node];
            loop {
                let cur_node = match cur_nodes.pop() {
                    None => {
                        break;
                    }
                    Some(i) => i,
                };
                let out_neighbors = self
                    .g
                    .neighbors_directed(cur_node, Direction::Outgoing)
                    .collect::<Vec<_>>();
                if out_neighbors.len() >= 1 {
                    continue;
                }
                let neighbors = self
                    .g
                    .neighbors_directed(cur_node, Direction::Incoming)
                    .collect::<Vec<_>>();
                self.g.remove_node(cur_node);
                removed_count += 1;
                for n in neighbors {
                    cur_nodes.push(n);
                }
            }
        }
        eprintln!("Removed {} nodes when pruning", removed_count);
    }

    pub fn get_keys(&self) {
        for node in self.g.node_indices() {
            self.g[node].print_key_values();
        }
    }

    pub fn get_edges(&self) -> Vec<TraceEdge> {
        let edge_indices = self.g.edge_indices();
        let edge_endpoints : Vec<(NodeIndex, NodeIndex, EdgeIndex)> = edge_indices.into_iter().map(
            | ei | -> (NodeIndex, NodeIndex, EdgeIndex) {
                let endpoints = self.g.edge_endpoints(ei).unwrap();
                return (endpoints.0, endpoints.1, ei.clone());
            }
        ).collect();
        return edge_endpoints.into_iter().map(
            | nne | -> TraceEdge {
                let node1 = self.g.node_weight(nne.0).unwrap();
                let node2 = self.g.node_weight(nne.1).unwrap();
                // let edge = self.g.edge_weight(nne.2).unwrap();
                let (tid1, tp1, tt1) = (
                    node1.tracepoint_id,
                    node1.tracepoint_id.to_string(),
                    node1.timestamp.timestamp_nanos()
                );
                let (tid2, tp2, tt2) = (
                    node2.tracepoint_id,
                    node2.tracepoint_id.to_string(),
                    node2.timestamp.timestamp_nanos()
                );
                println!();
                println!("EVENT HOST:");
                println!("{}", node2.tracepoint_id);
                println!("{:?}", node2.key_value_pair.get("host"));
                println!();
                TraceEdge {
                    // uuid: self.base_id,
                    id: self.base_id.clone(),
                    request_type: self.request_type.clone(),
                    tid_start: tid1,
                    tp_start: tp1,
                    start: tt1,
                    tid_end: tid2,
                    tp_end: tp2,
                    end: tt2,
                }
            }
        ).collect();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TraceEdge {
    // pub uuid : Uuid,
    pub id : IDType,
    // pub request_type : OSPRequestType,
    pub request_type : RequestType,
    pub tid_start : TracepointID,
    pub tp_start : String,
    pub start : i64,
    pub tid_end : TracepointID,
    pub tp_end : String,
    pub end : i64,
}

impl TraceEdge {
    pub fn overlaps_with(&self, te: &TraceEdge) -> bool {
        // println!();
        // println!("Checking for overlap!");
        // println!("{}  ||  {}", self.start, self.end);
        // println!("{}  ||  {}", te.start, te.end);
        // println!();
        (self.start < te.end) && (self.end > te.start)
    }
}

impl Event {
    pub fn print_key_values(&self) {
        println!("{:?}", self.key_value_pair);
    }
}
impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // let d = Dot::new(&self.g);
        // write!(f, "{:?}", d)
        write!(f, "{:?}", self.g)
    }
}

/// Events contain trace and tracepoint IDs, as well as timestamps and KV pairs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    // A trace id is shared between two ends of a span, otherwise it should be unique to events
    // pub trace_id: Uuid,
    pub trace_id: IDType,
    // A tracepoint id represents a place in code
    pub tracepoint_id: TracepointID,
    pub timestamp: NaiveDateTime,
    // Synthetic nodes are added to preserve the hierarchy, they are
    // not actual events that happened
    pub is_synthetic: bool,
    pub variant: EventType,
    pub key_value_pair: HashMap<String, Value>,
   // pub variance: f64,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, Eq, PartialEq)]
pub enum EventType {
    Entry,
    Exit,
    /// Annotations are free-standing events that are not part of a span
    Annotation,
}

impl Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.variant {
            EventType::Entry => write!(f, "{} start: {}", self.trace_id, self.tracepoint_id),
            EventType::Annotation => write!(f, "{}: {}", self.trace_id, self.tracepoint_id),
            EventType::Exit => write!(f, "{} end", self.trace_id),
        }
    }
}

impl PartialEq<Event> for Event {
    fn eq(&self, other: &Event) -> bool {
        self.tracepoint_id == other.tracepoint_id && self.variant == other.variant
    }
}

/// A DAGEdge contains a duration and an edge type (child-of or follows-from)
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DAGEdge {
    pub duration: Duration,
    pub variant: EdgeType,
    pub service: Option<String>,
    pub host: Option<String>
}

/// These edge types are taken from OpenTracing, but they are not used much in the codebase
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum EdgeType {
    ChildOf,
    FollowsFrom,
}

impl Display for DAGEdge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.variant {
            EdgeType::ChildOf => write!(f, "{}: C", self.duration.as_nanos()),
            EdgeType::FollowsFrom => write!(f, "{}: F", self.duration.as_nanos()),
        }
    }
}

/// A trace node is an abstract node, so it doesn't have a timestamp or trace id, it just has a
/// tracepoint id and variant.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TraceNode {
    pub tracepoint_id: TracepointID,
    pub variant: EventType,
    pub key_value_pair: HashMap<String, Vec<Value>>,
   // pub variance: f64,
}

impl PartialEq for TraceNode {
    fn eq(&self, other: &Self) -> bool {
        self.tracepoint_id == other.tracepoint_id && self.variant == other.variant
    }
}

impl Eq for TraceNode {}

impl TraceNode {
    //building hashmap that contains key value pairs where key is a string, and value is a vector
    //of Value
    pub fn from_event(event: &Event) -> Self {
        let mut map = HashMap::new();
        let mut vec_value: Vec<Value> = Vec::new();
        let mut vec_host: Vec<Value> = Vec::new();
        let mut vec_agent: Vec<Value> = Vec::new();
        let mut vec_hrt: Vec<Value> = Vec::new();
        let mut vec_proc_id: Vec<Value> = Vec::new();
        let mut vec_proc_name: Vec<Value> = Vec::new();
        let mut vec_thread_id: Vec<Value> = Vec::new();
        let mut vec_thread_name: Vec<Value> = Vec::new();
        for (key, value) in event.key_value_pair.clone() {
            if key == "lock_queue".to_string() {
                vec_value.push(value);
            } else if key == "host".to_string() {
                vec_host.push(value);
            } else if key == "agent".to_string() {
                vec_agent.push(value);
            } else if key == "hrt".to_string() {
                vec_hrt.push(value);
            } else if key == "process ID".to_string() {
                vec_proc_id.push(value);
            } else if key == "process name" {
                vec_proc_name.push(value);
            } else if key == "thread ID" {
                vec_thread_id.push(value);
            } else if key == "thread name" {
                vec_thread_name.push(value);
            }
        }

        map.insert("lock_queue".to_string(), vec_value);
        map.insert("host".to_string(), vec_host);

       // let mut var = variance()
        TraceNode {
            tracepoint_id: event.tracepoint_id,
            variant: event.variant,
            key_value_pair: map,
           // variance: event.pairs_variance(),
        }
    }
/*
    pub fn pairs_variance(event: &Event) -> f64 {
        let mut varian;
        for (key, value) in event.key_value_pair.clone() {
            varian = variance(event.value.iter().map(|x| x.duration.as_nanos()));
        }
          varian = variance(event.key_value_pair.iter().map(|x| x.duration.as_nanos()));
        return varian;
    }*/
/*
    pub fn get_key_values() -> HashMap<String, Vec<Value>> {
        return Self{key_value_pair};
    } */
}

impl Display for TraceNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.variant {
            EventType::Entry => write!(f, "{}: start", self.tracepoint_id),
            EventType::Exit => write!(f, "{}: end", self.tracepoint_id),
            EventType::Annotation => write!(f, "{}", self.tracepoint_id),
        }
    }
}

impl PartialEq<TraceNode> for Event {
    fn eq(&self, other: &TraceNode) -> bool {
        self.tracepoint_id == other.tracepoint_id && self.variant == other.variant
    }
}

lazy_static! {
    static ref TRACEPOINT_ID_MAP: Mutex<BiMap<String, usize>> = Mutex::new(BiMap::new());
}

/// We do some tricks to keep tracepoint ids as `usize`s so it uses less memory than strings.
#[derive(Hash, Clone, Copy, Eq, PartialEq)]
pub struct TracepointID {
    id: usize,
}

impl TracepointID {
    pub fn to_string(&self) -> String {
        TRACEPOINT_ID_MAP
            .lock()
            .unwrap()
            .get_by_right(&self.id)
            .unwrap()
            .clone()
    }

    pub fn from_str(s: &str) -> Self {
        let mut map = TRACEPOINT_ID_MAP.lock().unwrap();
        match map.get_by_left(&s.to_string()) {
            Some(&id) => Self { id: id },
            None => {
                let id = map.len();
                map.insert(s.to_string(), id);
                Self { id: id }
            }
        }
    }

    pub fn bytes(&self) -> [u8; 8] {
        self.id.to_ne_bytes()
    }
}

impl Display for TracepointID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Debug for TracepointID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TracepointID")
            .field("id", &self.id)
            .field("full_name", &self.to_string())
            .finish()
    }
}

impl Serialize for TracepointID {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

struct TracepointIDVisitor;

impl<'de> de::Visitor<'de> for TracepointIDVisitor {
    type Value = TracepointID;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string representing a tracepoint id")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TracepointID::from_str(s))
    }
}

impl<'de> Deserialize<'de> for TracepointID {
    fn deserialize<D>(d: D) -> Result<TracepointID, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(TracepointIDVisitor)
    }
}
