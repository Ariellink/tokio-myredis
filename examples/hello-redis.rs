//示例代码1：
use mini_redis::{
    client,
    Result,
};

#[tokio::main()]
async fn main() -> Result<()> {
    //使用mini-redis包提供的connect函数，与指定ip建立长连接，一旦连接成功。client初始化完成
    let mut client = client::connect("127.0.0.1:6379").await?;

    client.set("hello", "world".into()).await?;
    let result = client.get("hello").await?;

    println!("从服务器获取到结果 = {:?}", result);

    Ok(())
}

//示例代码2：
// async fn say_to_world() -> String {
//     String::from("lazy")
// }

// #[tokio::main()]
// async fn main() {
//     let op = say_to_world();
//     println!("hello");
//     let a = op.await;
//     print!("{}",a);
// }

//示例代码3：
// fn main() {
//     let mut runtime = tokio::runtime::Runtime::new().unwrap();
//     rt.block_on(async {
//         println!("hello");
//     })
// }
