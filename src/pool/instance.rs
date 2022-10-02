mod instance_builder;
use instance_builder::{InstanceBuilder, BuildError};

use std::time::Duration;

#[derive(Debug)]
pub struct Instance {
    pub token: String,
    pub client: reqwest::Client,
    pub api_url: String,
    pub api_version: String,
    pub time_between_requests: Duration,
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token &&
        self.api_url == other.api_url &&
        self.api_version == other.api_version &&
        self.time_between_requests == other.time_between_requests
    }
}

impl Instance {
    pub fn new() -> InstanceBuilder {
        InstanceBuilder::new()
    }

    pub fn from_tokens<I, T>(tokens: I) -> Result<Vec<Instance>, BuildError>
    where 
        I: Iterator<Item = T>,
        T: ToString
    {
        let mut instances = Vec::new();

        for token in tokens {
            instances.push(Instance::new().token(token.to_string()).build()?);
        }
        
        Ok(instances)
    }
}
