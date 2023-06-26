# 在同步代码中使用一小部分异步代码

## [tokio::main]的展开
#[tokio::main]将`async fn main`函数替换为：
```rust
fn main() {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(async {
			/*异步代码*/
		})
}
```
`runtime.block_on(/*async func*)`: 异步代码中等待一个`Future`完成并返回其结果。`block_on`函数会阻塞当前线程，直到传入的`Future`完成。
```rust
let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
        
    // 在tokio runtime上调用block_on方法阻塞当前线程直到future执行完毕
    let inner = runtime.block_on(client::connect(addr))?;
```

`current_thread`和`multi_thread`

- `multi_thread`: 【默认】生成多个运行在后台的线程，它们可以高效的实现多个任务的同时并行处理。
- `current_thread`: 运行时并不生成新的线程，只是运行在已有的主线程上。因此只有当 `block_on` 被调用后，该运行时才能执行相应的操作。。一旦 `block_on` 返回，那运行时上所有生成的任务将再次冻结，直到 `block_on` 的再次调用。在同一时间点只需要做一件事，无需并行处理时使用。

[`enable_all`](https://docs.rs/tokio/1.16.1/tokio/runtime/struct.Builder.html#method.enable_all) 方法调用，它可以开启 `Tokio` 运行时提供的 IO 和定时器服务。

**tokio提供了几种方法在同步代码中运行异步代码:**
- `runtime.block_on()`
- `runtime.spawn()`:  通过 `Runtime` 的 `spawn` 方法来创建一个基于该运行时的后台任务。
- 使用消息传递的方式与runtime进行交互


## 使用block_on()在同步代码中运行异步代码
```rust
impl BlockingClient {
    //构造函数
    //建立一个到redis server的连接
    pub fn connect<T: ToSocketAddrs>(addr: T) -> Result<BlockingClient> {
        // 创建一个单线程的tokio runtime
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        // 在tokio runtime上调用block_on方法阻塞当前线程直到future执行完毕
        // 使用这个runtime来调用异步的connect连接方法
        let inner = runtime.block_on(client::connect(addr))?;
        Ok(BlockingClient { inner, runtime })
    }

    //同步接口：通过 block_on 将异步形式的 Client 的方法变成同步调用的形式。
    pub fn get(&mut self, key: &str) -> Result<Option<Bytes>> {
        self.runtime.block_on(self.inner.get(key))
    }
```

## runtime.spawn()

`spawn` 方法返回一个 `JoinHandle`，它是一个 `Future`，因此可以通过  `block_on` 来等待它完成

```rust
asyn fn async_task() {...}

fn main() {
	//创建一个多线程的Tokio运行时
	let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
    //创建一个`handles`向量来存储后台任务的`JoinHandle`。
	let mut handles = Vec::with_capacitu(10);
	//使用`for`循环创建10个后台任务，并将它们的`JoinHandle`添加到`handles`向量中。
-   在后台任
	for i in 0..10 {
		handles.push(runtime.spawn(async_task(i)));
	}
	/*耗时工作*/
	for handles in handles {
		//等待异步任务的完成
		runtime.block_on(handle).unwarp();
	}
}
```