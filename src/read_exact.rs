use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::{Context, Poll};

use crate::cipher::Cipher;
use crate::util::eof;

use tokio::io::{AsyncRead, ReadBuf};

pub struct DecryptReadExact<'a, A: ?Sized> {
    cipher: Arc<Mutex<Cipher>>,
    reader: &'a mut A,
    buf: &'a mut [u8],
    pos: usize,
}

pub fn read_exact<'a, A>(
    cipher: Arc<Mutex<Cipher>>,
    reader: &'a mut A,
    buf: &'a mut [u8],
) -> DecryptReadExact<'a, A>
where
    A: AsyncRead + Unpin + ?Sized,
{
    DecryptReadExact {
        cipher,
        reader,
        buf,
        pos: 0,
    }
}

impl<A> Future for DecryptReadExact<'_, A>
where
    A: AsyncRead + Unpin + ?Sized,
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        let me = &mut *self;
        let mut cipher = me.cipher.lock().unwrap();
        if cipher.dec.is_none() {
            while me.pos < cipher.iv.len() {
                let mut buf = ReadBuf::new(&mut cipher.iv[me.pos..]);
                ready!(Pin::new(&mut *me.reader).poll_read(cx, &mut buf))?;
                let n = buf.filled().len();
                me.pos += n;
                if n == 0 {
                    return Err(eof()).into();
                }
            }
            me.pos = 0;
            cipher.init_decrypt();
        };

        // if our buffer is empty, then we need to read some data to continue.
        while me.pos < me.buf.len() {
            let mut buf = ReadBuf::new(&mut cipher.iv[me.pos..]);
            ready!(Pin::new(&mut *me.reader).poll_read(cx, &mut buf))?;
            let n = buf.filled().len();
            me.pos += n;
            if n == 0 {
                return Err(eof()).into();
            }
        }

        let copy_len = me.buf.len();
        cipher.decrypt(&mut me.buf[..copy_len]);
        Poll::Ready(Ok(me.pos))
    }
}
