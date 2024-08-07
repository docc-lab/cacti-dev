/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

//! Methods that talk to Pythia agents.

use futures::future::Future;
use futures::stream::Stream;
use futures::Async;
use hyper::rt;
use jsonrpc_client_transports::transports::http;
use jsonrpc_core::Value;
use jsonrpc_core_client::{RpcChannel, RpcError, TypedClient};
use serde_json;
use uuid::Uuid;

use pythia_common::{NodeStats, RequestType};
use pythia_common::OSProfilerSpan;
use pythia_common::OSPRequestType;

use crate::trace::TracepointID;

#[derive(Clone)]
struct PythiaClient(TypedClient);

impl From<RpcChannel> for PythiaClient {
    fn from(channel: RpcChannel) -> Self {
        PythiaClient(channel.into())
    }
}

impl PythiaClient {
    fn get_events(&self, trace_id: String) -> impl Future<Item = Value, Error = RpcError> {
        self.0.call_method("get_events", "String", (trace_id,))
    }

    fn set_all_tracepoints(&self, to_write: [u8; 1]) -> impl Future<Item = (), Error = RpcError> {
        eprintln!("to_write_1: {:?}", to_write);
        self.0.call_method("set_all_tracepoints", "", (to_write,))
    }

    fn set_tracepoints(
        &self,
        // settings: Vec<(TracepointID, Option<OSPRequestType>, [u8; 1])>,
        settings: Vec<(TracepointID, Option<RequestType>, [u8; 1])>,
    ) -> impl Future<Item = (), Error = RpcError> {
        // let new_settings: Vec<(String, Option<OSPRequestType>, [u8; 1])> = settings
        let new_settings: Vec<(String, Option<RequestType>, [u8; 1])> = settings
            .iter()
            .map(|(x, y, z)| (x.to_string(), y.clone(), z.clone()))
            .collect();
        self.0.call_method("set_tracepoints", "", (new_settings,))
    }

    fn read_node_stats(&self) -> impl Future<Item = NodeStats, Error = RpcError> {
        self.0.call_method("read_node_stats", "", ())
    }

    fn free_keys(&self, keys: Vec<String>) -> impl Future<Item = (), Error = RpcError> {
        self.0.call_method("free_keys", "", (keys,))
    }

    //////////
    // New section - tracepoint enabling/disabling
    //////////
    // fn enable_disable_tracepoints(
    //     &self,
    //     settings: Vec<(TracepointID, Option<RequestType>, [u8; 1])>,
    // ) -> impl Future<Item = (), Error = RpcError> {
    //
    // }
}

/// Read the overhead stats from the agent
pub fn read_client_stats(client_uri: &str) -> NodeStats {
    let (tx, mut rx) = futures::sync::mpsc::unbounded();

    let run = http::connect(client_uri)
        .and_then(move |client: PythiaClient| {
            client.read_node_stats().and_then(move |result| {
                drop(client);
                let _ = tx.unbounded_send(result);
                Ok(())
            })
        })
        .map_err(|e| eprintln!("RPC Client error: {:?}", e));

    rt::run(run);

    loop {
        match rx.poll() {
            Ok(Async::Ready(Some(v))) => {
                return v;
            }
            Ok(Async::NotReady) => {}
            Ok(Async::Ready(None)) => panic!("Got nothing from request"),
            Err(e) => panic!("Got error from poll: {:?}", e),
        }
    }
}

/// Get events matching the trace_id. OSProfiler-specific
pub fn get_events_from_client(client_uri: &str, trace_id: Uuid) -> Vec<OSProfilerSpan> {
    let (tx, mut rx) = futures::sync::mpsc::unbounded();

    let run = http::connect(client_uri)
        .and_then(move |client: PythiaClient| {
            client
                .get_events(trace_id.hyphenated().to_string())
                .and_then(move |result| {
                    drop(client);
                    let _ = tx.unbounded_send(result);
                    Ok(())
                })
        })
        .map_err(|e| eprintln!("RPC Client error: {:?}", e));

    rt::run(run);
    let mut final_result = Vec::new();

    loop {
        match rx.poll() {
            Ok(Async::Ready(Some(v))) => {
                let traces = match v {
                    Value::Array(o) => o,
                    _ => panic!("Got something weird from request {:?}", v),
                };
                final_result.extend(
                    traces
                        .iter()
                        .map(|x| x.to_string())
                        .map(|x: String| serde_json::from_str(&x).unwrap())
                        .collect::<Vec<OSProfilerSpan>>(),
                );
            }
            Ok(Async::NotReady) => {}
            Ok(Async::Ready(None)) => {
                break;
            }
            Err(e) => panic!("Got error from poll: {:?}", e),
        }
    }
    final_result
}

/// Used by controller
pub fn set_all_client_tracepoints(client_uri: &str, to_write: [u8; 1]) {
    let (tx, mut rx) = futures::sync::mpsc::unbounded();
    eprintln!("to_write_0: {:?}", to_write);
    let run = http::connect(client_uri)
        .and_then(move |client: PythiaClient| {
            client
                .set_all_tracepoints(to_write.clone())
                .and_then(move |x| {
                    drop(client);
                    let _ = tx.unbounded_send(x);
                    Ok(())
                })
        })
        .map_err(|e| eprintln!("RPC Client error: {:?}", e));

    rt::run(run);
    loop {
        match rx.poll() {
            Ok(Async::Ready(Some(()))) => {
                return;
            }
            Ok(Async::NotReady) => {}
            Ok(Async::Ready(None)) => {
                break;
            }
            Err(e) => panic!("Got error from poll: {:?}", e),
        }
    }
}

/// Used by controller
pub fn set_client_tracepoints(
    client_uri: &str,
    // settings: Vec<(TracepointID, Option<OSPRequestType>, [u8; 1])>,
    settings: Vec<(TracepointID, Option<RequestType>, [u8; 1])>,
) {
    let (tx, mut rx) = futures::sync::mpsc::unbounded();

    // Connect to client and set tracepoints
    // TODO: Figure out what "move" does
    // TODO: console log variables to see what they are and where they come from
    let run = http::connect(client_uri)
        .and_then(move |client: PythiaClient| {
            client.set_tracepoints(settings).and_then(move |x| {
                drop(client);
                tx.unbounded_send(x).unwrap();
                Ok(())
            })
        })
        .map_err(|e| eprintln!("RPC Client error: {:?}", e));

    rt::run(run);
    loop {
        match rx.poll() {
            Ok(Async::Ready(Some(()))) => {
                return;
            }
            Ok(Async::NotReady) => {}
            Ok(Async::Ready(None)) => {
                break;
            }
            Err(e) => panic!("Got error from poll: {:?}", e),
        }
    }
}

/// Free the used traces from redis so that we don't use too much memory
pub fn free_keys(client_uri: &str, keys: Vec<String>) {
    if keys.len() == 0 {
        return;
    }
    let (tx, mut rx) = futures::sync::mpsc::unbounded();

    let run = http::connect(client_uri)
        .and_then(move |client: PythiaClient| {
            client.free_keys(keys).and_then(move |x| {
                drop(client);
                tx.unbounded_send(x).unwrap();
                Ok(())
            })
        })
        .map_err(|e| eprintln!("RPC Client error: {:?}", e));

    rt::run(run);
    loop {
        match rx.poll() {
            Ok(Async::Ready(Some(()))) => {
                return;
            }
            Ok(Async::NotReady) => {}
            Ok(Async::Ready(None)) => {
                break;
            }
            Err(e) => panic!("Got error from poll: {:?}", e),
        }
    }
}
