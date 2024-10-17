use serde::{Deserialize, Serialize};

use std::fmt;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct SWRequestType {
    pub rt: String
}

impl fmt::Display for SWRequestType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SkyWalkingRT({})", self.rt)
    }
}