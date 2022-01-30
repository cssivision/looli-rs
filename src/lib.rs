macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

pub mod args;
pub mod cipher;
pub mod config;
mod copy;
pub mod io;
pub mod listener;
pub mod local;
mod read_exact;
pub mod resolver;
pub mod server;
pub mod socks5;
pub mod util;
mod write_all;
