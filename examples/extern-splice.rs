use tokio::{io, net::TcpListener, net::TcpStream};
use tokio_splice::zero_copy_bidirectional;

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = "127.0.0.1:20000";
    let server = "127.0.0.1:20001";
    proxy(client, server).await
}

async fn proxy(client: &str, server: &str) -> io::Result<()> {
    let listener = TcpListener::bind(client).await?;
    loop {
        let (mut eyeball, _) = listener.accept().await?;
        let mut origin = TcpStream::connect(server).await?;

        tokio::spawn(async move { zero_copy_bidirectional(&mut eyeball, &mut origin).await });
        // select! {
        //     _ = _e2o => println!("e2o done"),
        //     _ = _o2e => println!("o2e done"),
        // }
    }
}
