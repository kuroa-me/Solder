#![feature(unix_socket_ancillary_data)]
use listenfd::ListenFd;
use std::io::IoSliceMut;
use std::os::fd::FromRawFd;
use std::os::unix::net::{AncillaryData, SocketAncillary, UnixListener, UnixStream};
use tokio::{io, net::TcpStream};

// const SCM_MAX_FD: usize = 253;
// const SYSTEMD_SOCKET: &str = "/tmp/sam_sock";
const NON_SYSTEMD_SOCKET: &str = "/tmp/fragile_sock";

async fn proxy_tcp(up: i32, down: i32) -> io::Result<()> {
    //TODO: make a better socket conversion

    let mut up = TcpStream::from_std(unsafe { std::net::TcpStream::from_raw_fd(up) })?;
    let mut down = TcpStream::from_std(unsafe { std::net::TcpStream::from_raw_fd(down) })?;

    let bi = tokio::spawn(async move { io::copy_bidirectional(&mut up, &mut down).await });

    bi.await??;

    return Ok(());
}

async fn handle_client(sock: UnixStream) -> std::io::Result<()> {
    // let mut fds = [0; 8];
    let mut ancillary_buffer = [0; 128];
    let mut ancillary = SocketAncillary::new(&mut ancillary_buffer[..]);

    let mut buf = [1; 8];
    let bufs = &mut [IoSliceMut::new(&mut buf[..])][..];
    sock.recv_vectored_with_ancillary(bufs, &mut ancillary)?;

    let mut fd_pair: [i32; 2] = [0; 2];

    for ancillary_result in ancillary.messages() {
        if let AncillaryData::ScmRights(scm_rights) = ancillary_result.unwrap() {
            for (i, fd) in scm_rights.enumerate() {
                println!("receive file descriptor: {fd}");
                if i >= 2 {
                    break;
                }
                fd_pair[i] = fd;
            }
            proxy_tcp(fd_pair[0], fd_pair[1]).await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();
    let lis: UnixListener;

    if let Some(listener) = listenfd.take_unix_listener(0)? {
        lis = listener;
    } else {
        let _ = std::fs::remove_file(NON_SYSTEMD_SOCKET);
        lis = UnixListener::bind(NON_SYSTEMD_SOCKET)?;
    }

    for stream in lis.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream).await?;
            }
            Err(err) => {
                println!("accept error: {}", err);
                break;
            }
        }
    }

    Ok(())
}
