use tokio::{
    fs::File,
    io::{
        self,
        AsyncReadExt,
        AsyncWriteExt,
    },
    net::TcpListener,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket_lis = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut stream, _) = socket_lis.accept().await?;

        tokio::spawn(async move {
            //由于我们的读取器和写入器都是同一个 stream
            //任何一个读写器( reader + writer )都可以使用 io::split
            // 方法进行分离，最终返回一个读取器和写入器，这两者可以独自的使用，
            // 例如可以放入不同的任务中。 不使用io::copy:
            //构造一个中转的堆上分配缓冲区： 如果想在 .await
            // 调用过程中存在，那它必须存储在当前任务内。在我们的代码中，buf 会在 .await
            // 调用过程中被使用，因此它必须要存储在任务内。
            // 当任务因为调度在线程间移动时，存储在栈上的数据需要进行保存和恢复，
            let mut buf = vec![0; 1024];
            loop {
                //stream全部读进buffer中
                match stream.read(&mut buf).await {
                    //当 TCP 连接的读取端关闭后，再调用 read 方法会返回 Ok(0)
                    //忘记在 EOF 时退出读取循环，是网络编程中一个常见的 bug
                    Ok(0) => return,
                    Ok(n) => {
                        if stream.write_all(&buf[0..n]).await.is_err() {
                            return;
                        }
                    }
                    Err(e) => {
                        println!("error: {:?}", e);
                        return;
                    }
                }
            }
        });
    }
}
