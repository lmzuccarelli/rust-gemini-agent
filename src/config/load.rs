use serde_derive::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Parameters {
    pub name: String,
    pub description: String,
    pub log_level: String,
    pub base_url: String,
    pub db_path: String,
    pub test: bool,
}

pub trait ConfigInterface {
    fn read(&self, dir: String) -> Result<Parameters, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct ImplConfigInterface {}

impl ConfigInterface for ImplConfigInterface {
    fn read(&self, name: String) -> Result<Parameters, Box<dyn std::error::Error>> {
        let json_data = File::open(&name)?;
        let params = serde_json::from_reader(json_data)?;
        Ok(params)
    }
}
