//struct Config represents a yaml configuration file

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::CliArgs;

pub type Config = Vec<Job>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Job{
    pub job: String,
    pub phases: Vec<Phase>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Phase{
    pub name: String,
    pub operations: Vec<CliArgs>,
}

pub fn load_config(path: &str) -> Config {
    let path = Path::new(path);
    let contents = fs::read_to_string(path).expect("could not read file");
    let config: Config = serde_yaml::from_str(&contents).expect("could not parse yaml");
    config
}