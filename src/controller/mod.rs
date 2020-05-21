mod hdfs;
mod osprofiler;

use pythia_common::RequestType;

use crate::controller::hdfs::HDFSController;
use crate::controller::osprofiler::OSProfilerController;
use crate::settings::ApplicationType;
use crate::settings::Settings;
use crate::trace::TracepointID;

pub trait Controller: Send + Sync {
    fn enable(&self, points: &Vec<(TracepointID, Option<RequestType>)>);
    fn disable(&self, points: &Vec<(TracepointID, Option<RequestType>)>);
    fn is_enabled(&self, point: &(TracepointID, Option<RequestType>)) -> bool;
    fn disable_all(&self);
    fn enable_all(&self);

    fn disable_by_name(&self, point: &str) {
        self.disable(&vec![(TracepointID::from_str(point), None)]);
    }
}

pub fn controller_from_settings(settings: &Settings) -> Box<dyn Controller> {
    match &settings.application {
        ApplicationType::OpenStack => Box::new(OSProfilerController::from_settings(settings)),
        ApplicationType::HDFS => Box::new(HDFSController::from_settings(settings)),
        ApplicationType::Uber => panic!("Can't control uber"),
    }
}