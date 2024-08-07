// /*
// This source code is licensed under the BSD-style license found in the
// LICENSE file in the root directory of this source tree.
//
// Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
// All rights reserved.
// */
//
// /// Stuff related to working with osprofiler
// ///
// use std::collections::HashMap;
// use std::fmt;
//
// use chrono::NaiveDateTime;
// use regex::RegexSet;
// use serde::{Deserialize, Serialize};
// use uuid::Uuid;
//
// // use strum::IntoEnumIterator; // 0.17.1
// // use strum_macros::EnumIter; // 0.17.1
//
// use crate::PythiaError;
//
// /// Type of a request.
// ///
// /// It's defined here because for now we only use them for OpenStack.
// #[derive(Serialize, Deserialize, Debug, Copy, Eq, PartialEq, Hash, Clone)]
// pub enum RequestType {
//     ServerCreate,
//     ServerDelete,
//     ServerList,
//     FloatingIPCreate,
//     FloatingIPDelete,
//     FloatingIPList,
//     UsageList,
//     Unknown,
// }
//
// lazy_static! {
//     // The ordering of the below two structures should match each other. If any
//     // tracepoint id in the trace matches any of these regexes, we set the
//     // request type accordingly.
//     pub static ref REQUEST_TYPE_REGEXES: RegexSet = RegexSet::new(&[
//         r"openstackclient\.compute\.v2\.server\.CreateServer\.take_action",
//         r"openstackclient\.compute\.v2\.server\.ListServer\.take_action",
//         r"openstackclient\.compute\.v2\.server\.DeleteServer\.take_action",
//         r"openstackclient\.network\.v2\.floating_ip\.CreateFloatingIP\.take_action_network",
//         r"openstackclient\.network\.v2\.floating_ip\.ListFloatingIP\.take_action_network",
//         r"openstackclient\.network\.v2\.floating_ip\.DeleteFloatingIP\.take_action_network",
//         r"novaclient\.v2\.usage\.UsageManager\.list",
//     ])
//     .unwrap();
//     pub static ref REQUEST_TYPES: Vec<RequestType> = vec![
//         RequestType::ServerCreate,
//         RequestType::ServerList,
//         RequestType::ServerDelete,
//         RequestType::FloatingIPCreate,
//         RequestType::FloatingIPList,
//         RequestType::FloatingIPDelete,
//         RequestType::UsageList,
//     ];
// }
//
// impl RequestType {
//     pub fn from_str(typ: &str) -> Result<RequestType, &str> {
//         match typ {
//             "ServerCreate" => Ok(RequestType::ServerCreate),
//             "ServerDelete" => Ok(RequestType::ServerDelete),
//             "ServerList" => Ok(RequestType::ServerList),
//             "FloatingIPCreate" => Ok(RequestType::FloatingIPCreate),
//             "FloatingIPDelete" => Ok(RequestType::FloatingIPDelete),
//             "FloatingIPList" => Ok(RequestType::FloatingIPList),
//             "UsageList" => Ok(RequestType::UsageList),
//             "Unknown" => Ok(RequestType::Unknown),
//             _ => Err("Unknown request type"),
//         }
//     }
//
//     pub fn all_types() -> Vec<RequestType> {
//         let mut to_return = Vec::new();
//
//         to_return.push(RequestType::ServerCreate);
//         to_return.push(RequestType::ServerList);
//         to_return.push(RequestType::ServerDelete);
//         to_return.push(RequestType::UsageList);
//         to_return.push(RequestType::Unknown);
//
//         to_return
//     }
// }
//
// impl fmt::Display for RequestType {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
//
// impl OSProfilerSpan {
//     /// We need this method because span endings do not have tracepoint IDs in OSProfiler.
//     ///
//     /// So, we keep track of previous trace ids and tracepoint ids, and for exits we use the hashmap.
//     pub fn get_tracepoint_id(
//         &self,
//         map: &mut HashMap<Uuid, String>,
//     ) -> Result<String, PythiaError> {
//         // The map needs to be initialized and passed to it from outside :(
//         Ok(match &self.info {
//             OSProfilerEnum::FunctionEntry(_) | OSProfilerEnum::RequestEntry(_) => {
//                 map.insert(self.trace_id, self.tracepoint_id.clone());
//                 self.tracepoint_id.clone()
//             }
//             OSProfilerEnum::Annotation(_) => self.tracepoint_id.clone(),
//             OSProfilerEnum::Exit(_) => match map.remove(&self.trace_id) {
//                 Some(s) => s,
//                 None => {
//                     if self.name.starts_with("asynch_wait") {
//                         self.tracepoint_id.clone()
//                     } else {
//                         return Err(PythiaError(format!(
//                             "Couldn't find trace id for {:?}",
//                             self
//                         )));
//                     }
//                 }
//             },
//         })
//     }
// }
//
// /// What an OSProfiler event json has.
// ///
// /// What we collect from redis needs to exactly match this struct, otherwise
// /// it will not be parsed correctly. Most of the info here is thrown out, but
// /// we parse all of it just in case.
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// pub struct OSProfilerSpan {
//     pub trace_id: Uuid,
//     pub parent_id: Uuid,
//     project: String,
//     pub name: String,
//     pub base_id: Uuid,
//     service: String,
//     pub tracepoint_id: String,
//     #[serde(with = "serde_timestamp")]
//     pub timestamp: NaiveDateTime,
//     pub info: OSProfilerEnum,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(untagged)]
// pub enum OSProfilerEnum {
//     Annotation(AnnotationEnum),
//     FunctionEntry(FunctionEntryInfo),
//     RequestEntry(RequestEntryInfo),
//     Exit(ExitEnum),
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(untagged)]
// pub enum AnnotationEnum {
//     WaitFor(WaitAnnotationInfo),
//     Child(ChildAnnotationInfo),
//     Plain(PlainAnnotationInfo),
//     Log(LogAnnotationInfo),
//     KeyValue(KeyValueAnnotationInfo),
//     //the last 3 are for parsing OSProfiler String... needs work
//     Args(ArgsKeyValueInfo),
//     Results(ResultKeyValueInfo),
//     WaitForKeyValue(WaitForKeyValueAnnotation),
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct WaitForKeyValueAnnotation {
//     wait_for: Uuid,
//     function: ArgsKeyValueFunction,
//     tracepoint_id: String,
//     host: String,
//     thread_id: u64,
//     pid: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct ArgsKeyValueInfo {
//     function: ArgsKeyValueFunction,
//     tracepoint_id: String,
//     host: String,
//     thread_id: u64,
//     pid: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct ArgsKeyValueFunction {
//     pub name: String,
//     pub args: String,
//     pub kwargs: String,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct ResultKeyValueInfo {
//     function: ResultFunction,
//     host: String,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct ResultFunction {
//     result: String,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct KeyValueAnnotationInfo {
//     pub value: u64,
//     tracepoint_id: String,
//     pub host: String,
//     thread_id: u64,
//     pid: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct WaitAnnotationInfo {
//     function: FunctionEntryFunction,
//     thread_id: u64,
//     pub host: String,
//     tracepoint_id: String,
//     pid: u64,
//     pub wait_for: Uuid,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct LogAnnotationInfo {
//     thread_id: u64,
//     pub host: String,
//     tracepoint_id: String,
//     pid: u64,
//     msg: String,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct PlainAnnotationInfo {
//     thread_id: u64,
//     pub host: String,
//     tracepoint_id: String,
//     pid: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct ChildAnnotationInfo {
//     thread_id: u64,
//     pub host: String,
//     tracepoint_id: String,
//     pub child_id: Uuid,
//     pid: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct RequestEntryInfo {
//     request: RequestEntryRequest,
//     thread_id: u64,
//     pub host: String,
//     tracepoint_id: String,
//     pid: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// struct RequestEntryRequest {
//     path: String,
//     scheme: String,
//     method: String,
//     query: String,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(untagged)]
// pub enum ExitEnum {
//     Normal(NormalExitInfo),
//     Error(ErrorExitInfo),
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct NormalExitInfo {
//     pub host: String,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct ErrorExitInfo {
//     etype: String,
//     message: String,
//     pub host: String,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// pub struct FunctionEntryInfo {
//     function: FunctionEntryFunction,
//     thread_id: u64,
//     pub host: String,
//     tracepoint_id: String,
//     pid: u64,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
// #[serde(deny_unknown_fields)]
// struct FunctionEntryFunction {
//     name: String,
// }
//
// pub mod serde_timestamp {
//     use chrono::NaiveDateTime;
//     use serde::de;
//     use serde::ser;
//     use std::fmt;
//
//     pub fn deserialize<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
//     where
//         D: de::Deserializer<'de>,
//     {
//         d.deserialize_str(NaiveDateTimeVisitor)
//     }
//
//     pub fn serialize<S>(t: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error>
//     where
//         S: ser::Serializer,
//     {
//         s.serialize_str(&t.format("%Y-%m-%dT%H:%M:%S%.6f").to_string())
//     }
//
//     struct NaiveDateTimeVisitor;
//
//     impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
//         type Value = NaiveDateTime;
//
//         fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//             write!(formatter, "a string represents chrono::NaiveDateTime")
//         }
//
//         fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
//         where
//             E: de::Error,
//         {
//             match NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.6f") {
//                 Ok(t) => Ok(t),
//                 Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(s), &self)),
//             }
//         }
//     }
// }
//
// pub fn parse_field(field: &String) -> Result<OSProfilerSpan, String> {
//     let result: OSProfilerSpan = match serde_json::from_str(field) {
//         Ok(a) => a,
//         Err(e) => {
//             return Err(e.to_string());
//         }
//     };
//     if result.name == "asynch_request" || result.name == "asynch_wait" {
//         return match result.info {
//             OSProfilerEnum::Annotation(_) => Ok(result),
//             _ => {
//                 println!("{:?}", result);
//                 Err("".to_string())
//             }
//         };
//     }
//     Ok(result)
// }
// //testing parsing field
// #[cfg(test)]
// mod tests {
//
//     use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
//
//     use super::*;
//
//     #[test]
//     fn test_parse() {
//         let d = NaiveDate::from_ymd(2020, 06, 23);
//         let t = NaiveTime::from_hms_milli(14, 32, 34, 0058);
//
//         let dt = NaiveDateTime::new(d, t);
//         let y: u64 = 293402358;
//
//         let current_info =
//             OSProfilerEnum::Annotation(AnnotationEnum::KeyValue(KeyValueAnnotationInfo {
//                 value: y,
//                 tracepoint_id: "nova/usr/local".to_string(),
//                 host: "cloudlab".to_string(),
//                 thread_id: 5743728237,
//                 pid: 4771,
//             }));
//
//         let my_uuid = Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap();
//
//         let test_struct = OSProfilerSpan {
//             trace_id: my_uuid,
//             parent_id: my_uuid,
//             project: "nova".to_string(),
//             name: "build_instance".to_string(),
//             base_id: my_uuid,
//             service: "nova".to_string(),
//             tracepoint_id: "nova/manager.py".to_string(),
//             timestamp: dt,
//             info: current_info,
//         };
//
//         //checking if parse_field function works with the added struct to parse the code correctly
//         assert_eq!(parse_field(&(r#"{"trace_id": "936DA01F9ABD4d9d80C702AF85C822A8", "parent_id": "936DA01F9ABD4d9d80C702AF85C822A8", "project": "nova", "name": "build_instance",  "base_id": "936DA01F9ABD4d9d80C702AF85C822A8", "service": "nova", "tracepoint_id": "nova/manager.py", "timestamp": "2020-06-23T14:32:34.058", "info": {"value":293402358,"tracepoint_id": "nova/usr/local", "host": "cloudlab", "thread_id": 5743728237, "pid": 4771}}"#).to_string()),Ok(test_struct));
//     }
// }

/*
This source code is licensed under the BSD-style license found in the
LICENSE file in the root directory of this source tree.

Copyright (c) 2022, Diagnosis and Control of Clouds Laboratory
All rights reserved.
*/

/// Stuff related to working with osprofiler
///
use std::collections::HashMap;
use std::fmt;

use chrono::NaiveDateTime;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// use strum::IntoEnumIterator; // 0.17.1
// use strum_macros::EnumIter; // 0.17.1

use crate::PythiaError;

/// Type of a request.
///
/// It's defined here because for now we only use them for OpenStack.
#[derive(Serialize, Deserialize, Debug, Copy, Eq, PartialEq, Hash, Clone)]
pub enum OSPRequestType {
    ServerCreate,
    ServerDelete,
    ServerList,
    FloatingIPCreate,
    FloatingIPDelete,
    FloatingIPList,
    UsageList,
    Unknown,
}

lazy_static! {
    // The ordering of the below two structures should match each other. If any
    // tracepoint id in the trace matches any of these regexes, we set the
    // request type accordingly.
    pub static ref REQUEST_TYPE_REGEXES: RegexSet = RegexSet::new(&[
        r"openstackclient\.compute\.v2\.server\.CreateServer\.take_action",
        r"openstackclient\.compute\.v2\.server\.ListServer\.take_action",
        r"openstackclient\.compute\.v2\.server\.DeleteServer\.take_action",
        r"openstackclient\.network\.v2\.floating_ip\.CreateFloatingIP\.take_action_network",
        r"openstackclient\.network\.v2\.floating_ip\.ListFloatingIP\.take_action_network",
        r"openstackclient\.network\.v2\.floating_ip\.DeleteFloatingIP\.take_action_network",
        r"novaclient\.v2\.usage\.UsageManager\.list",
    ])
    .unwrap();
    // pub static ref REQUEST_TYPES: Vec<OSPRequestType> = vec![
    pub static ref REQUEST_TYPES: Vec<OSPRequestType> = vec![
        OSPRequestType::ServerCreate,
        OSPRequestType::ServerList,
        OSPRequestType::ServerDelete,
        OSPRequestType::FloatingIPCreate,
        OSPRequestType::FloatingIPList,
        OSPRequestType::FloatingIPDelete,
        OSPRequestType::UsageList,
    ];
}

impl OSPRequestType {
    // pub fn from_str(typ: &str) -> Result<OSPRequestType, &str> {
    pub fn from_str(typ: &str) -> Result<OSPRequestType, String> {
        match typ {
            "ServerCreate" => Ok(OSPRequestType::ServerCreate),
            "ServerDelete" => Ok(OSPRequestType::ServerDelete),
            "ServerList" => Ok(OSPRequestType::ServerList),
            "FloatingIPCreate" => Ok(OSPRequestType::FloatingIPCreate),
            "FloatingIPDelete" => Ok(OSPRequestType::FloatingIPDelete),
            "FloatingIPList" => Ok(OSPRequestType::FloatingIPList),
            "UsageList" => Ok(OSPRequestType::UsageList),
            "Unknown" => Ok(OSPRequestType::Unknown),
            _ => Err(("Unknown request type").to_string()),
        }
    }

    pub fn all_types() -> Vec<OSPRequestType> {
        let mut to_return = Vec::new();

        to_return.push(OSPRequestType::ServerCreate);
        to_return.push(OSPRequestType::ServerList);
        to_return.push(OSPRequestType::ServerDelete);
        to_return.push(OSPRequestType::UsageList);
        to_return.push(OSPRequestType::Unknown);

        to_return
    }
}

impl fmt::Display for OSPRequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl OSProfilerSpan {
    /// We need this method because span endings do not have tracepoint IDs in OSProfiler.
    ///
    /// So, we keep track of previous trace ids and tracepoint ids, and for exits we use the hashmap.
    pub fn get_tracepoint_id(
        &self,
        map: &mut HashMap<Uuid, String>,
    ) -> Result<String, PythiaError> {
        // The map needs to be initialized and passed to it from outside :(
        Ok(match &self.info {
            OSProfilerEnum::FunctionEntry(_) | OSProfilerEnum::RequestEntry(_) => {
                map.insert(self.trace_id, self.tracepoint_id.clone());
                self.tracepoint_id.clone()
            }
            OSProfilerEnum::Annotation(_) => self.tracepoint_id.clone(),
            OSProfilerEnum::Exit(_) => match map.remove(&self.trace_id) {
                Some(s) => s,
                None => {
                    if self.name.starts_with("asynch_wait") {
                        self.tracepoint_id.clone()
                    } else {
                        return Err(PythiaError(format!(
                            "Couldn't find trace id for {:?}",
                            self
                        )));
                    }
                }
            },
        })
    }
}

/// What an OSProfiler event json has.
///
/// What we collect from redis needs to exactly match this struct, otherwise
/// it will not be parsed correctly. Most of the info here is thrown out, but
/// we parse all of it just in case.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct OSProfilerSpan {
    pub trace_id: Uuid,
    pub parent_id: Uuid,
    project: String,
    pub name: String,
    pub base_id: Uuid,
    service: String,
    pub tracepoint_id: String,
    #[serde(with = "serde_timestamp")]
    pub timestamp: NaiveDateTime,
    pub info: OSProfilerEnum,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum OSProfilerEnum {
    Annotation(AnnotationEnum),
    FunctionEntry(FunctionEntryInfo),
    RequestEntry(RequestEntryInfo),
    Exit(ExitEnum),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum AnnotationEnum {
    WaitFor(WaitAnnotationInfo),
    Child(ChildAnnotationInfo),
    Plain(PlainAnnotationInfo),
    Log(LogAnnotationInfo),
    KeyValue(KeyValueAnnotationInfo),
    //the last 3 are for parsing OSProfiler String... needs work
    Args(ArgsKeyValueInfo),
    Results(ResultKeyValueInfo),
    WaitForKeyValue(WaitForKeyValueAnnotation),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WaitForKeyValueAnnotation {
    wait_for: Uuid,
    function: ArgsKeyValueFunction,
    tracepoint_id: String,
    host: String,
    thread_id: u64,
    pid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ArgsKeyValueInfo {
    function: ArgsKeyValueFunction,
    tracepoint_id: String,
    host: String,
    thread_id: u64,
    pid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ArgsKeyValueFunction {
    pub name: String,
    pub args: String,
    pub kwargs: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ResultKeyValueInfo {
    function: ResultFunction,
    host: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ResultFunction {
    result: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct KeyValueAnnotationInfo {
    pub value: u64,
    tracepoint_id: String,
    pub host: String,
    thread_id: u64,
    pid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WaitAnnotationInfo {
    function: FunctionEntryFunction,
    thread_id: u64,
    pub host: String,
    tracepoint_id: String,
    pid: u64,
    pub wait_for: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LogAnnotationInfo {
    thread_id: u64,
    pub host: String,
    tracepoint_id: String,
    pid: u64,
    msg: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlainAnnotationInfo {
    thread_id: u64,
    pub host: String,
    tracepoint_id: String,
    pid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChildAnnotationInfo {
    thread_id: u64,
    pub host: String,
    tracepoint_id: String,
    pub child_id: Uuid,
    pid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RequestEntryInfo {
    request: RequestEntryRequest,
    thread_id: u64,
    pub host: String,
    tracepoint_id: String,
    pid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct RequestEntryRequest {
    path: String,
    scheme: String,
    method: String,
    query: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum ExitEnum {
    Normal(NormalExitInfo),
    Error(ErrorExitInfo),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NormalExitInfo {
    pub host: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ErrorExitInfo {
    etype: String,
    message: String,
    pub host: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FunctionEntryInfo {
    function: FunctionEntryFunction,
    thread_id: u64,
    pub host: String,
    tracepoint_id: String,
    pid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct FunctionEntryFunction {
    name: String,
}

pub mod serde_timestamp {
    use chrono::NaiveDateTime;
    use serde::de;
    use serde::ser;
    use std::fmt;

    pub fn deserialize<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        d.deserialize_str(NaiveDateTimeVisitor)
    }

    pub fn serialize<S>(t: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
    {
        s.serialize_str(&t.format("%Y-%m-%dT%H:%M:%S%.6f").to_string())
    }

    struct NaiveDateTimeVisitor;

    impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
        type Value = NaiveDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a string represents chrono::NaiveDateTime")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
        {
            match NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.6f") {
                Ok(t) => Ok(t),
                Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(s), &self)),
            }
        }
    }
}

pub fn parse_field(field: &String) -> Result<OSProfilerSpan, String> {
    let result: OSProfilerSpan = match serde_json::from_str(field) {
        Ok(a) => a,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    if result.name == "asynch_request" || result.name == "asynch_wait" {
        return match result.info {
            OSProfilerEnum::Annotation(_) => Ok(result),
            _ => {
                println!("{:?}", result);
                Err("".to_string())
            }
        };
    }
    Ok(result)
}
//testing parsing field
#[cfg(test)]
mod tests {

    use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

    use super::*;

    #[test]
    fn test_parse() {
        let d = NaiveDate::from_ymd(2020, 06, 23);
        let t = NaiveTime::from_hms_milli(14, 32, 34, 0058);

        let dt = NaiveDateTime::new(d, t);
        let y: u64 = 293402358;

        let current_info =
            OSProfilerEnum::Annotation(AnnotationEnum::KeyValue(KeyValueAnnotationInfo {
                value: y,
                tracepoint_id: "nova/usr/local".to_string(),
                host: "cloudlab".to_string(),
                thread_id: 5743728237,
                pid: 4771,
            }));

        let my_uuid = Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap();

        let test_struct = OSProfilerSpan {
            trace_id: my_uuid,
            parent_id: my_uuid,
            project: "nova".to_string(),
            name: "build_instance".to_string(),
            base_id: my_uuid,
            service: "nova".to_string(),
            tracepoint_id: "nova/manager.py".to_string(),
            timestamp: dt,
            info: current_info,
        };

        //checking if parse_field function works with the added struct to parse the code correctly
        assert_eq!(parse_field(&(r#"{"trace_id": "936DA01F9ABD4d9d80C702AF85C822A8", "parent_id": "936DA01F9ABD4d9d80C702AF85C822A8", "project": "nova", "name": "build_instance",  "base_id": "936DA01F9ABD4d9d80C702AF85C822A8", "service": "nova", "tracepoint_id": "nova/manager.py", "timestamp": "2020-06-23T14:32:34.058", "info": {"value":293402358,"tracepoint_id": "nova/usr/local", "host": "cloudlab", "thread_id": 5743728237, "pid": 4771}}"#).to_string()),Ok(test_struct));
    }
}

