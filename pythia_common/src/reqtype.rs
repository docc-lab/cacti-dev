use serde::{Deserialize, Serialize};

use crate::PythiaError;
use crate::osprofiler::OSPRequestType;
use crate::jaeger::JaegerRequestType;
use crate::skywalking::SWRequestType;

use std::fmt;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub enum RequestType {
    OSP(OSPRequestType),
    Jaeger(JaegerRequestType),
    SW(SWRequestType),
    Unknown,
}

impl RequestType {
    pub fn from_str(typ: &str, app: &str) -> Result<RequestType, String> {
        match app {
            "OpenStack" => {
                match OSPRequestType::from_str(typ) {
                    Ok(osprt) => Ok(RequestType::OSP(osprt)),
                    Err(err) => Err(err),
                }
            },
            "Jaeger" => Ok(RequestType::Jaeger(JaegerRequestType {
                rt: typ.to_string()
            })),
            "SkyWalking" => Ok(RequestType::SW(SWRequestType {
                rt: typ.to_string()
            })),
            _ => Err(("Unknown request type!").to_string())
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RequestType::OSP(osprt) => osprt.to_string(),
            RequestType::Jaeger(jrt) => jrt.rt.clone(),
            RequestType::SW(swrt) => swrt.rt.clone(),
            _ => "".to_string()
        }
    }
}

impl fmt::Display for RequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestType::OSP(ort) => write!(f, "{:?}", ort),
            RequestType::Jaeger(jrt) => write!(f, "{:?}", jrt),
            RequestType::SW(swrt) => write!(f, "{:?}", swrt),
            RequestType::Unknown => write!(f, "UnknownRT"),
        }
    }
}