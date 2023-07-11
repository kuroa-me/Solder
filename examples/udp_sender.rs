use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::os::fd::AsRawFd;
use tokio::net::UdpSocket;

fn get_random_data(buf: &mut [u8]) -> Result<usize, Box<dyn Error>> {
    let path = "/dev/urandom";
    // TODO: use async file read and zero-copy
    let mut file = File::open(path)?;
    let n = file.read(buf)?;
    Ok(n)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let remote_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "192.0.2.2:8080".into())
        .parse()?;

    let local_addr: SocketAddr = if remote_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    }
    .parse()?;

    let socket = UdpSocket::bind(local_addr).await?;

    unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            libc::IPPROTO_IP,
            libc::IP_MTU_DISCOVER,
            &libc::IP_PMTUDISC_PROBE as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
    }

    const MAX_DATAGRAM_SIZE: usize = 65507;
    socket.connect(&remote_addr).await?;

    // Get MTU
    let mut mtu: u32 = 0;
    unsafe {
        libc::getsockopt(
            socket.as_raw_fd(),
            libc::IPPROTO_IP,
            libc::IP_MTU,
            &mut mtu as *mut _ as *mut libc::c_void,
            &mut std::mem::size_of::<libc::c_int>() as *mut _ as *mut libc::socklen_t,
        );
    }

    // let gso_size: u32 = mtu - 20 - 8;
    let gso_size: u32 = 1500 - 20 - 8;

    println!("MTU: {}, GSO size: {}", mtu, gso_size);

    // unsafe {
    //     libc::setsockopt(
    //         socket.as_raw_fd(),
    //         libc::SOL_UDP,
    //         libc::UDP_SEGMENT,
    //         &gso_size as *const _ as *const libc::c_void,
    //         std::mem::size_of::<libc::c_int>() as libc::socklen_t,
    //     );
    // }

    let mut buf = [0u8; MAX_DATAGRAM_SIZE];
    let mut sliced_buf = &mut buf[..gso_size as usize];
    loop {
        let n = get_random_data(&mut sliced_buf)?;
        socket.send(&mut sliced_buf[..n]).await?;
        // println!("Tried to send {} bytes, sent {} bytes", n, m);
    }
}
