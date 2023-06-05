# 

# Pin

# Furture

Future trait  
```rust
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Future {
    type Output; //Future 执行完成后才返回的类型
    //somehow, future.await() => future运行并等待Future的完成
    fn poll(self: Pin<&mut Self>, cx: &mut Context)
        -> Poll<Self::Output>;
}
```

为了运行一个异步函数，我们必须使用`tokio::spawn` 或通过 `#[tokio::main]` 标注的 `async fn main` 函数。它们有一个非常重要的作用：将最外层 `Future` 提交给 `Tokio` 的执行器。该执行器负责调用 `poll` 函数，然后推动 `Future` 的执行，最终直至完成。

若 Future 无法被完成，例如它所等待的资源还没有准备好，此时就会返回 Poll::Pending，该返回值会通知调用者： Future 会在稍后才能完成。  

当一个 Future 由其它 Future 组成时，调用外层 Future 的 poll 函数会同时调用一次内部 Future 的 poll 函数。

> 我们的 mini-tokio 只应该在 `Future` 准备好可以进一步运行后，才去 `poll` 它，例如该 `Future` 之前阻塞等待的资源已经准备好并可以被使用了，就可以对其进行 `poll`。再比如，如果一个 `Future` 任务在阻塞等待从 `TCP socket` 中读取数据，那我们只想在 `socket` 中有数据可以读取后才去 `poll` 它，而不是没事就 `poll` 着玩。 为了实现这个功能，我们需要 `通知 -> 运行` 机制：当任务可以进一步被推进运行时，它会主动通知执行器，然后执行器再来 `poll`。  

一切的答案都在 Waker 中，资源可以用它来通知正在等待的任务：该资源已经准备好，可以继续运行了。

### Waker
```rust
core::task::wake
pub struct Context<'a> 
```
`Context`: The context of an asynchronous task.  异步任务的上下文。
Currently, Context only serves to provide access to a &Waker which can be used to wake the current task.
<br>

`Context` 参数中包含有 `waker()`方法。该方法返回一个绑定到当前任务上的 `Waker`，然后 `Waker` 上定义了一个 `wake()` 方法，用于通知执行器相关的任务可以继续执行。   

准确来说，当 `Future` 阻塞等待的资源已经准备好时(例如 `socket` 中有了可读取的数据)，该资源可以调用 `wake()` 方法，来通知执行器 这个`Future` 的 `poll` 方法可以取得进展。

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context)
        -> Poll<Self::Output>;
```

在`poll()`中加入唤醒条件线程：   

> 千万不能忘记在返回 Poll::Pending 时调用 wake! 否则会导致任务永远被挂起，再也不会被执行器 poll。
```rust
impl Future for Delay {
    type Output = &'static str;
    //Context: &waker的access
    //<'_ >代表一个匿名的生命周期注解，这意味着Context中的引用的有效期和调用poll函数时传入的引用的有效期是一致的。
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            print!("Hello World");
            Poll::Ready("done")
        } else {
            //为当前任务克隆一个waker的句柄
            let waker = cx.waker().clone(); 
            let when = self.when;
            thread::spawn(move || {
                let now = Instant::now();
                //计时器用来模拟一个阻塞等待的资源
                if now < when {
                    thread::sleep(when - now);
                }
                //一旦计时结束(该资源已经准备好)，资源会通过 waker.wake() 调用通知执行器我们的任务要被再次调度
                waker.wake();
            });
            
            Poll::Pending
        }
    }
}
```

通知的控制权掌握在程序员手里：  
之前这段代码，我们没有加上任何条件就直接去唤醒。直接调用了 `wake_by_ref()` 进行通知, 这样做会导致future立刻被再次调度执行。


```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<&'static str>
    {
        if Instant::now() >= self.when {
            // 时间到了，Future 可以结束
            println!("Hello world");
            // Future 执行结束并返回 "done" 字符串
            Poll::Ready("done")
        } else {
            // 目前先忽略下面这行代码
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
```
 Waker 是 Rust 异步编程的基石，因此绝大多数时候，我们并不需要直接去使用它。例如，在 Delay 的例子中， 可以使用 tokio::sync::Notify 去实现。


 A Notify can be thought of as a [Semaphore] starting with 0 permits. The [notified().await] method waits for a permit to become available, and [notify_one()] sets a permit if there currently are no available permits.

The synchronization details of Notify are similar to thread::park and Thread::unpark from std. A Notify value contains a single permit. [notified().await] waits for the permit to be made available, consumes the permit, and resumes. [notify_one()] sets the permit, waking a pending task if there is one.

If notify_one() is called before notified().await, then the next call to notified().await will complete immediately, consuming the permit. Any subsequent calls to notified().await will wait for a new permit.

If notify_one() is called multiple times before notified().await, only a single permit is stored. The next call to notified().await will complete immediately, but the one after will wait for a new permit.