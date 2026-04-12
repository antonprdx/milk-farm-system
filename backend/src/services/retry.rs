use crate::errors::AppError;

pub async fn retry_db<F, Fut, T>(f: F) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let mut delay = std::time::Duration::from_millis(100);
    for attempt in 0..3 {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if attempt < 2 => {
                tokio::time::sleep(delay).await;
                delay = delay.min(std::time::Duration::from_secs(3)) * 2;
                tracing::warn!(error = %e, attempt, "Retrying DB operation");
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
