use std::pin::Pin;
use std::task::{Context,Poll};
use std::future::Future;
use std::{thread};
use std::time::{Duration};
use chrono::{Utc};


#[tokio::main()]
async fn main() {
    
    // let t1 = tokio::spawn(async {
    //     let future1 = read_file1();
    //     future1.await;
    // });

    
    let t2 = tokio::spawn(async {
        let future2 = read_file2();
        future2.await;
    });

    let t3 = tokio::spawn(async {
        let future3 = ReadFileFuture {};
        future3.await
    });

    let _ = tokio::join!(t2,t3);
}

struct ReadFileFuture {}

impl Future for ReadFileFuture {
    type Output = String;
    // 由于future会被异步运行时反复的poll,所以把future固定Pin到内存中的特定位置不允许移动，必要安全性
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>{
        println!("tokio, Stop polling me!");
        _cx.waker().wake_by_ref(); //告诉执行器再次poll，poll函数继续从头被调用
        Poll::Pending
    }
}

async fn read_file2() -> String {
    println!("file2 pre-work...{:?}", Utc::now());
    thread::sleep(Duration::new(2, 0));
    println!( "Processing file 2 {:?}", Utc::now());   
    String::from("Hello, there from file 2.")
}

async fn read_file1() -> String {
    println!("file1 pre-work... {:?}", Utc::now());
    thread::sleep(Duration::new(4, 0));
    println!("Processing file 1 {:?}", Utc::now());
    String::from("Hello, there from file 1.")
}

/*
thread0:

t1 file 1 pre-working
t1 blocking -----> t2 file 2 pre-working                             
    |2m            t2 blocking
t1 next                 | 4m
t1 done --------------->|
                    t2 next
                    t2 done
 */