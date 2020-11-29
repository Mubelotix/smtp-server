use smtp_server::{SmtpServer, EventHandler};

#[tokio::main]
async fn main() {
    env_logger::init();

    struct EHandler {};

    #[async_trait::async_trait]
    impl EventHandler for EHandler {
        async fn on_mail<'b>(&self, email: std::pin::Pin<&email_parser::email::Email<'b>>) -> Result<(), String> {
            println!("{:?}", email.as_ref().body.as_ref());
            Ok(())
        }
    }

    let mut server = SmtpServer::new(EHandler {});
    server.run();
}