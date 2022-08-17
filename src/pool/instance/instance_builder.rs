use super::Instance;
use std::time::Duration;

mod build_error;
use build_error::BuildError;

pub struct InstanceBuilder<'a> {
    token: Option<String>,
    api_url: Option<&'a str>,
    api_version: Option<&'a str>,
    time_between_requests: Option<std::time::Duration>,
}

impl<'a> InstanceBuilder<'a> {
    pub fn new() -> InstanceBuilder<'a> {
        InstanceBuilder {
            token: None,
            api_url: None,
            api_version: None,
            time_between_requests: None,
        }
    }

    pub fn token(mut self, token: String) -> InstanceBuilder<'a> {
        self.token = Some(token);
        self
    }

    pub fn api_url(mut self, api_url: &'a str) -> InstanceBuilder<'a> {
        self.api_url = Some(api_url);
        self
    }

    pub fn api_version(mut self, api_version: &'a str) -> InstanceBuilder<'a> {
        self.api_version = Some(api_version);
        self
    }

    pub fn time_between_requests(mut self, time_between_requests: std::time::Duration) -> InstanceBuilder<'a> {
        self.time_between_requests = Some(time_between_requests);
        self
    }

    pub fn build(self) -> Result<Instance<'a>, BuildError> {
        let token = match self.token {
            Some(token) => token,
            None => return Err(BuildError::MissingParameter(String::from("token"))),
        };

        Ok(Instance {
            token,
            api_url: self.api_url.unwrap_or("https://api.vk.com/"),
            api_version: self.api_version.unwrap_or("5.103"),
            time_between_requests: self.time_between_requests.unwrap_or(Duration::from_millis(334)),
        })
    }
}

#[cfg(test)]
mod tests {
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
            .build();

        assert_eq!(
            instance.ok(),
            Some(Instance {
                token: String::from("token"),
                api_url: "https://example.com/",
                api_version: "5.103",
                time_between_requests: Duration::from_millis(334)
            })
        );
    }

    #[test]
    fn custom_all() {
        let instance = InstanceBuilder::new()
            .api_url("https://api.vk.ru/")
            .api_version("5.143")
            .token(String::from("123456789"))
            .time_between_requests(Duration::from_millis(500))
            .build();

        assert_eq!(
            instance.ok(),
            Some(Instance {
                token: String::from("123456789"),
                api_url: "https://api.vk.ru/",
                api_version: "5.143",
                time_between_requests: Duration::from_millis(500)
            })
        );
    }
}
