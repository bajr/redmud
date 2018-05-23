use bytes::BytesMut;
use tokio::io;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::prelude::*;

use shared::*;

use super::Rx;

// https://github.com/carllerche/bytes/issues/193
// TODO Place bounds on the buffers and stop leaking memory?
// TODO Do I need to validate that the buffers contain valid utf-8?
// TODO Set a reasonable limit for reading input and alert player if their input was truncated

// I am splitting up the socket into read and write so I can poll them asynchronously
#[derive(Debug)]
pub struct RecvLines {
    pub insock: ReadHalf<TcpStream>, // The read half of the TCP socket
    rd: BytesMut,                    // Internal read buffer
}

impl RecvLines {
    pub fn new(insock: ReadHalf<TcpStream>) -> Self {
        RecvLines {
            insock,
            rd: BytesMut::new(),
        }
    }

    // Read data from the socket. This only returns `Ready` when the socket has closed.
    fn fill_read_buf(&mut self) -> Poll<(), io::Error> {
        loop {
            // Reserve capacity for the buffer. This might result in an internal allocation.
            self.rd.reserve(1024);

            // Read data into the buffer.
            let n = try_ready!(self.insock.read_buf(&mut self.rd));

            if n == 0 {
                return Ok(Async::Ready(()));
            }
        }
    }
}

// This future ends when the socket is closed...
impl Stream for RecvLines {
    type Item = BytesMut;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // First, read any new data that might have been received off the socket
        let sock_closed = self.fill_read_buf()?.is_ready();

        // Now, try finding lines
        let pos = self.rd.windows(2).position(|bytes| bytes == b"\r\n");

        if let Some(pos) = pos {
            // Return the line up to the matched "\r\n"
            let mut line = self.rd.split_to(pos + 2);
            line.split_off(pos);
            return Ok(Async::Ready(Some(line)));
        }

        if sock_closed {
            Ok(Async::Ready(None))
        } else {
            Ok(Async::NotReady)
        }
    }
}

#[derive(Debug)]
pub struct SendLines {
    pub outsock: WriteHalf<TcpStream>, // The write half of the TCP socket
    rx: Rx,
}

// I spent an inordinate amount of time trying to figure out how to turn the outbound
// socket/channel into an Evented object so that the tokio reactor could run poll whenever there
// was anything to write out to them. I was unsuccessful and unable to find help.
impl SendLines {
    pub fn new(mut outsock: WriteHalf<TcpStream>, rx: Rx) -> Self {
        // I acknowledge that doing this will block the thread until the socket is Ready, which is
        // not in the spirit of Asynchronous I/O. But I could not find any other way of making sure
        // the player was given this prompt consistently.
        while let Result::Ok(Async::NotReady) = outsock.poll_write(SPLASH) {}
        while let Result::Ok(Async::NotReady) = outsock.poll_flush() {}

        SendLines { outsock, rx }
    }
}

// This future should end when the user closes the connection.
impl Future for SendLines {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        // Tokio (and futures) use cooperative scheduling without any preemption.
        // If a task never yields execution back to the executor, then other tasks may be starved.
        //
        // To deal with this, robust applications should not have any unbounded loops.
        // So we will read at most `LINES_PER_TICK` lines from the client on each tick.
        const LINES_PER_TICK: usize = 10;

        // Receive all messages from peers.
        for i in 0..LINES_PER_TICK {
            // Polling an `UnboundedReceiver` cannot fail, so `unwrap` here is safe.
            match self.rx.poll().unwrap() {
                Async::Ready(Some(v)) => {
                    let _ = self.outsock.poll_write(&v); // Buffer the line
                    let _ = self.outsock.poll_flush();

                    // If the limit is hit, the current task is notified, informing the executor to
                    // schedule the task again.
                    if i + 1 == LINES_PER_TICK {
                        task::current().notify();
                    }
                }
                _ => break,
            }
        }

        return Ok(Async::NotReady);
    }
}
