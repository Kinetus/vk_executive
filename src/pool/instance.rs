use std::time::Duration;

mod instance_builder;
use instance_builder::InstanceBuilder;

#[derive(PartialEq, Debug)]
pub struct Instance<'a> {
    token: String,
    api_url: &'a str,
    api_version: &'a str,
    time_between_requests: Duration,
}

impl<'a> Instance<'a> {
    pub fn new<'b>() -> InstanceBuilder<'b> {
        InstanceBuilder::new()
    }

    pub fn from_tokens<'b, I: Iterator<Item = &'b str>>(tokens: I) -> Vec<Instance<'a>> {
        let mut instances = Vec::new();

        for token in tokens {
            instances.push(Instance::new().token(String::from(token)).build().unwrap());
        }
        
        instances
    }

    pub fn token(&self) -> &String {
        &self.token
    }

    pub fn api_url(&self) -> &str {
        self.api_url
    }

    pub fn api_version(&self) -> &str {
        self.api_version
    }

    pub fn time_between_requests(&self) -> Duration {
        self.time_between_requests
    }
}