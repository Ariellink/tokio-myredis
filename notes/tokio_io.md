# Tokio I/O

tokio中的I/O接口和std在使用方式上没有差别，但是std是同步的，tokio是异步的。tokio的读写trait分别是`AsyncRead`和`AsyncWrite`。

- `AsyncReadExt::read`:  是一个异步方法可以将数据读入缓冲区( buffer )中，然后返回读取的字节数。  
- `AsyncReadExt::read_to_end`: 方法会从字节流中读取所有的字节，直到遇到 EOF。  
- `AsyncWriteExt::write`: 异步方法会尝试将缓冲区的内容写入到写入器( writer )中，同时返回写入的字节数。
- `AsyncWriteExt::write_all`: 将缓冲区的内容全部写入到写入器中。  
- `tokio::io::copy()`: 异步的将读取器( reader )中的内容拷贝到写入器( writer )中.
    - `io::copy` ，它有两个参数：一个读取器，一个写入器，然后将读取器中的数据直接拷贝到写入器中，
    ```rust
    use tokio::fs::File;
    use tokio::io;

    #[tokio::main]
    async fn main() -> io::Result<()> {
        let mut reader: &[u8] = b"hello";
        let mut file = File::create("foo.txt").await?;
        //字节数组 &[u8] 实现了 AsyncRead,因此直接可以用作reader
        io::copy(&mut reader, &mut file).await?;
        Ok(())
    }
    ```
    - `io::split`: 任何一个读写器( reader + writer )都可以使用 io::split 方法进行分离，最终返回一个读取器和写入器，这两者可以独自的使用，例如可以放入不同的任务中。  

    </br>

### 为什么要在堆上分配缓冲区
A stack buffer is explicitly avoided. [link](https://tokio.rs/tokio/tutorial/spawning#send-bound) Tasks spawned by tokio::spawn must implement Send. This allows the Tokio runtime to move the tasks between threads while they are suspended at an `.await.` When .await is called, the task yields back to the scheduler. The next time the task is executed, it resumes from the point it last yielded. **To make this work, all state that is used `after` .await must be saved by the task.**

> We noted that all task data that lives across calls to `.await` must be stored by the task.In this case, buf is used across `.await` calls. All task data is stored in a single allocation. You can think of it as an enum where each variant is the data that needs to be stored for a specific call to `.await`.  
> 一个数据如果想在 .await 调用过程中存在，那它必须存储在当前任务内。在我们的代码中，buf 会在 .await 调用过程中被使用，因此它必须要存储在任务内。当任务因为调度在线程间移动时，存储在栈上的数据需要进行保存和恢复，过大的栈上变量会带来不小的数据拷贝开销。因此，存储大量数据的变量最好放到堆上。
 </br>

## Echo_server  
client.rs 使用io::copy and io::split  
```rust
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
```

server.rs

```rust
    use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
    use tokio::fs::File;
    use tokio::net::TcpListener;

    #[tokio::main]
    async fn main() -> io::Result<()>{
        let socket_lis = TcpListener::bind("127.0.0.1:6142").await?;
        
        loop {
            let (mut stream,_) = socket_lis.accept().await?;

            tokio::spawn( async move {
                //由于我们的读取器和写入器都是同一个 stream
                //任何一个读写器( reader + writer )都可以使用 io::split 方法进行分离，最终返回一个读取器和写入器，这两者可以独自的使用，例如可以放入不同的任务中。
                //不使用io::copy:
                //构造一个中转的堆上分配缓冲区： 如果想在 .await 调用过程中存在，那它必须存储在当前任务内。在我们的代码中，buf 会在 .await 调用过程中被使用，因此它必须要存储在任务内。
                //当任务因为调度在线程间移动时，存储在栈上的数据需要进行保存和恢复，
                let mut buf = vec![0;1024];
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
                        },
                        Err(e) => {
                            println!("error: {:?}",e);
                            return;
                        },
                    }
                }
            });
        }
    }
```

*reference: * 
https://course.rs/advance-practice/io.html#%E4%BD%BF%E7%94%A8-iocopy
