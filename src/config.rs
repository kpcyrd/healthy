use crate::errors::*;
use serde_derive::{Serialize, Deserialize};
use std::fs;
use std::net::Ipv4Addr;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub target: Vec<Target>,
}

impl Config {
    pub fn load(path: &str) -> Result<Config> {
        let content = fs::read(path)?;
        let config = toml::from_slice(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub host: Ipv4Addr,
}
