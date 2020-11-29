use std::sync::Arc;
use tokio_native_tls::TlsAcceptor;

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
    pub fn new(domain: String) -> Config {
        Config {
            raw_config: Arc::new(RawConfig {
                domain,
                server_agent: String::from("Rust SMTP server"),
                tls_acceptor: None,
                tls_required: false,
            })
        }
    }

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