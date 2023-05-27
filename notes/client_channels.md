# Client 异步发送请求，接收response [tokio Channel]

- 消息通道缓冲区

## tokio 中的channel
`use tokio::sync::mpsc;`  
`use tokio::sync::oneshot;`

使用tokio的mpsc channel创建接收端和发送端。用于生成管理任务。`manager`

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // 创建一个新通道，缓冲队列长度是 32
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let manager = tokio::spawn(async move {
        //与server建立连接
        let mut client = Client::connect("127.0.01:6379").await.unwrap();
        //rx.recv()来接收task生成发过来的消息
        //匹配对应的command
        //向server端发出command
        while let Some(message) = rx.recv().await {
            match message {
                Get{key} => {
                    client.get(...).await;
                },
                Set{key,val} => {
                    client.set(...).await;
                }
            }
        }
        //TODO:
        //接收server发过来的消息，打印在客户端
    });


}
```

构造两个任务（GET， SET）
```rust
    let tx2 = tx1.clone();
    let task1 = tokio::spawn(async move {
        let message = Command::Get { key: "hello".to_string() };
         tx1.send(message).await.unwrap();
    });

    let task2 = tokio::spawn(async move{
        let message = Command::Set { key: "foo".to_string(), val: "bar".into() };
        tx2.send(message).await.unwrap();
    });

    task1.await.unwrap();
    task2.await.unwrap();
} //the end of main
```
> **TODO:**
> 接收server发过来的response，打印在客户端。  

使用oneshot: 一发一收，改造manager。  
为了让server response --> manager --> task1返回到发送者手中，这个管道的发送端必须要随着命令一起发送。一个比较好的实现就是将管道的发送端放入 Command 的数据结构中。
```rust
use tokio::sync::oneshot;

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
//更新manager
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

// 任务 发送manger command, 接收server端消息
    
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

} // fn main

```  

在 Tokio 中我们必须要显式地引入并发和队列:

    tokio::spawn
    select!
    join!
    mpsc::channel
