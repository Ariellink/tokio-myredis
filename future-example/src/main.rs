use std::time::{Instant, Duration};
use future_example::delay::Delay;

#[tokio::main]
async fn main() {
    print!("Instant::now is {:?}", Instant::now());
    let when = Instant::now() + Duration::from_millis(10);
    print!("when is {:?}", when);
    let future = Delay {when};

    let out = future.await;

    println!("{}",out);
}
