use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericAIError {
    details: String,
}

#[allow(dead_code)]
impl GenericAIError {
    pub fn new<S: Into<String>>(msg: S) -> GenericAIError {
        GenericAIError {
            details: msg.into(),
        }
    }
}

impl fmt::Display for GenericAIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for GenericAIError {
    fn description(&self) -> &str {
        &self.details
    }
}
