use std::{fmt::Display, future::Future, time::Duration};

use rand::{Rng, SeedableRng};

/// Applies a jitter + exponential backoff
/// 
/// The provided closure must return a `Future`
///
/// The `usize` in the closure represents the attempt number
///
/// If the provided closure fails on all attempts, then this function will return the last error that the closure had.
pub async fn retry_with_backoff_async<T, E, F, Fut>(
    mut function: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Display,
{
    let mut delay = initial_delay;
    // Use a thread-safe RNG
    let mut rng = rand_chacha::ChaChaRng::from_entropy();

    for attempt in 1..=max_retries {
        match function(attempt).await {
            Ok(result) => return Ok(result),
            Err(_) if attempt < max_retries => {
                // Add jitter: Randomize delay within 50%-150% of the current delay
                let jitter: f64 = rng.gen_range(0.5..1.5);
                let jittered_delay = delay.mul_f64(jitter);

                tokio::time::sleep(jittered_delay).await;

                // Exponential backoff: Double the delay for the next attempt
                delay *= 2;
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    unreachable!() // This should never be reached because all cases are handled above.
}

/// Applies a jitter + exponential backoff
/// 
/// The provided closure should not be async
///
/// The `usize` in the closure represents the attempt number
///
/// If the provided closure fails on all attempts, then this function will return the last error that the closure had.
pub async fn retry_with_backoff<T, E, F>(
    mut function: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut(usize) -> Result<T, E>,
    E: Display,
{
    let mut delay = initial_delay;
    // Use a thread-safe RNG
    let mut rng = rand_chacha::ChaChaRng::from_entropy();

    for attempt in 1..=max_retries {
        match function(attempt) {
            Ok(result) => return Ok(result),
            Err(_) if attempt < max_retries => {
                // Add jitter: Randomize delay within 50%-150% of the current delay
                let jitter: f64 = rng.gen_range(0.5..1.5);
                let jittered_delay = delay.mul_f64(jitter);

                tokio::time::sleep(jittered_delay).await;

                // Exponential backoff: Double the delay for the next attempt
                delay *= 2;
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    unreachable!() // This should never be reached because all cases are handled above.
}
