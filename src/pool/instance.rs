use std::time::Duration;

mod instance_builder;
use instance_builder::InstanceBuilder;

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

    pub fn from_tokens<I, T>(tokens: I) -> Vec<Instance>
    where 
        I: Iterator<Item = T>,
        T: ToString
    {
        let mut instances = Vec::new();

        for token in tokens {
            instances.push(Instance::new().token(token.to_string()).build().unwrap());
        }
        
        instances
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    pub fn time_between_requests(&self) -> Duration {
        self.time_between_requests
    }
}