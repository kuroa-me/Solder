use tokio::{io, net::TcpListener, net::TcpStream, select};
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
        let (eyeball, _) = listener.accept().await?;
        let origin = TcpStream::connect(server).await?;

        // let (mut eread, mut ewrite) = eyeball.into_split();
        // let (mut oread, mut owrite) = origin.into_split();

        // let _e2o = tokio::spawn(async move { io::copy(&mut eread, &mut owrite).await });
        // let _o2e = tokio::spawn(async move { io::copy(&mut oread, &mut ewrite).await });

        zero_copy_bidirectional(&mut eyeball, &mut origin);
        // select! {
        //     _ = e2o => println!("e2o done"),
        //     _ = o2e => println!("o2e done"),
        // }
    }
}
