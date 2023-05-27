use tokio::io::{self, AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()>{
    let sockect_stream = TcpStream::connect("127.0.0.1:6142").await?;
    let (mut rd, mut wr) = io::split(sockect_stream);

    tokio::spawn(async move {
        wr.write_all(b"hello\n").await?;
        wr.write_all(b"world!\n").await?; 
        
        Ok::<_, io::Error>(())
    });

    let mut buf = vec![0,128];
    let mut message = String::new();
    loop {
        let readbytes = rd.read(&mut buf).await?;

        if readbytes == 0 {
            break;
        }
        let received_data = String::from_utf8_lossy(&buf[0..readbytes]);

        message.push_str(&received_data);

        if message.ends_with('\n') {
            println!("Got: {}", message.trim_end());
            message.clear();
        }
        //println!{"Got: {:?}", received_data }
    }
    Ok(())
}

