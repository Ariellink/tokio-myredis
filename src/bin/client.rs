use mini_redis::client;
use tokio::sync::{mpsc, oneshot};
use bytes::Bytes;


#[derive(Debug)]
enum Command {
    Get {
        key: String,
        //接收server resoonse的发送端
        resp: oneshot::Sender<mini_redis::Result<Option<Bytes>>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: oneshot::Sender<mini_redis::Result<()>>,
    },
}

#[tokio::main()]
async fn main() {
    let (tx1, mut rx) = mpsc::channel(32);
    

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        
        while let Some(message) = rx.recv().await {
            use Command::*;

            match message {
                Get {key,resp} => {
                    //从server端拿到的返回res
                    let res = client.get(&key).await;
                    //将res通过Get的sender: resp发送出去
                    let _ = resp.send(res);
                }
                Set {key, val,resp} => {
                    //从server端拿到的返回res
                    let res = client.set(&key,val).await;
                    //将res通过Set的sender: resp发送出去
                    let _ = resp.send(res);
                }
            }
        }
    });
    
    let tx2 = tx1.clone();
    let task1 = tokio::spawn(async move {
        let (resp_tx,resp_rx) = oneshot::channel();
        let message = Command::Get { 
            key: "foo".to_string(),
            resp: resp_tx,
         };

        //发送Get请求
        tx1.send(message).await.unwrap();

        //等待server回复
         let res_of_get = resp_rx.await;
         println!("GOT Result = {:?}", res_of_get);
    });

    let task2 = tokio::spawn(async move{
        let (resp_tx, resp_rx) = oneshot::channel();
        //构造消息
        let message = Command::Set { key: "hello".to_string(), val: "world".into(), resp: resp_tx};

        //发送消息到manager
        tx2.send(message).await.unwrap();

        //等待接收消息
        let res_of_set = resp_rx.await;

        println!("SET Result = {:?}", res_of_set)
    });

    //set
    task2.await.unwrap();
    //get
    task1.await.unwrap();
    manager.await.unwrap();
}
