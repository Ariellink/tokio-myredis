use future_example::delay::Delay;
use std::time::{
    Duration,
    Instant,
};

#[tokio::main]
async fn main() {
    //changhushallhdsdufcdh hsiauhfdiudh suidaghashdjh ashdhasdiugeafgsduhashd a
    // shduiahfuhefgsufgsdufhdsf  sadfhuishfushfs fdasfhshfuisefyh fsdhfudshfusd fsudfsda
    // asdfhuidshfusdfhuadshfsaif fasdhuihfuidshfsd
    print!("Instant::now is {:?}", Instant::now());
    let when = Instant::now() + Duration::from_millis(10);
    print!("when is {:?}", when);
    let future = Delay { when };

    let out = future.await;

    println!("{}", out);
}
