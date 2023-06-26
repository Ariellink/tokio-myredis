# Implement shutdown in asynchronous applications

- 搞清楚何时shut down
- 告诉程序的每个部分去shut down
- 等待程序的每个部分shut down

## tokio::signal::ctrl_c
当应用收到来自于操作系统的关闭信号时。例如通过 `ctrl + c` 来关闭正在运行的命令行程序。
为了检测来自操作系统的关闭信号，`Tokio` 提供了一个 `tokio::signal::ctrl_c` 函数，他将一直睡眠直到收到对应的信号：
```rust
use tokio::signal;

#[tokio::main]
async fn main() {
	...
	match signal::ctrl_c().await {
		Ok(()) => {},
		Err(e) => {
			eprintln!("Unable to listen for shutdown signal: {}", err);
			// we also shut down in case of error
		},
	}
	// send shutdown signal to application and wait
}
```
## mpsc channel receiving shutdown signal
`self.connection.read_frame()`方法，该方法返回一个`Future`对象，表示从连接中读取一个帧（frame）的异步操作。
- 当这个异步操作完成时，将返回结果保存在`res`变量中。-> `next_frame = res?`
- 使用`?`运算符将`res`结果进行错误处理。如果`res`是一个错误值，那么整个`async fn`函数将返回这个错误。
`tokio::select!`宏会等待两个分支中的任意一个完成。如果第一个分支中的异步操作完成，它的结果将被绑定到`res`变量。`next_frame = res?`

`_ = self.shutdown.recv() => { return Ok(()); }`这个分支等待`self.shutdown.recv()`方法的完成，该方法返回一个`Future`对象，表示从`shutdown`通道中接收一个消息的异步操作。
-   当接收到关闭信号时，执行花括号内的代码块，其中的`return Ok(());`语句会直接从`async fn`函数中返回`Ok(())`，即返回一个成功的`Result`。

**Summary**
即如果第一个分支中的异步操作完成，`next_frame`将获得`self.connection.read_frame()`的结果，表示成功读取到一个帧。如果接收到关闭信号，整个异步函数将直接返回成功的`Result`并结束执行。这样可以灵活地处理不同的异步事件和退出条件。
```rust
use tokio::signal;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (shutdown_send, shutdown_recv) = mpsc::unbounded_channel();

    // ... spawn application as separate task ...
    //
    // application uses shutdown_send in case a shutdown was issued from inside
    // the application

    let next_frame = tokio::select! {
		res = self.connection.read_frame() => res?,
        _ = signal::ctrl_c() => {
			// 当收到关闭信号后，直接从 `select!` 返回，此时 `select!` 中的另一个分支会自动释放，其中的任务也会结束
	        return Ok(());
        },
        _ = shutdown_recv.recv() => {
	        return Ok(());
        },
    }

    // send shutdown signal to application and wait
}
```
## `CancellationToken`
When you want to tell one or more tasks to shut down, you can use [Cancellation Tokens](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html).

## drop senders
[`mpsc`](https://docs.rs/tokio/1/tokio/sync/mpsc/index.html) 消息通道有一个重要特性：当所有发送端都 `drop` 时，消息通道会自动关闭，此时继续接收消息就会报错。

Rust 消息通道关闭的两个条件：所有发送者全部被`drop`或接收者被`drop`。如果不`drop(sender)`, `recv`会始终阻塞。

> 这个特性特别适合优雅关闭的场景：主线程持有消息通道的接收端，然后每个代码部分拿走一个发送端，当该部分结束时，就 `drop` 掉发送端，因此所有发送端被 `drop` 也就意味着所有的部分都已关闭，此时主线程的接收端就会收到错误，进而结束。

```rust
use tokio::sync::mpsc::{channel, Sender};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (send, mut recv) = channel(1);

	let mut tasks = Vec::new();
	//使用`tokio::spawn`将10个异步任务放入Tokio的运行时中执行。这些任务是并发执行的，它们可能会在`main`函数继续执行之前就完成。
    for i in 0..10 {
        let task = tokio::spawn(some_operation(i, send.clone()));
		tasks.push(task);
    }
    //在`some_operation`函数的最后，`sender`对象会超出作用域
	
    // 等待各个任务的完成
    //如果不等待任务的完成，而直接继续执行`let _ = recv.recv().await;`等待接收端接收消息，那么可能会导致接收端在所有任务完成之前就返回了，从而无法获取到所有任务的结果。
	 let _ = tokio::try_join_all(tasks).await;
    // 我们需要 drop 自己的发送端，因为等下的 `recv()` 调用会阻塞, 如果不 `drop` ，那发送端就无法被全部关闭, `recv` 也将永远无法结束，这将陷入一个类似死锁的困境
    //通过`drop(send);`显式地丢弃了`sender`对象，使其超出作用域，从而关闭了通道。这样做是为了告诉接收端没有更多的消息会发送过来。
    drop(send);

    // 当所有发送端都超出作用域被 `drop` 时 (当前的发送端并不是因为超出作用域被 `drop` 而是手动 `drop` 的),`recv` 调用会返回一个错误这里使用了`let _ = recv.recv().await;`来忽略这个错误。
    // 这样就不会recv()阻塞
    let _ = recv.recv().await; 
}

async fn some_operation(i: u64, _sender: Sender<()>) {
    sleep(Duration::from_millis(100 * i)).await;
    println!("Task {} shutting down.", i);

    // 发送端超出作用域，然后被 `drop`
}
```

