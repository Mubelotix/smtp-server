use tokio_native_tls::TlsAcceptor;

#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) domain: String,
    pub(crate) server_agent: String,
    pub(crate) tls_acceptor: Option<TlsAcceptor>,
    pub(crate) tls_required: bool,
}

impl Config {
    pub fn new(domain: String) -> Config {
        Config {
            domain,
            server_agent: String::from("Rust SMTP server"),
            tls_acceptor: None,
            tls_required: false,
        }
    }
}
