mod builder;
pub use builder::{BuildError, Builder};
use hyper::body::Body;
use std::time::Duration;

use http::request::Request;
use tower::Service;
use crate::client::HyperClient;

#[derive(Debug)]
pub struct Config<C = HyperClient>
where
    C: Service<Request<Body>>,
{
    pub token: String,
    pub http_client: C,
    pub api_url: String,
    pub api_version: String,
    pub time_between_requests: Duration,
}

impl<C> PartialEq for Config<C>
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

impl Default for Config<HyperClient> {
    fn default() -> Self {
        Self::builder().build().unwrap()
    }
}

impl Config<HyperClient> {
    /// Constructs a new [`Config`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [`Builder`] to configure a [`Config`]
    pub fn builder() -> Builder<HyperClient> {
        Builder::new()
    }

    /// Constructs vector of `Configs` from any [`Iterator`] of tokens
    ///
    /// Example:
    /// ```rust
    /// use vk_executive::Config;
    ///
    /// let configs = Config::from_tokens(["123456789", "1111"].into_iter()).unwrap();
    ///
    /// assert_eq!(
    ///     configs,
    ///     vec![
    ///         Config::builder().token("123456789".to_string()).build().unwrap(),
    ///         Config::builder().token("1111".to_string()).build().unwrap()
    ///     ]
    /// )
    /// ```
    pub fn from_tokens<Tokens, Token>(tokens: Tokens) -> Result<Vec<Self>, BuildError>
    where
        Tokens: Iterator<Item = Token>,
        Token: ToString,
    {
        Self::from_tokens_by_prototype(tokens, &Builder::new())
    }

}

impl<C> Config<C>
where
    C: Service<Request<Body>> + Clone,
{
    pub fn from_tokens_by_prototype<Tokens, Token>(
        tokens: Tokens,
        prototype: &Builder<C>,
    ) -> Result<Vec<Self>, BuildError>
    where
        Tokens: Iterator<Item = Token>,
        Token: ToString,
    {
        let mut configs = Vec::new();

        for token in tokens {
            configs.push(prototype.clone().token(token).build()?);
        }

        Ok(configs)
    }
}
