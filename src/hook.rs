use async_trait::async_trait;

/// Hook will be riggered by an event.
#[async_trait]
pub trait Hook<E> {
    /// This method will trigger when the event `E` happens.
    async fn call_hook(event: E) -> Result<(), anyhow::Error>;
}
