mod builder;
pub use builder::{BuildError, Builder};
use std::time::Duration;

use http::request::Request;
use hyper::{body::Body, client::HttpConnector};
use hyper_tls::HttpsConnector;
use tower::Service;

pub type HyperClient = hyper::client::Client<HttpsConnector<HttpConnector>>;

#[derive(Debug)]
pub struct Instance<C>
where
    C: Service<Request<Body>>,
{
    pub token: String,
    pub http_client: C,
    pub api_url: String,
    pub api_version: String,
    pub time_between_requests: Duration,
}

impl<C> PartialEq for Instance<C>
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

impl Default for Instance<HyperClient> {
    fn default() -> Self {
        Self::builder().build().unwrap()
    }
}

impl Instance<HyperClient> {
    /// Constructs a new [`Instance`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [`InstanceBuilder`] to configure a [`Instance`]
    pub fn builder() -> Builder<HyperClient> {
        Builder::new()
    }

    /// Constructs vector of `Instances` from any [`IntoIterator`] of tokens
    ///
    /// Example:
    /// ```rust
    /// use fast_vk::Instance;
    ///
    /// let instances = Instance::from_tokens(["123456789", "1111"]).unwrap();
    ///
    /// assert_eq!(
    ///     instances,
    ///     vec![
    ///         Instance::builder().token("123456789".to_string()).build().unwrap(),
    ///         Instance::builder().token("1111".to_string()).build().unwrap()
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

impl<C> Instance<C>
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
        let mut instances = Vec::new();

        for token in tokens {
            instances.push(prototype.clone().token(token).build()?);
        }

        Ok(instances)
    }
}
