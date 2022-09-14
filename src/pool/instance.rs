use std::time::Duration;

mod instance_builder;
use instance_builder::InstanceBuilder;

#[derive(PartialEq, Debug)]
pub struct Instance {
    token: String,
    api_url: String,
    api_version: String,
    time_between_requests: Duration,
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

    pub fn token(&self) -> &String {
        &self.token
    }

    pub fn api_url(&self) -> &String {
        &self.api_url
    }

    pub fn api_version(&self) -> &String {
        &self.api_version
    }

    pub fn time_between_requests(&self) -> Duration {
        self.time_between_requests
    }
}