use serde_derive::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct EremitConfig {
    pub version: u8,
    pub port: String,
}