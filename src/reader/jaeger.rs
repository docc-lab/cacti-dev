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
struct JaegerSpan {
    traceID: String,
    spanID: String,
    flags: i32,
    operationName: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerTrace {
    traceID: String,
    spans: Vec<JaegerSpan>
    // processes: Vec:
}

#[derive(Debug, Serialize, Deserialize)]
struct JaegerPayload {
    data: Vec<JaegerSpan>,
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
    fn get_recent_traces(&mut self) -> Vec<Trace> {
        // let mut ids = Vec::new();

        let mut traces: HashMap<String, Vec<Span>> = HashMap::new();

        // let resp: reqwest::blocking::Response = reqwest::blocking::get("https://httpbin.org/ip").unwrap();
        let resp: reqwest::blocking::Response =
            reqwest::blocking::get(self.fetch_url.clone() + "/api/traces?service=nginx-web-server&limit=10")
                .unwrap();

        // match resp.text() {
        //     Ok(res) => {
        //         eprintln!("RESPONSE = {:?}", resp.text());
        //     }
        // }

        let resp_text = resp.text();

        // let resp_obj: JaegerPayload =
        //     serde_json::from_str(
        //         (resp_text.unwrap() as String).as_str()).unwrap();

        let static_resp_text = r#"
        {
  "data": [
    {
      "traceID": "0db1191b4e3bb3a0",
      "spans": [
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "1b48999a9ebb3eea",
          "flags": 1,
          "operationName": "read_home_timeline_redis_find_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "aa833c45e2372b21"
            }
          ],
          "startTime": 1713382467579378,
          "duration": 166,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p1",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "aa833c45e2372b21",
          "flags": 1,
          "operationName": "read_home_timeline_server",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "be30b0e65c8f4fe2"
            }
          ],
          "startTime": 1713382467579365,
          "duration": 1359,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p1",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "4f2fa980e18c05a9",
          "flags": 1,
          "operationName": "post_storage_mmc_mget_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "4b4cff74948349ec"
            }
          ],
          "startTime": 1713382467579854,
          "duration": 576,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p2",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "4b4cff74948349ec",
          "flags": 1,
          "operationName": "post_storage_read_posts_server",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "aa833c45e2372b21"
            }
          ],
          "startTime": 1713382467579651,
          "duration": 933,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p2",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "be30b0e65c8f4fe2",
          "flags": 1,
          "operationName": "read_home_timeline_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "6b18a9b3f4ad7313"
            }
          ],
          "startTime": 1713382467579131,
          "duration": 5486,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "6b18a9b3f4ad7313",
          "flags": 1,
          "operationName": "/wrk2-api/home-timeline/read",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "0db1191b4e3bb3a0"
            }
          ],
          "startTime": 1713382467579059,
          "duration": 5593,
          "tags": [
            {
              "key": "component",
              "type": "string",
              "value": "nginx"
            },
            {
              "key": "nginx.worker_pid",
              "type": "string",
              "value": "9"
            },
            {
              "key": "peer.address",
              "type": "string",
              "value": "172.19.0.1:35116"
            },
            {
              "key": "http.method",
              "type": "string",
              "value": "GET"
            },
            {
              "key": "http.url",
              "type": "string",
              "value": "http://localhost:8080/wrk2-api/home-timeline/read?user_id=15&start=43&stop=53"
            },
            {
              "key": "http.host",
              "type": "string",
              "value": "localhost:8080"
            },
            {
              "key": "http.status_code",
              "type": "int64",
              "value": 200
            },
            {
              "key": "http.status_line",
              "type": "string",
              "value": ""
            },
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "0db1191b4e3bb3a0",
          "flags": 1,
          "operationName": "/wrk2-api/home-timeline/read",
          "references": [],
          "startTime": 1713382467578000,
          "duration": 6651,
          "tags": [
            {
              "key": "sampler.type",
              "type": "string",
              "value": "probabilistic"
            },
            {
              "key": "sampler.param",
              "type": "float64",
              "value": 0.2
            },
            {
              "key": "http.status_code",
              "type": "int64",
              "value": 200
            },
            {
              "key": "http.status_line",
              "type": "string",
              "value": ""
            },
            {
              "key": "component",
              "type": "string",
              "value": "nginx"
            },
            {
              "key": "nginx.worker_pid",
              "type": "string",
              "value": "9"
            },
            {
              "key": "peer.address",
              "type": "string",
              "value": "172.19.0.1:35116"
            },
            {
              "key": "http.method",
              "type": "string",
              "value": "GET"
            },
            {
              "key": "http.url",
              "type": "string",
              "value": "http://localhost:8080/wrk2-api/home-timeline/read?user_id=15&start=43&stop=53"
            },
            {
              "key": "http.host",
              "type": "string",
              "value": "localhost:8080"
            },
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        }
      ],
      "processes": {
        "p1": {
          "serviceName": "home-timeline-service",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "home-timeline-service"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        },
        "p2": {
          "serviceName": "post-storage-service",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "post-storage-service"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        },
        "p3": {
          "serviceName": "nginx-web-server",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "nginx-thrift"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        }
      },
      "warnings": null
    },
    {
      "traceID": "0c7996e1fdf4f54c",
      "spans": [
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "003b77ea7fae5361",
          "flags": 1,
          "operationName": "read_home_timeline_redis_find_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "d918f7a03fe2298f"
            }
          ],
          "startTime": 1713382468245494,
          "duration": 157,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p1",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "d918f7a03fe2298f",
          "flags": 1,
          "operationName": "read_home_timeline_server",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "6fc286da6f0673c9"
            }
          ],
          "startTime": 1713382468245480,
          "duration": 351,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p1",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "387f09aa13907219",
          "flags": 1,
          "operationName": "post_storage_read_posts_server",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "d918f7a03fe2298f"
            }
          ],
          "startTime": 1713382468245753,
          "duration": 9,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p2",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "6fc286da6f0673c9",
          "flags": 1,
          "operationName": "read_home_timeline_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "4ed5a3cde246374e"
            }
          ],
          "startTime": 1713382468245248,
          "duration": 718,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "4ed5a3cde246374e",
          "flags": 1,
          "operationName": "/wrk2-api/home-timeline/read",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "0c7996e1fdf4f54c"
            }
          ],
          "startTime": 1713382468245175,
          "duration": 852,
          "tags": [
            {
              "key": "component",
              "type": "string",
              "value": "nginx"
            },
            {
              "key": "nginx.worker_pid",
              "type": "string",
              "value": "9"
            },
            {
              "key": "peer.address",
              "type": "string",
              "value": "172.19.0.1:35116"
            },
            {
              "key": "http.method",
              "type": "string",
              "value": "GET"
            },
            {
              "key": "http.url",
              "type": "string",
              "value": "http://localhost:8080/wrk2-api/home-timeline/read?user_id=64&start=96&stop=106"
            },
            {
              "key": "http.host",
              "type": "string",
              "value": "localhost:8080"
            },
            {
              "key": "http.status_code",
              "type": "int64",
              "value": 200
            },
            {
              "key": "http.status_line",
              "type": "string",
              "value": ""
            },
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "0c7996e1fdf4f54c",
          "flags": 1,
          "operationName": "/wrk2-api/home-timeline/read",
          "references": [],
          "startTime": 1713382468245000,
          "duration": 1025,
          "tags": [
            {
              "key": "sampler.type",
              "type": "string",
              "value": "probabilistic"
            },
            {
              "key": "sampler.param",
              "type": "float64",
              "value": 0.2
            },
            {
              "key": "http.status_code",
              "type": "int64",
              "value": 200
            },
            {
              "key": "http.status_line",
              "type": "string",
              "value": ""
            },
            {
              "key": "component",
              "type": "string",
              "value": "nginx"
            },
            {
              "key": "nginx.worker_pid",
              "type": "string",
              "value": "9"
            },
            {
              "key": "peer.address",
              "type": "string",
              "value": "172.19.0.1:35116"
            },
            {
              "key": "http.method",
              "type": "string",
              "value": "GET"
            },
            {
              "key": "http.url",
              "type": "string",
              "value": "http://localhost:8080/wrk2-api/home-timeline/read?user_id=64&start=96&stop=106"
            },
            {
              "key": "http.host",
              "type": "string",
              "value": "localhost:8080"
            },
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        }
      ],
      "processes": {
        "p1": {
          "serviceName": "home-timeline-service",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "home-timeline-service"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        },
        "p2": {
          "serviceName": "post-storage-service",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "post-storage-service"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        },
        "p3": {
          "serviceName": "nginx-web-server",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "nginx-thrift"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        }
      },
      "warnings": null
    }
  ],
  "total": 0,
  "limit": 0,
  "offset": 0,
  "errors": null
}
        "#;

        let static_resp_text_2 = r#"[
    {
      "traceID": "0db1191b4e3bb3a0",
      "spans": [
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "1b48999a9ebb3eea",
          "flags": 1,
          "operationName": "read_home_timeline_redis_find_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "aa833c45e2372b21"
            }
          ],
          "startTime": 1713382467579378,
          "duration": 166,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p1",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "aa833c45e2372b21",
          "flags": 1,
          "operationName": "read_home_timeline_server",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "be30b0e65c8f4fe2"
            }
          ],
          "startTime": 1713382467579365,
          "duration": 1359,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p1",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "4f2fa980e18c05a9",
          "flags": 1,
          "operationName": "post_storage_mmc_mget_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "4b4cff74948349ec"
            }
          ],
          "startTime": 1713382467579854,
          "duration": 576,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p2",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "4b4cff74948349ec",
          "flags": 1,
          "operationName": "post_storage_read_posts_server",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "aa833c45e2372b21"
            }
          ],
          "startTime": 1713382467579651,
          "duration": 933,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p2",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "be30b0e65c8f4fe2",
          "flags": 1,
          "operationName": "read_home_timeline_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "6b18a9b3f4ad7313"
            }
          ],
          "startTime": 1713382467579131,
          "duration": 5486,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "6b18a9b3f4ad7313",
          "flags": 1,
          "operationName": "/wrk2-api/home-timeline/read",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0db1191b4e3bb3a0",
              "spanID": "0db1191b4e3bb3a0"
            }
          ],
          "startTime": 1713382467579059,
          "duration": 5593,
          "tags": [
            {
              "key": "component",
              "type": "string",
              "value": "nginx"
            },
            {
              "key": "nginx.worker_pid",
              "type": "string",
              "value": "9"
            },
            {
              "key": "peer.address",
              "type": "string",
              "value": "172.19.0.1:35116"
            },
            {
              "key": "http.method",
              "type": "string",
              "value": "GET"
            },
            {
              "key": "http.url",
              "type": "string",
              "value": "http://localhost:8080/wrk2-api/home-timeline/read?user_id=15&start=43&stop=53"
            },
            {
              "key": "http.host",
              "type": "string",
              "value": "localhost:8080"
            },
            {
              "key": "http.status_code",
              "type": "int64",
              "value": 200
            },
            {
              "key": "http.status_line",
              "type": "string",
              "value": ""
            },
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        },
        {
          "traceID": "0db1191b4e3bb3a0",
          "spanID": "0db1191b4e3bb3a0",
          "flags": 1,
          "operationName": "/wrk2-api/home-timeline/read",
          "references": [],
          "startTime": 1713382467578000,
          "duration": 6651,
          "tags": [
            {
              "key": "sampler.type",
              "type": "string",
              "value": "probabilistic"
            },
            {
              "key": "sampler.param",
              "type": "float64",
              "value": 0.2
            },
            {
              "key": "http.status_code",
              "type": "int64",
              "value": 200
            },
            {
              "key": "http.status_line",
              "type": "string",
              "value": ""
            },
            {
              "key": "component",
              "type": "string",
              "value": "nginx"
            },
            {
              "key": "nginx.worker_pid",
              "type": "string",
              "value": "9"
            },
            {
              "key": "peer.address",
              "type": "string",
              "value": "172.19.0.1:35116"
            },
            {
              "key": "http.method",
              "type": "string",
              "value": "GET"
            },
            {
              "key": "http.url",
              "type": "string",
              "value": "http://localhost:8080/wrk2-api/home-timeline/read?user_id=15&start=43&stop=53"
            },
            {
              "key": "http.host",
              "type": "string",
              "value": "localhost:8080"
            },
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        }
      ],
      "processes": {
        "p1": {
          "serviceName": "home-timeline-service",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "home-timeline-service"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        },
        "p2": {
          "serviceName": "post-storage-service",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "post-storage-service"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        },
        "p3": {
          "serviceName": "nginx-web-server",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "nginx-thrift"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        }
      },
      "warnings": null
    },
    {
      "traceID": "0c7996e1fdf4f54c",
      "spans": [
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "003b77ea7fae5361",
          "flags": 1,
          "operationName": "read_home_timeline_redis_find_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "d918f7a03fe2298f"
            }
          ],
          "startTime": 1713382468245494,
          "duration": 157,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p1",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "d918f7a03fe2298f",
          "flags": 1,
          "operationName": "read_home_timeline_server",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "6fc286da6f0673c9"
            }
          ],
          "startTime": 1713382468245480,
          "duration": 351,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p1",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "387f09aa13907219",
          "flags": 1,
          "operationName": "post_storage_read_posts_server",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "d918f7a03fe2298f"
            }
          ],
          "startTime": 1713382468245753,
          "duration": 9,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p2",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "6fc286da6f0673c9",
          "flags": 1,
          "operationName": "read_home_timeline_client",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "4ed5a3cde246374e"
            }
          ],
          "startTime": 1713382468245248,
          "duration": 718,
          "tags": [
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "4ed5a3cde246374e",
          "flags": 1,
          "operationName": "/wrk2-api/home-timeline/read",
          "references": [
            {
              "refType": "CHILD_OF",
              "traceID": "0c7996e1fdf4f54c",
              "spanID": "0c7996e1fdf4f54c"
            }
          ],
          "startTime": 1713382468245175,
          "duration": 852,
          "tags": [
            {
              "key": "component",
              "type": "string",
              "value": "nginx"
            },
            {
              "key": "nginx.worker_pid",
              "type": "string",
              "value": "9"
            },
            {
              "key": "peer.address",
              "type": "string",
              "value": "172.19.0.1:35116"
            },
            {
              "key": "http.method",
              "type": "string",
              "value": "GET"
            },
            {
              "key": "http.url",
              "type": "string",
              "value": "http://localhost:8080/wrk2-api/home-timeline/read?user_id=64&start=96&stop=106"
            },
            {
              "key": "http.host",
              "type": "string",
              "value": "localhost:8080"
            },
            {
              "key": "http.status_code",
              "type": "int64",
              "value": 200
            },
            {
              "key": "http.status_line",
              "type": "string",
              "value": ""
            },
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        },
        {
          "traceID": "0c7996e1fdf4f54c",
          "spanID": "0c7996e1fdf4f54c",
          "flags": 1,
          "operationName": "/wrk2-api/home-timeline/read",
          "references": [],
          "startTime": 1713382468245000,
          "duration": 1025,
          "tags": [
            {
              "key": "sampler.type",
              "type": "string",
              "value": "probabilistic"
            },
            {
              "key": "sampler.param",
              "type": "float64",
              "value": 0.2
            },
            {
              "key": "http.status_code",
              "type": "int64",
              "value": 200
            },
            {
              "key": "http.status_line",
              "type": "string",
              "value": ""
            },
            {
              "key": "component",
              "type": "string",
              "value": "nginx"
            },
            {
              "key": "nginx.worker_pid",
              "type": "string",
              "value": "9"
            },
            {
              "key": "peer.address",
              "type": "string",
              "value": "172.19.0.1:35116"
            },
            {
              "key": "http.method",
              "type": "string",
              "value": "GET"
            },
            {
              "key": "http.url",
              "type": "string",
              "value": "http://localhost:8080/wrk2-api/home-timeline/read?user_id=64&start=96&stop=106"
            },
            {
              "key": "http.host",
              "type": "string",
              "value": "localhost:8080"
            },
            {
              "key": "internal.span.format",
              "type": "string",
              "value": "proto"
            }
          ],
          "logs": [],
          "processID": "p3",
          "warnings": null
        }
      ],
      "processes": {
        "p1": {
          "serviceName": "home-timeline-service",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "home-timeline-service"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        },
        "p2": {
          "serviceName": "post-storage-service",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "post-storage-service"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        },
        "p3": {
          "serviceName": "nginx-web-server",
          "tags": [
            {
              "key": "hostname",
              "type": "string",
              "value": "nginx-thrift"
            },
            {
              "key": "ip",
              "type": "string",
              "value": "127.0.0.1"
            },
            {
              "key": "jaeger.version",
              "type": "string",
              "value": "C++-0.4.2"
            }
          ]
        }
      },
      "warnings": null
    }
  ]"#;

        // let resp_obj: JaegerPayload =
        //     serde_json::from_str(static_resp_text).unwrap();

        let resp_obj: Vec<JaegerTrace> =
            serde_json::from_str(static_resp_text_2).unwrap();

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