mod instance_builder;
pub use instance_builder::{InstanceBuilder, BuildError};

use std::time::Duration;

#[derive(Debug)]
pub struct Instance {
    pub token: String,
    pub http_client: reqwest::Client,
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

impl Default for Instance {
    fn default() -> Self {
        Instance::new()
    }
}

impl Instance {
    /// Constructs [`Instance`]
    pub fn new() -> Instance {
        Instance::builder().build().unwrap()
    }

    /// Constructs [`InstanceBuilder`]
    pub fn builder() -> InstanceBuilder {
        InstanceBuilder::new()
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
    pub fn from_tokens<Tokens, Token>(tokens: Tokens) -> Result<Vec<Instance>, BuildError>
    where 
        Tokens: IntoIterator<Item = Token>,
        Token: ToString
    {
        Instance::from_tokens_by_prototype(tokens, InstanceBuilder::new())
    }

    pub fn from_tokens_by_prototype<Tokens, Token>(tokens: Tokens, prototype: InstanceBuilder) -> Result<Vec<Instance>, BuildError>
    where 
        Tokens: IntoIterator<Item = Token>,
        Token: ToString
    {
        let mut instances = Vec::new();

        for token in tokens {
            instances.push(prototype.clone().token(token).build()?);
        }
        Ok(instances)
    }
}

