use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send + Sync + 'static {
    async fn get(&self);

    async fn set(&self);

    async fn remove(&self);

    async fn reset(&self);

    async fn close(&self);
}

