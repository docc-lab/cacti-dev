use serde::{Deserialize, Serialize};

use std::fmt;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct JaegerRequestType {
    pub rt: String
}

impl fmt::Display for JaegerRequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JaegerRT({})", self.rt)
    }
}