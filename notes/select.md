# tokio::select

`select!` is a macro that allows you to wait on multiple futures at the same time. It is similar to `Future::select` in that it returns the index of the future that completes first, but it also allows you to wait on multiple futures at the same time.
`select!` 是允许你同时等待多个furtues的完成的macro。既可以用作在得到最先完成的future就返回，也可以同时等待多个furtures完成。?

```rust
tokio::select! {
//<模式> = <async 表达式> => <结果处理>
//<pattern> = <async expression> => <handler>
 val1 = rx1 => { /*do something*/ }
 val2 = rx2 => { /*do something*/ }
}
```
- `async表达式` 就是要执行的异步任务，结果处理就是当异步任务完成后，要执行的代码。
当` select!`宏开始执行后，所有的分支(`rx1`&`rx2`, 也叫做async表达式 )会开始**并发**的执行。当任何一个表达式完成时，会将结果跟模式(`val1`&`val2`)进行匹配。若匹配成功，则剩下的表达式会被丢弃。
> `async`表达式都会返回一个`Future`，`Future`是惰性的，直到被`poll`时才会执行。因此，丢弃掉一个分支的`async`表达式，即意味着释放掉一个`Future`，那么这个异步操作无法再执行，因为所有associated state已经被drop。

### Future implemention
`MySelect`中包含了两个`Future`，当它被poll的时候，第一个分支`Pin::new(&mut self.async_operation_1).poll(cx)`会先执行，如果执行完成，那么`MySelct`会随之结束，另一个对应的`Future`会被释放掉。
```rust
struct MySelect {
	async_operation_1 : /*type*/
	async_operation_2 : /*type*/
}

impl Future for MySelect {
	type Output = ();
	
	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
		if let Poll::Ready(val) = Pin::new(&mut self.async_operation_1).poll(cx) {
			//do somthing
			return Poll::Ready(());
		} 
		if let Poll::Ready(val) = Pin::new(&mut self.async_operation_2).poll(cx) {
			//do somthing
			return Poll::Ready(());
		} 
		Poll::Pending
	}
}
```

> 这里没有在返回`Pending`时使用`Waker`来唤醒的原因是，参数`cx`被传入了内层`Future`(`async_operation_1`和`async_operation_2`的`Future`)的`poll(cx)`调用。只要内层的`Future`满足了`Waker`, `MySelect`也会自动满足 `Waker`。

#### tokio::select! 的返回值
`let out = tokio::select!` 我们可以接住`select！`的返回值，但这里`select!` 的所有分支必须返回一样的类型，否则编译器会报错。
```rust
#[tokio::main]
async fn main() {
    let out = tokio::select! {
        res1 = computation1() => res1,
        res2 = computation2() => res2,
    };
```

### Error传播
在`async`表达式中使用`?`向上返回错误，`async block`必须是返回`Result`类型:
`let (socket, _) = listener.accept().await?;`使用`?`因此它将`async`表达式的返回类型变成了`Result`类型。但是这里的闭包并没有具体的返回值，所以我们要使用`Ok::<_, io::Error>(())`来辅助 Rust 编译器的类型推断，推断闭包的返回类型为 `Result<(), io::Error>`，即一个空的 `Result`。
> `Ok::<_, io::Error>(())` 的作用是为异步任务提供一个明确的返回类型，指示任务成功完成且没有返回值。

```rust
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use std::io;
 
fn process(_socket: tokio::net::TcpStream) {
    println!("processing socket");
}
  
#[tokio::main]
async fn main() -> io::Result<()> {
    // [setup `rx` oneshot channel]
    let (tx, rx): (oneshot::Sender<()>, oneshot::Receiver<()>) = oneshot::channel();
    let listener = TcpListener::bind("localhost:3465").await?;
    
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
```
`res`被赋予`async`表达式的返回类型，On an error, `res` will be set to `Err(_)`。`res?`直接将错误传播propagate到`main()`, 让`main()`返回 `Result`。

### 借用规则
`spawning tasks` vs `select!`
- `spawning tasks` 必须要取得所有数据的所有权。
- `select!`没有该限制，而是可以借用数据，然后并发操作。借用规则是，多个分支的async表达式可以immutable borrow同一数据，或者一个分支表达式可以一次mutable borrow某个数据。但在{handler}中则不需要考虑借用规则，因为最终只有一个handler会执行。

`tokio::spawn` 函数会启动新的任务来运行一个异步操作。每个任务都是一个独立的对象可以单独被 Tokio 调度运行，因此两个不同的任务的调度都是独立进行的。=> `并行`
`select!` 宏就不一样了，它在同一个任务中**并发**运行所有的分支。正是因为这样，在同一个任务中，这些分支无法被同时运行。 => `并发`

### Loop & `Select!`
loop中嵌套select会让模式匹配持续进行。当一个分支执行完毕后，`select!`会继续循环等待并执行其他分支，直到所有分支都没有数据，通过break跳出循环。
`select!` 中哪个分支先被执行是无法确定的，若 `rx1` 中总是有数据，那每次循环都只会去处理第一个分支，后面两个分支永远不会被执行。

```rust
loop {
	let msg = tokio::select! {
		Some(msg) = rx1.recv() => msg,
		Some(msg) = rx2.recv() => msg,
		Some(msg) = rx3.recv() => msg,
		else => { break }
	};
```

### `&mut Future` 继续一个异步操作
- `&mut operation` 每一次循环调用就变成了对同一次 `action()` 的调用。
- 不加`&mut`，那么 operation表示每一次循环调用都是一次全新的`async_op()`调用。

如果要在一个引用上使用 `.await`，那么引用的值就必须是不能移动的或者实现了 `Unpin`。
```rust
async fn async_op(){...}

#[tokio::main()] 
async fn main() {
	let (tx, rx) = tokio::sync::mpsc::channel(128);
	let operation = async_op(); // lazy, generate a future
	tokio::pin!(operation);
	loop {
		tokio::select! {
			_ = &mut operation = break,
			Some(v) = rx.recv() => {...}
		}
	}
}
```

### future.set修改分支
`operation.set(action(Some(v)))`
[rust course: 修改一个分支](https://course.rs/advance-practice/select.html#%E4%BF%AE%E6%94%B9%E4%B8%80%E4%B8%AA%E5%88%86%E6%94%AF)



