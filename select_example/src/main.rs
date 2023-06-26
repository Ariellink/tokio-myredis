// use tokio::sync::oneshot;

// #[tokio::main()]
// async fn main() {
//     // prepare 2 channels in tokio onshot
//     let (tx1,rx1) = oneshot::channel();
//     let (tx2, rx2) = oneshot::channel();

//     // spawn tokio tasks to sent the message
//     tokio::spawn(async{
//         let _ = tx1.send("One");
//     });
//     tokio::spawn(async{
//         let _ = tx2.send("Two");
//     });
//     // bind the val to the rx1 and rx2
//     // select! macro will wait for the first one to complete
//     tokio::select! {
//         val1 = rx1 => {
//             println!("rx1 completed first with {:?}", val1);
//         }
//         val2 = rx2 => {
//             println!("rx2 completed first with {:?}",val2);
//         }
//     }
// }

use std::io;
use tokio::{
    net::TcpListener,
    sync::oneshot,
};

fn process(_socket: tokio::net::TcpStream) {
    println!("processing socket");
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // [setup `rx` oneshot channel]
    let (tx, rx): (oneshot::Sender<()>, oneshot::Receiver<()>) = oneshot::channel();
    let listener = TcpListener::bind("localhost:3465").await?;
    //tx.send(()).unwrap();
    tokio::select! {
        res = async {
            loop {
                let (socket, _) = listener.accept().await?;
                tokio::spawn(async move { process(socket) });
            }

            // Help the rust type inferencer out
            Ok::<_, io::Error>(())
        } => {
            res?;
        }
        _ = rx => {
            println!("terminating accept loop");
        }
    }

    Ok(())
}
