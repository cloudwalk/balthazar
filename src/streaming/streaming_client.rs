use super::message::Message;

#[crate::async_trait]
pub trait StreamingClient: Sync + Send + 'static {
    async fn publish(&self, message: Message) -> crate::Result<()>;
    async fn health_check(&self) -> crate::Result<()>;
}
