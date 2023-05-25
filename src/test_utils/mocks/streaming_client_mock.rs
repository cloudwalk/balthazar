use crate::{async_trait, throw, Message, Result, StreamingClient as StreamingClientInterface};
use mockall::{mock, predicate::eq};

mock! {
    pub StreamingClient {}

    #[async_trait]
    impl StreamingClientInterface for StreamingClient {
        async fn publish(&self, message: Message) -> Result<()>;
        async fn health_check(&self) -> Result<()>;
    }
}

impl MockStreamingClient {
    pub fn publish(mut self, message: Message, result: Result<()>) -> Self {
        self.expect_publish()
            .times(1)
            .with(eq(message))
            .returning(move |_| match &result {
                Ok(_) => Ok(()),
                Err(_) => Err(throw!("Publish error")),
            });

        self
    }

    pub fn health_check(mut self, result: Result<()>) -> Self {
        self.expect_health_check()
            .times(1)
            .returning(move || match &result {
                Ok(_) => Ok(()),
                Err(_) => Err(throw!("Health check error")),
            });

        self
    }
}
