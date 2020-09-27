use std::sync::Arc;
use tokio_native_tls::TlsAcceptor;

#[derive(Debug)]
pub struct ConfigBuilder {
    domain: String,
    server_agent: Option<String>,
    tls_acceptor: Option<TlsAcceptor>,
    tls_required: bool,
}

impl ConfigBuilder {
    pub fn new<T: Into<String>>(domain: T) -> ConfigBuilder {
        ConfigBuilder {
            domain: domain.into(),
            server_agent: None,
            tls_acceptor: None,
            tls_required: false,
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

    pub fn with_tls<T: Into<TlsAcceptor>>(mut self, tls_acceptor: T) -> ConfigBuilder {
        self.tls_acceptor = Some(tls_acceptor.into());
        self
    }

    pub fn force_tls(mut self) -> ConfigBuilder {
        self.tls_required = true;
        self
    }

    pub fn build(self) -> Config {
        Config {
            raw_config: Arc::new(RawConfig {
                domain: self.domain,
                server_agent: self.server_agent.unwrap_or(String::from("Rust SMTP server")),
                tls_acceptor: self.tls_acceptor,
                tls_required: self.tls_required,
            })
        }
    }
}

#[derive(Debug)]
struct RawConfig {
    domain: String,
    server_agent: String,
    tls_acceptor: Option<TlsAcceptor>,
    tls_required: bool,
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

    pub fn tls_acceptor(&self) -> Option<&TlsAcceptor> {
        self.raw_config.tls_acceptor.as_ref()
    }

    pub fn tls_available(&self) -> bool {
        self.raw_config.tls_acceptor.is_some()
    }

    pub fn tls_required(&self) -> bool {
        self.raw_config.tls_required
    }
}