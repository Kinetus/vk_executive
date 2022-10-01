mod build_error;
pub use build_error::BuildError;

use super::Instance;

use std::time::Duration;

pub struct InstanceBuilder {
    pub token: Option<String>,
    pub client: reqwest::Client,
    pub api_url: String,
    pub api_version: String,
    pub time_between_requests: std::time::Duration,
}

impl InstanceBuilder {
    pub fn new() -> InstanceBuilder {
        InstanceBuilder::default()
    }

    pub fn token<T>(mut self, token: T) -> InstanceBuilder
    where 
        T: ToString
    {
        self.token = Some(token.to_string());
        self
    }

    pub fn api_url<T>(mut self, api_url: T) -> InstanceBuilder
    where 
        T: ToString
    {
        self.api_url = api_url.to_string();
        self
    }

    pub fn api_version<T>(mut self, api_version: T ) -> InstanceBuilder
    where 
        T: ToString
    {
        self.api_version = api_version.to_string();
        self
    }

    pub fn time_between_requests(mut self, time_between_requests: std::time::Duration) -> InstanceBuilder {
        self.time_between_requests = time_between_requests;
        self
    }

    pub fn build(self) -> Result<Instance, BuildError> {
        let token = match self.token {
            Some(token) => token,
            None => return Err(BuildError::MissingParameter(String::from("token"))),
        };

        Ok(Instance {
            token,
            client: self.client,
            api_url: self.api_url,
            api_version: self.api_version,
            time_between_requests: self.time_between_requests,
        })
    }
}

impl Default for InstanceBuilder {
    fn default() -> Self {
        InstanceBuilder {
            token: None,
            client: reqwest::Client::new(),
            api_url: String::from("https://api.vk.com/"),
            api_version: String::from("5.103"),
            time_between_requests: Duration::from_millis(334),
        }
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use super::*;

    #[test]
    fn missing_token() {
        let instance = InstanceBuilder::new().build();

        assert_eq!(
            instance.err(),
            Some(BuildError::MissingParameter(String::from("token")))
        );
    }

    #[test]
    fn custom_api_url() {
        let instance = InstanceBuilder::new()
            .api_url("https://example.com/")
            .token(String::from("token"))
            .build()
            .unwrap();

        assert_eq!(
            instance,
            Instance {
                token: String::from("token"),
                client: Client::new(),
                api_url: String::from("https://example.com/"),
                api_version: String::from("5.103"),
                time_between_requests: Duration::from_millis(334)
            }
        );
    }

    #[test]
    fn custom_all() {
        let instance = InstanceBuilder::new()
            .api_url("https://api.vk.ru/")
            .api_version("5.143")
            .token(String::from("123456789"))
            .time_between_requests(Duration::from_millis(500))
            .build()
            .unwrap();

        assert_eq!(
            instance,
            Instance {
                token: String::from("123456789"),
                client: Client::new(),
                api_url: String::from("https://api.vk.ru/"),
                api_version: String::from("5.143"),
                time_between_requests: Duration::from_millis(500)
            }
        );
    }
}
