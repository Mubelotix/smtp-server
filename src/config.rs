use std::sync::Arc;

#[derive(Debug)]
pub struct ConfigBuilder {
    domain: String,
    server_agent: Option<String>,
}

impl ConfigBuilder {
    pub fn new<T: Into<String>>(domain: T) -> ConfigBuilder {
        ConfigBuilder {
            domain: domain.into(),
            server_agent: None,
        }
    }

    pub fn with_domain<T: Into<String>>(mut self, domain: T) -> ConfigBuilder {
        self.domain = domain.into();
        self
    }

    pub fn with_server_agent<T: Into<String>>(mut self, domain: T) -> ConfigBuilder {
        self.server_agent = Some(domain.into());
        self
    }

    pub fn build(self) -> Config {
        Config {
            raw_config: Arc::new(RawConfig {
                domain: self.domain,
                server_agent: self.server_agent.unwrap_or(String::from("Rust SMTP server"))
            })
        }
    }
}

#[derive(Debug)]
struct RawConfig {
    domain: String,
    server_agent: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    raw_config: Arc<RawConfig>,
}

impl Config {
    pub fn domain(&self) -> &str {
        &self.raw_config.domain
    }

    pub fn server_agent(&self) -> &str {
        &self.raw_config.server_agent
    }
}