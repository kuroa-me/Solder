#![feature(unix_socket_ancillary_data)]
use libc;
use std::env;
use std::error::Error;
use std::io::{self, IoSliceMut};
use std::mem::{size_of, zeroed};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;

fn main() -> Result<(), Box<dyn Error>> {
    let listen_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:8080".to_string())
        .parse()?;
    let server_addr: SocketAddr = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1:8081".to_string())
        .parse()?;

    println!("Listening on: {}", listen_addr);
    println!("Proxying to: {}", server_addr);

    let sock = UdpSocket::bind(listen_addr)?;

    unsafe {
        let rc = cvt(libc::setsockopt(
            sock.as_raw_fd(),
            libc::IPPROTO_UDP,
            libc::UDP_GRO,
            &1 as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        ));
        if rc.is_err() {
            println!("Failed to enable GRO: {}", rc.unwrap_err());
        }
    }

    const MAX_SEGMENT_SIZE: usize = 64 * 1024 - 20 - 8;
    let mut buf = [0; MAX_SEGMENT_SIZE];
    let mut bufs = &mut [IoSliceMut::new(&mut buf)][..];

    let mut ancillary_buffer = [0; 128];
    let mut ancillary = SocketAncillary::new(&mut ancillary_buffer[..]);

    loop {
        let (size, _truncated, sender) =
            recv_vectored_with_ancillary_from(&sock, &mut bufs, &mut ancillary)?;

        println!("Received {} bytes from {:?}", size, sender);

        //     let (len, addr) = sock.recv_from(&mut buf)?;
        //     println!("{:?} bytes received from {:?}", len, addr);

        //     let len = sock.send_to(&buf[..len], server_addr)?;
        //     println!("{:?} bytes sent", len);
    }
}

fn recv_vectored_with_ancillary_from(
    socket: &UdpSocket,
    bufs: &mut [IoSliceMut<'_>],
    ancillary: &mut SocketAncillary<'_>,
) -> io::Result<(usize, bool, io::Result<SocketAddr>)> {
    unsafe {
        // Using sockadd_in6 here and hope it is compatible with both v4 and v6
        let mut msg_name: libc::sockaddr_storage = zeroed();
        let mut msg: libc::msghdr = zeroed();
        msg.msg_name = &mut msg_name as *mut _ as *mut _; //? why cast twice
        msg.msg_namelen = size_of::<libc::sockaddr_in6>() as libc::socklen_t;
        msg.msg_iov = bufs.as_mut_ptr().cast();
        msg.msg_iovlen = bufs.len() as _;
        msg.msg_controllen = ancillary.buffer.len() as _;
        // macos requires that the control pointer is null when the len is 0.
        if msg.msg_controllen > 0 {
            msg.msg_control = ancillary.buffer.as_mut_ptr().cast();
        }

        let count = cvt(libc::recvmsg(
            socket.as_raw_fd(),
            &mut msg as *mut _,
            libc::MSG_TRUNC,
        ))?;

        ancillary.length = msg.msg_controllen as usize;
        ancillary.truncated = msg.msg_flags & libc::MSG_TRUNC == libc::MSG_TRUNC;

        let truncated = msg.msg_flags & libc::MSG_TRUNC == libc::MSG_TRUNC;
        let addr = from_parts(msg_name, msg.msg_namelen);

        let mut cmsg = libc::CMSG_FIRSTHDR(&msg);

        while !cmsg.is_null() {
            if (*cmsg).cmsg_level == libc::SOL_UDP && (*cmsg).cmsg_type == libc::UDP_GRO {
                let grosizeptr = libc::CMSG_DATA(cmsg);
                let grosize = *grosizeptr as u16;
                println!("grosize: {}", grosize);
            }
            cmsg = libc::CMSG_NXTHDR(&msg, cmsg);
        }

        Ok((count as usize, truncated, addr))
    }
}

pub fn from_parts(addr: libc::sockaddr_storage, len: libc::socklen_t) -> io::Result<SocketAddr> {
    match addr.ss_family as libc::c_int {
        libc::AF_INET if len == size_of::<libc::sockaddr_in>() as libc::socklen_t => {
            let sock4 = unsafe { *(&addr as *const _ as *const libc::sockaddr_in) };
            Ok(SocketAddr::new(
                unsafe { *(&sock4.sin_addr as *const _ as *const Ipv4Addr) }.into(),
                sock4.sin_port.to_be(),
            ))
        }
        libc::AF_INET6 if len == size_of::<libc::sockaddr_in6>() as libc::socklen_t => {
            let sock6 = unsafe { *(&addr as *const _ as *const libc::sockaddr_in6) };
            Ok(SocketAddr::new(
                unsafe { *(&sock6.sin6_addr as *const _ as *const Ipv6Addr) }.into(),
                sock6.sin6_port.to_be(),
            ))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid sockaddr",
        )),
    }
}

#[derive(Debug)]
pub struct SocketAncillary<'a> {
    buffer: &'a mut [u8],
    length: usize,
    truncated: bool,
}

impl<'a> SocketAncillary<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        SocketAncillary {
            buffer,
            length: 0,
            truncated: false,
        }
    }
}

pub trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
    ($($t:ident)*) => ($(impl IsMinusOne for $t {
        fn is_minus_one(&self) -> bool {
            *self == -1
        }
    })*)
}

impl_is_minus_one! { i8 i16 i32 i64 isize }

pub fn cvt<T: IsMinusOne>(t: T) -> crate::io::Result<T> {
    if t.is_minus_one() {
        Err(crate::io::Error::last_os_error())
    } else {
        Ok(t)
    }
}
