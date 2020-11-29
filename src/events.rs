use async_trait::async_trait;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_mail<'b>(
        &self,
        email: std::pin::Pin<&email_parser::email::Email<'b>>,
    ) -> Result<(), String>;

    async fn expand_mailing_list(&self, _name: String) -> Option<Vec<String>> {
        None
    }

    async fn verify_user(&self, _name: String) -> bool {
        false
    }
}
