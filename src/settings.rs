use std::collections::HashMap;
use std::path::PathBuf;

use config::{Config, File, FileFormat};

#[derive(Debug)]
pub struct Settings {
    pub server_address: String,
    pub pythia_cache: PathBuf,
    pub manifest_file: PathBuf,
    pub manifest_root: PathBuf,
    pub redis_url: String,
    pub pythia_clients: Vec<String>,
    pub application: ApplicationType,
    pub manifest_method: ManifestMethod,
}

#[derive(Debug)]
pub enum ApplicationType {
    HDFS,
    OpenStack,
}

#[derive(Debug)]
pub enum ManifestMethod {
    Flat,
    CCT,
    Poset,
    Historic,
}

impl Settings {
    pub fn read() -> Settings {
        let mut settings = Config::default();
        settings
            .merge(File::new(
                "/opt/stack/reconstruction/Settings.toml",
                FileFormat::Toml,
            ))
            .unwrap();
        let results = settings.try_into::<HashMap<String, String>>().unwrap();
        let mut manifest_file = PathBuf::from(results.get("pythia_cache").unwrap());
        let manifest_method = match results.get("manifest_method").unwrap().as_str() {
            "CCT" => {
                manifest_file.push("cct_manifest");
                ManifestMethod::CCT
            }
            "Poset" => {
                manifest_file.push("poset_manifest");
                ManifestMethod::Poset
            }
            "Historic" => {
                manifest_file.push("historic_manifest");
                ManifestMethod::Historic
            }
            "Flat" => {
                manifest_file.push("flat_manifest");
                ManifestMethod::Flat
            }
            _ => panic!("Unsupported manifest method"),
        };
        let mut trace_cache = PathBuf::from(results.get("pythia_cache").unwrap());
        trace_cache.push("traces");
        Settings {
            server_address: results.get("server_address").unwrap().to_string(),
            pythia_cache: trace_cache,
            manifest_file: manifest_file,
            redis_url: results.get("redis_url").unwrap().to_string(),
            manifest_root: PathBuf::from(results.get("manifest_root").unwrap()),
            pythia_clients: results
                .get("pythia_clients")
                .unwrap()
                .split(",")
                .map(|x| x.to_string())
                .collect(),
            application: match results.get("application").unwrap().as_str() {
                "OpenStack" => ApplicationType::OpenStack,
                "HDFS" => ApplicationType::HDFS,
                _ => panic!("Unknown application type"),
            },
            manifest_method: manifest_method,
        }
    }
}