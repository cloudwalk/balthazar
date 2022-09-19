use std::{
    fmt::Display,
    future::Future,
    time::{Duration, Instant},
};

use eyre::Error;
use serde::Serialize;
use tokio::time::timeout;

#[derive(Debug, Serialize)]
pub struct HealthStatusReport {
    pub status: HealthStatus,
    pub duration: Duration,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Offline { error: String },
}

impl Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Offline { error: _ } => write!(f, "offline"),
        }
    }
}

impl HealthStatusReport {
    pub async fn check_with_timeout_and_degrade<F>(
        fut: F,
        timeout_ms: u64,
        degrade_ms: u64,
    ) -> HealthStatusReport
    where
        F: Future<Output = Result<(), Error>>,
    {
        let start = Instant::now();

        let status = match timeout(Duration::from_millis(timeout_ms), fut).await {
            Ok(result) => match result {
                Ok(()) => {
                    let elapsed = start.elapsed();

                    if elapsed > Duration::from_millis(degrade_ms) {
                        HealthStatus::Degraded
                    } else {
                        HealthStatus::Healthy
                    }
                }
                Err(e) => HealthStatus::Offline {
                    error: e.to_string(),
                },
            },
            Err(e) => HealthStatus::Offline {
                error: e.to_string(),
            },
        };
        HealthStatusReport {
            status,
            duration: start.elapsed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use eyre::eyre;
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn health_status_check() {
        let future = || async move {
            sleep(Duration::from_millis(1)).await;
            Ok::<(), eyre::Error>(())
        };

        // timeout_ms = 10, degraded_ms = 5
        // future sleeps for 1 ms so should be healthy
        let report = HealthStatusReport::check_with_timeout_and_degrade(future(), 10, 5).await;
        assert!(matches!(report.status, HealthStatus::Healthy));

        // timeout_ms = 10, degraded_ms = 0
        // future sleeps for 1 ms so should be degraded
        let report = HealthStatusReport::check_with_timeout_and_degrade(future(), 10, 0).await;
        assert!(matches!(report.status, HealthStatus::Degraded));

        // timeout_ms = 0, degraded_ms = 10
        // future sleeps for 1 ms so should be offline with matching message
        let report = HealthStatusReport::check_with_timeout_and_degrade(future(), 0, 10).await;
        assert!(matches!(report.status, HealthStatus::Offline { error: _ }));
        assert_eq!(
            report.status,
            HealthStatus::Offline {
                error: "deadline has elapsed".to_string()
            }
        );

        let future = || async move {
            sleep(Duration::from_millis(1)).await;
            Err::<(), eyre::Error>(eyre!("test error!".to_string()))
        };
        // timeout_ms = 10, degraded_ms = 5
        // future sleeps for 1 ms but return an error, so should be offline with matching message
        let report = HealthStatusReport::check_with_timeout_and_degrade(future(), 10, 5).await;
        assert!(matches!(report.status, HealthStatus::Offline { error: _ }));
        assert_eq!(
            report.status,
            HealthStatus::Offline {
                error: "test error!".to_string()
            }
        );
    }
}
