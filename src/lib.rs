use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use std::os::unix::io::{AsRawFd, RawFd};

use ringbahn::Submission;
use ringbahn::{Driver, Event, Read, DRIVER};

struct FdWrapper(RawFd);
impl AsRawFd for FdWrapper {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

// TODO: we can probably get creative with VecDeque and Vec::split_off to reuse memory.
pub struct File {
    file: async_std::fs::File,
    buf: Option<Vec<u8>>, // we sometimes need to give this away in order to
    read_sub: Option<Submission<Read<'static, File>, Driver>>,
    cursor: usize,
}

impl File {
    pub async fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
        let file = async_std::fs::File::open(path.as_ref()).await?;
        Ok(Self {
            file,
            buf: None,
            read_sub: None,
            cursor: 0,
        })
    }
}

impl async_std::io::Read for File {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.file).poll_read(cx, buf)
    }
}

impl async_std::io::BufRead for File {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<&[u8]>> {
        if let None = self.read_sub {
            let fd = FdWrapper(self.file.as_raw_fd());
            let read = Read::new(&mut fd, self.buf.take().unwrap());
            self.read_sub = Some(read.submit(&DRIVER));
        }
        todo!();
    }
    fn consume(self: Pin<&mut Self>, amt: usize) {
        todo!();
    }
}

pub trait BufWrite {
    fn poll_write_buf(self: Pin<&mut Self>, cx: &mut Context, buf: Vec<u8>)
        -> Poll<io::Result<()>>;
}
