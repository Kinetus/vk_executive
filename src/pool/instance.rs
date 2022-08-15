use std::str::Split;

pub struct Instance {
    token: String,
    millis_between_requests: u64,
}

impl Instance {
    pub fn new(token: String, millis_between_requests: u64) -> Instance {
        Instance {
            token,
            millis_between_requests
        }
    }

    pub fn vector_from_args(number: usize, tokens: Split<&str>, millis_between_requests: u64) -> Vec<Instance> {
        let mut instances = Vec::new();

        for token in tokens.take(number) {
            instances.push(Instance::new(token.to_string(), millis_between_requests));
        }
        
        instances
    }

    pub fn token(&self) -> String {
        self.token.clone()
    }

    pub fn millis_between_requests(&self) -> u64 {
        self.millis_between_requests
    }
}