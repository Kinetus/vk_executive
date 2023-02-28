mod build_error;
pub use build_error::BuildError;
use hyper_tls::HttpsConnector;

use super::Config;

use std::time::Duration;

use http::request::Request;
use hyper::body::Body;
use tower::Service;

use crate::client::HyperClient;

#[derive(Debug)]
pub struct Builder<C>
where
    C: Service<Request<Body>>,
{
    pub token: Option<String>,
    pub http_client: C,
    pub api_url: String,
    pub api_version: String,
    pub time_between_requests: std::time::Duration,
}

impl<C> PartialEq for Builder<C>
where
    C: Service<Request<Body>>,
{
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
            && self.api_url == other.api_url
            && self.api_version == other.api_version
            && self.time_between_requests == other.time_between_requests
    }
}

impl<C> Clone for Builder<C>
where
    C: Service<Request<Body>> + Clone,
{
    fn clone(&self) -> Self {
        #[allow(clippy::clone_on_copy)]
        Self {
            token: self.token.clone(),
            http_client: self.http_client.clone(),
            api_url: self.api_url.clone(),
            api_version: self.api_version.clone(),
            time_between_requests: self.time_between_requests.clone(),
        }
    }
}

impl Builder<HyperClient> {
    /// Constructs new `Builder`
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Builder<HyperClient> {
    fn default() -> Self {
        Self {
            token: None,
            http_client: hyper::client::Client::builder().build(HttpsConnector::new()),
            api_url: String::from("https://api.vk.com/"),
            api_version: String::from("5.103"),
            time_between_requests: Duration::from_millis(334),
        }
    }
}

impl<C> Builder<C>
where
    C: Service<Request<Body>>,
{
    /// Sets token. It's required field.
    ///
    /// # Example:
    /// ```rust
    /// use vk_executive::config;
    ///
    /// let config = config::Builder::new()
    ///     .token(String::from("12345"));
    ///
    /// assert_eq!(
    ///     config,
    ///     config::Builder {
    ///         token: Some(String::from("12345")),
    ///         ..config::Builder::default()
    ///     }
    /// );
    /// ```
    pub fn token<T>(mut self, token: T) -> Self
    where
        T: ToString,
    {
        self.token = Some(token.to_string());
        self
    }

    /// Sets http client
    ///
    /// # Example:
    /// ```rust
    /// use hyper::client::Client;
    /// use hyper_tls::HttpsConnector;
    /// use vk_executive::config;
    ///
    /// let config = config::Builder::new()
    ///     .http_client(Client::builder().build(HttpsConnector::new()));
    ///
    /// assert_eq!(
    ///     config,
    ///     config::Builder {
    ///         http_client: Client::builder().build(HttpsConnector::new()),
    ///         ..config::Builder::default()
    ///     }
    /// );
    /// ```
    pub fn http_client(mut self, http_client: C) -> Self {
        self.http_client = http_client;
        self
    }

    /// Sets server url
    ///
    /// # Example:
    /// ```rust
    /// use vk_executive::config;
    ///
    /// let config = config::Builder::new()
    ///     .api_url(String::from("https:://vk.ru"));
    ///
    /// assert_eq!(
    ///     config,
    ///     config::Builder {
    ///         api_url: String::from("https:://vk.ru"),
    ///         ..config::Builder::default()
    ///     }
    /// );
    /// ```
    pub fn api_url(mut self, api_url: impl ToString) -> Self {
        self.api_url = api_url.to_string();
        self
    }

    /// Sets an api version
    ///
    /// # Example:
    /// ```rust
    /// use vk_executive::config;
    ///
    /// let config = config::Builder::new()
    ///     .api_version(String::from("5.144"));
    ///
    /// assert_eq!(
    ///     config,
    ///     config::Builder {
    ///         api_version: String::from("5.144"),
    ///         ..config::Builder::default()
    ///     }
    /// );
    /// ```
    pub fn api_version(mut self, api_version: impl ToString) -> Self {
        self.api_version = api_version.to_string();
        self
    }

    /// Sets time between http requests
    ///
    /// # Example:
    /// ```rust
    /// use std::time::Duration;
    /// use vk_executive::config;
    ///
    /// let config = config::Builder::new()
    ///     .time_between_requests(Duration::from_millis(300));
    ///
    /// assert_eq!(
    ///     config,
    ///     config::Builder {
    ///         time_between_requests: Duration::from_millis(300),
    ///         ..config::Builder::default()
    ///     }
    /// );
    /// ```
    pub const fn time_between_requests(
        mut self,
        time_between_requests: std::time::Duration,
    ) -> Self {
        self.time_between_requests = time_between_requests;
        self
    }

    /// Builds an [`Config`]
    ///
    /// # Example:
    /// ```rust
    /// use hyper::Client;
    /// use hyper_tls::HttpsConnector;
    /// use std::time::Duration;
    /// use vk_executive::config;
    ///
    /// let config = config::Builder::new()
    ///     .token(String::from("123456789"))
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(
    ///     config,
    ///     config::Config {
    ///         token: String::from("123456789"),
    ///         http_client: Client::builder().build(HttpsConnector::new()),
    ///         api_url: String::from("https://api.vk.com/"),
    ///         api_version: String::from("5.103"),
    ///         time_between_requests: Duration::from_millis(334)
    ///     }
    /// );
    /// ```
    ///
    /// # Errors
    /// This method fails whenever token haven't passed
    pub fn build(self) -> Result<Config<C>, BuildError> {
        if let None | Some("") = self.token.as_deref() {
            return Err(BuildError::MissingParameter(String::from("token")));
        };

        Ok(Config {
            token: self.token.unwrap(),
            http_client: self.http_client,
            api_url: self.api_url,
            api_version: self.api_version,
            time_between_requests: self.time_between_requests,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_token() {
        let config = Builder::new().build();

        assert_eq!(
            config.err(),
            Some(BuildError::MissingParameter(String::from("token")))
        );
    }

    #[test]
    fn custom_api_url() {
        let config = Builder::new()
            .api_url("https://example.com/")
            .token(String::from("token"))
            .build()
            .unwrap();

        assert_eq!(
            config,
            Config {
                token: String::from("token"),
                http_client: hyper::client::Client::builder().build(HttpsConnector::new()),
                api_url: String::from("https://example.com/"),
                api_version: String::from("5.103"),
                time_between_requests: Duration::from_millis(334)
            }
        );
    }

    #[test]
    fn custom_all() {
        let config = Builder::new()
            .api_url("https://api.vk.ru/")
            .api_version("5.143")
            .token(String::from("123456789"))
            .http_client(hyper::client::Client::builder().build(HttpsConnector::new()))
            .time_between_requests(Duration::from_millis(500))
            .build()
            .unwrap();

        assert_eq!(
            config,
            Config {
                token: String::from("123456789"),
                http_client: hyper::client::Client::builder().build(HttpsConnector::new()),
                api_url: String::from("https://api.vk.ru/"),
                api_version: String::from("5.143"),
                time_between_requests: Duration::from_millis(500)
            }
        );
    }

    #[test]
    fn custom_token() {
        let config = Builder::new()
            .token(String::from("123456789"))
            .build()
            .unwrap();

        assert_eq!(
            config,
            Config {
                token: String::from("123456789"),
                http_client: hyper::client::Client::builder().build(HttpsConnector::new()),
                api_url: String::from("https://api.vk.com/"),
                api_version: String::from("5.103"),
                time_between_requests: Duration::from_millis(334)
            }
        );
    }
}
