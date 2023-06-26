use chrono::Utc;
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

struct MyFuture {
    expiration_time: Instant,
}

impl Future for MyFuture {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.expiration_time {
            println!("Hello, it is the time for furture 1");
            Poll::Ready(String::from("future 1 has completed."))
        } else {
            println!("Hello, it is not yet the time for furture 1. Going to sleep.");
            let waker = cx.waker().clone();
            let expiration_time = self.expiration_time;
            //spawn a non-blocking thread
            thread::spawn(move || {
                let cur_time = Instant::now();
                if cur_time < expiration_time {
                    thread::sleep(expiration_time - cur_time);
                }
                waker.wake();
            });
            Poll::Pending
        }
    }
}

async fn read_file2() -> String {
    println!("file2 pre-work...{:?}", Utc::now());
    thread::sleep(Duration::new(2, 0));
    println!("Processing file 2 {:?}", Utc::now());
    String::from("Hello, there from file 2.")
}

#[tokio::main()]
async fn main() {
    let h1 = tokio::spawn(async {
        let future1 = MyFuture {
            expiration_time: Instant::now() + Duration::from_millis(4000),
        };
        println!("{:?}", future1.await);
    });

    let h2 = tokio::spawn(async {
        let future2 = read_file2().await;
        println!("{:?}", future2);
    });
    let _ = tokio::join!(h1, h2);
}
