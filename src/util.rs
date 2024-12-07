use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub ipp_uri: String,
}
