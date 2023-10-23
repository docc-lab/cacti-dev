mod chain;

use crate::grouping::Group;
use crate::trace::TracepointID;
use pythia_common::RequestType;

pub trait ProvenanceNode {
    fn add_child(&self, groups: Vec<Group>, decisions: Vec<(TracepointID, RequestType)>);
    fn find_child(&self, decision: Option<String>) -> Box<dyn ProvenanceNode>;
    fn find_parent(&self) -> Box<dyn ProvenanceNode>;
}

