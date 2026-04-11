use crate::errors::AppError;

pub async fn retry_db<F, Fut, T>(f: F) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let strategy = tokio_retry::strategy::ExponentialBackoff::from_millis(100)
        .max_delay(std::time::Duration::from_secs(3))
        .take(3);

    tokio_retry::Retry::spawn(strategy, f).await
}
