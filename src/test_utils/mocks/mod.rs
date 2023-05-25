#[cfg(feature = "streaming")]
mod streaming_client_mock;

#[cfg(feature = "streaming")]
pub use streaming_client_mock::MockStreamingClient;
