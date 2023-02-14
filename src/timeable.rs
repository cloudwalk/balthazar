use metrics::{describe_histogram, histogram};
use std::future::Future;
use tokio::time::Instant;

static mut METRIC_NAME: String = String::new();

/// Inits the `timeable` module, creating and describing the metrics that will be tracked.
pub fn init(service_name: &str) {
    let metric_name = format!("{}_task_duration_ms", service_name);

    describe_histogram!(
        metric_name.clone(),
        "Task execution duration in milliseconds."
    );
    unsafe {
        METRIC_NAME = metric_name;
    }
}

/// Tracks execution duration of futures.
#[crate::async_trait]
pub trait Timeable<T> {
    async fn time_as<S: Into<String> + Send>(self, task_name: S) -> T;
}

/// Tracks execution duration of futures using metrics histograms.
#[crate::async_trait]
impl<Fut, Res> Timeable<Res> for Fut
where
    Fut: Future<Output = Res> + Send,
{
    async fn time_as<S: Into<String> + Send>(self, task_name: S) -> Res {
        let start = Instant::now();
        let result = self.await;
        let duration = Instant::now() - start;

        unsafe {
            histogram!(METRIC_NAME.as_ref(), duration.as_millis() as f64, "task" => task_name.into())
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::timeable::Timeable;

    #[tokio::test]
    async fn timeable_tracks_any_future() {
        async fn async_task() -> usize {
            1
        }
        let res = async_task().time_as("test").await;
        assert_eq!(res, 1);
    }

    #[tokio::test]
    async fn timeable_tracks_result_ok() {
        async fn async_task() -> Result<usize, ()> {
            Ok(1)
        }
        let res = async_task().time_as("test").await;
        assert_eq!(res, Ok(1));
    }

    #[tokio::test]
    async fn timeable_tracks_result_err() {
        async fn async_task() -> Result<(), usize> {
            Err(1)
        }
        let res = async_task().time_as("test").await;
        assert_eq!(res, Err(1));
    }
}
