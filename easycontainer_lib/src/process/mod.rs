pub mod service;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Service {
    command: String,
    args: Vec<String>
}