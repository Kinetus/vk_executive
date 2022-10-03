mod build_error;
pub use build_error::BuildError;

use super::Instance;

use std::time::Duration;

#[derive(Debug)]
pub struct InstanceBuilder {
    pub token: Option<String>,
    pub http_client: reqwest::Client,
    pub api_url: String,
    pub api_version: String,
    pub time_between_requests: std::time::Duration,
}

impl PartialEq for InstanceBuilder {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token &&
        self.api_url == other.api_url &&
        self.api_version == other.api_version &&
        self.time_between_requests == other.time_between_requests
    }
}

impl InstanceBuilder {
    /// Constructs new `InstanceBuilder`
    pub fn new() -> InstanceBuilder {
        InstanceBuilder::default()
    }

    /// Sets token. It's required field.
    /// 
    /// # Example:
    /// ```rust
    /// use fast_vk::instance;
    /// 
    /// let instance = instance::InstanceBuilder::new()
    ///     .token(String::from("12345"));
    /// 
    /// assert_eq!(
    ///     instance,
    ///     instance::InstanceBuilder {
    ///         token: Some(String::from("12345")),
    ///         ..instance::InstanceBuilder::default()
    ///     }
    /// );
    /// ```
    pub fn token<T>(mut self, token: T) -> InstanceBuilder
    where 
        T: ToString
    {
        self.token = Some(token.to_string());
        self
    }

    /// Sets http client
    /// 
    /// # Example:
    /// ```rust
    /// use reqwest::Client;
    /// use fast_vk::instance;
    /// 
    /// let instance = instance::InstanceBuilder::new()
    ///     .http_client(Client::new());
    /// 
    /// assert_eq!(
    ///     instance,
    ///     instance::InstanceBuilder {
    ///         http_client: Client::new(),
    ///         ..instance::InstanceBuilder::default()
    ///     }
    /// );
    /// ```
    pub fn http_client(mut self, http_client: reqwest::Client) -> InstanceBuilder {
        self.http_client = http_client;
        self
    }

    /// Sets server url
    /// 
    /// # Example:
    /// ```rust
    /// use fast_vk::instance;
    /// 
    /// let instance = instance::InstanceBuilder::new()
    ///     .api_url(String::from("https:://vk.ru"));
    /// 
    /// assert_eq!(
    ///     instance,
    ///     instance::InstanceBuilder {
    ///         api_url: String::from("https:://vk.ru"),
    ///         ..instance::InstanceBuilder::default()
    ///     }
    /// );
    /// ```
    pub fn api_url<T>(mut self, api_url: T) -> InstanceBuilder
    where 
        T: ToString
    {
        self.api_url = api_url.to_string();
        self
    }

    /// Sets an api version
    /// 
    /// # Example:
    /// ```rust
    /// use fast_vk::instance;
    /// 
    /// let instance = instance::InstanceBuilder::new()
    ///     .api_version(String::from("5.144"));
    /// 
    /// assert_eq!(
    ///     instance,
    ///     instance::InstanceBuilder {
    ///         api_version: String::from("5.144"),
    ///         ..instance::InstanceBuilder::default()
    ///     }
    /// );
    /// ```
    pub fn api_version<T>(mut self, api_version: T ) -> InstanceBuilder
    where 
        T: ToString
    {
        self.api_version = api_version.to_string();
        self
    }

    /// Sets time between http requests
    /// 
    /// # Example:
    /// ```rust
    /// use std::time::Duration;
    /// use fast_vk::instance;
    /// 
    /// let instance = instance::InstanceBuilder::new()
    ///     .time_between_requests(Duration::from_millis(300));
    /// 
    /// assert_eq!(
    ///     instance,
    ///     instance::InstanceBuilder {
    ///         time_between_requests: Duration::from_millis(300),
    ///         ..instance::InstanceBuilder::default()
    ///     }
    /// );
    /// ```
    pub fn time_between_requests(mut self, time_between_requests: std::time::Duration) -> InstanceBuilder {
        self.time_between_requests = time_between_requests;
        self
    }
    
    /// Builds an [`Instance`]
    /// 
    /// # Example:
    /// ```rust
    /// use reqwest::Client;
    /// use std::time::Duration;
    /// use fast_vk::instance;
    /// 
    /// let instance = instance::InstanceBuilder::new()
    ///     .token(String::from("123456789"))
    ///     .build()
    ///     .unwrap();
    /// 
    /// assert_eq!(
    ///     instance,
    ///     instance::Instance {
    ///         token: String::from("123456789"),
    ///         http_client: Client::new(),
    ///         api_url: String::from("https://api.vk.com/"),
    ///         api_version: String::from("5.103"),
    ///         time_between_requests: Duration::from_millis(334)
    ///     }
    /// );
    /// ```
    pub fn build(self) -> Result<Instance, BuildError> {
        let token = match self.token {
            Some(token) => token,
            None => return Err(BuildError::MissingParameter(String::from("token"))),
        };

        Ok(Instance {
            token,
            http_client: self.http_client,
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
            http_client: reqwest::Client::new(),
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
                http_client: Client::new(),
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
            .http_client(Client::new())
            .time_between_requests(Duration::from_millis(500))
            .build()
            .unwrap();

        assert_eq!(
            instance,
            Instance {
                token: String::from("123456789"),
                http_client: Client::new(),
                api_url: String::from("https://api.vk.ru/"),
                api_version: String::from("5.143"),
                time_between_requests: Duration::from_millis(500)
            }
        );
    }

    #[test]
    fn custom_token() {
        let instance = InstanceBuilder::new()
            .token(String::from("123456789"))
            .build()
            .unwrap();
     
        assert_eq!(
            instance,
            Instance {
                token: String::from("123456789"),
                http_client: Client::new(),
                api_url: String::from("https://api.vk.com/"),
                api_version: String::from("5.103"),
                time_between_requests: Duration::from_millis(334)
            }
        );
    }
}
