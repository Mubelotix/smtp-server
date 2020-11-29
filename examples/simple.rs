use smtp_server::{EventHandler, SmtpServer};

#[tokio::main]
async fn main() {
    env_logger::init();

    struct EHandler {};

    #[async_trait::async_trait]
    impl EventHandler for EHandler {
        async fn on_mail<'b>(
            &self,
            email: std::pin::Pin<&email_parser::email::Email<'b>>,
        ) -> Result<(), String> {
            log::info!("{:?}", email.as_ref().body.as_ref());
            Ok(())
        }
    }

    SmtpServer::new(EHandler {}, "mubelotix.dev")
        .tls("certificate.pfx", "password")
        .run();
}
