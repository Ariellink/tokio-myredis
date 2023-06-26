use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
    thread,
    time::{
        Duration,
        Instant,
    },
};

pub struct Delay {
    pub when: Instant,
}

impl Future for Delay {
    type Output = &'static str;
    //Context: &waker的access
    //<'_ >代表一个匿名的生命周期注解，
    //<'_ 这意味着Context中的引用的有效期和调用poll函数时传入的引用的有效期是一致的。
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
                //一旦计时结束(该资源已经准备好)，资源会通过 waker.wake()
                // 调用通知执行器我们的任务再次被调度执行了
                waker.wake();
            });

            Poll::Pending
        }
    }
}

use std::sync::Arc;
use tokio::sync::Notify;

async fn delay(dur: Duration) {
    let when = Instant::now() + dur;
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();

    thread::spawn(move || {
        let now = Instant::now();

        if now < when {
            thread::sleep(when - now);
        }

        notify2.notify_one();
    });

    notify.notified().await;
}
