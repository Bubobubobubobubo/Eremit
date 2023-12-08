use serde_derive::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct EremitConfig {
    version: u8,
    port: String,
}

