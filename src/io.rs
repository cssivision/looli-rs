use std::future::Future;
use std::io;
use std::ops::{Add, Sub};
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project_lite::pin_project;
use tokio::time::{sleep, Duration, Instant, Sleep};

pub use crate::copy::copy_bidirectional;
pub use crate::read_exact::read_exact;
pub use crate::write_all::write_all;

pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const DEFAULT_VISITED_GAP: Duration = Duration::from_secs(3);

pin_project! {
    /// A future with timeout time set
    pub struct IdleTimeout<S: Future> {
        #[pin]
        inner: S,
        #[pin]
        sleep: Sleep,
        idle_timeout: Duration,
        last_visited: Instant,
    }
}

impl<S: Future> IdleTimeout<S> {
    pub fn new(inner: S, idle_timeout: Duration) -> Self {
        let sleep = sleep(idle_timeout);

        Self {
            inner,
            sleep,
            idle_timeout,
            last_visited: Instant::now(),
        }
    }
}

impl<S: Future> Future for IdleTimeout<S> {
    type Output = io::Result<S::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(v) => Poll::Ready(Ok(v)),
            Poll::Pending => match Pin::new(&mut this.sleep).poll(cx) {
                Poll::Ready(_) => Poll::Ready(Err(io::ErrorKind::TimedOut.into())),
                Poll::Pending => {
                    let now = Instant::now();
                    if now.sub(*this.last_visited) >= DEFAULT_VISITED_GAP {
                        *this.last_visited = now;
                        this.sleep.reset(now.add(*this.idle_timeout));
                    }
                    Poll::Pending
                }
            },
        }
    }
}
