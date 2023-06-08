#![feature(unix_socket_ancillary_data)]
use std::io::IoSlice;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{SocketAncillary, UnixStream};

fn main() -> std::io::Result<()> {
    let client = "127.0.0.1:20000";
    let server = "127.0.0.1:20001";
    let listener = TcpListener::bind(client)?;
    let (eyeball, _) = listener.accept()?;
    let origin = TcpStream::connect(server)?;

    let sock = UnixStream::connect("/tmp/fragile_sock")?;

    let mut ancillary_buffer = [0; 128];
    let mut ancillary = SocketAncillary::new(&mut ancillary_buffer[..]);
    ancillary.add_fds(&[eyeball.as_raw_fd(), origin.as_raw_fd()][..]);

    let buf = [1; 8];
    let bufs = &mut [IoSlice::new(&buf[..])][..];
    sock.send_vectored_with_ancillary(bufs, &mut ancillary)?;
    Ok(())
}
