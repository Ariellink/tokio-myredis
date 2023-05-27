# async and await

`cargo install mini-redis`  
`mini-redis-server` //在本地运行mini-redis-server  

Instead of 直接 `mini-redis-client set foo 333`and `mini-redis-client get foo`,下面我们直接调用mini-redis的接口（编辑cargo.toml）去写一个client连接mini-redis-server的rust代码：

看下mini-redis监听的端口：  
```shell
➜  my-redis git:(master) ✗ netstat -nlp | grep mini
(Not all processes could be identified, non-owned process info
 will not be shown, you would have to be root to see it all.)
tcp        0      0 127.0.0.1:6379          0.0.0.0:*               LISTEN      151811/mini-redis-s 
```
client.rs
```rust
use mini_redis::{client,Result};

#[tokio::main()]
async fn main() -> Result<()> {
    //使用mini-redis包提供的connect函数，与指定ip建立长连接，一旦连接成功。client初始化完成
    let mut client = client::connect("127.0.0.1:6379").await?;
    
    client.set("hello", "world".into()).await?;
    let result = client.get("hello").await?;
    
    println!("从服务器获取到结果 = {:?}", result);
    
    Ok(())
}
```

mini_redis中的connect()函数实现如下：  
```rust
use mini_redis::Result;
use mini_redis::client::Client;
use tokio::net::ToSocketAddrs;

pub async fn connect<T: ToSocketAdrs>(addr:T) -> Result<Client> {...}
```
1. `asycn fn` 函数不会立即返回值，而是返回一个`Future`.  
2. `Future` 会在未来某个点被执行，然后最终获取到真正的返回值 `Result<Client>`  
3. `Future` 需要配合 `await` 才能运行起来。
4. `await` 只能在 async 函数中使用,执行`await`他会挂起当前异步函数，并将控制权交还给调度器。在 .await 执行期间，任务可能会在线程间转移。因当使用多线程 Future 执行器( executor )时， Future 可能会在线程间被移动，因此 async 语句块中的变量必须要能在线程间传递。此没有实现send的类型要在await之前提前释放掉。


### 什么是异步编程？  
同步代码执行的顺序是一行一行按顺序执行，遇到某一行执行无法立即完成，整个函数就会进入阻塞状态，直到该操作完成。  
异步编程中，无法立即完成的操作A会被切到后台执行/等待，立即返回一个future。当前线程会继续往下执行下一行。一旦A执行完毕，他会通知执行器，然后执行器会调度它并从上次离开的点继续执行。？？？   

#### 异步执行顺序  

```rust
async fn say_to_world() -> String {
    String::from("lazy")
}

//#[tokio::main()]宏将aync fn main隐式转换陈fn main,同时初始化了整个异步运行时
#[tokio::main()]
async fn main() {
    let op = say_to_world(); //（1）say_to_world()未执行，op的类型是impl Future<Output=String>
    println!("hello");
    let a = op.await; //（2）say_to_world()执行返回类型为String的a
    print!("{}",a);
}
```
输出如下：  
```shell
hello
lazy
```
`asyn fn`的返回类型： 返回实现了 Future 特征的匿名类型: impl Future<Output = String>。

### #[tokio::main()]  
添加了`#[tokio::main()]`宏的`asyn fn main`函数隐形式转化成了：
```rust
fn main() {
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        println!("hello");
    })
}
```
## Tokio Tasks

A tokio task is an 异步绿色线程。通过传递aync block给tokio::spawn函数return joinHandle来创建。  
创建一个任务仅仅需要一次 64 字节大小的内存分配。
```rust
#[tokio::main()]
async fn main() {
    //任务（task）并不直接对应于操作系统的线程,通过事件驱动模型在单个线程上调度和执行的
    let handle: JoinHandle<&str> = tokio::spawn(
        async {
            "return value"
        }
    );
    
    //do some other work

    let out: &str = handle.await.unwarp();
    println!("Got: {}", out);
}
```

## await
1. await有点类似于递归出口，函数A如果是异步函数，使用await()才会开始运行，
2. 函数A中的代码块执行到await()时，函数A挂起。main继续往下执行。等到A中的await执行完了，


```rust

use std::thread;

use futures::executor::block_on;
use tokio::time::sleep;

struct Song {
    author: String,
    name: String,
}

async fn learn_song() -> Song {
    println!("song learning");
    sleep(std::time::Duration::from_secs(10)).await; //learning song挂起1
    //thread::sleep(std::time::Duration::from_secs(10)); //线程阻塞10s，learn_song()无法挂起
    Song {
        author: "曲婉婷".to_string(),
        name: String::from("《我的歌声里》"),
    }
}

async fn sing_song(song: Song) {
    println!(
        "给大家献上一首{}的{} ~ {}",
        song.author, song.name, "你存在我深深的脑海里~ ~"
    );
}

async fn dance() {
    println!("唱到情深处，身体不由自主的动了起来~ ~");
}

async fn learn_and_sing() {
    // 这里使用`.await`来等待学歌的完成，但是并不会阻塞当前线程，该线程在学歌的任务`.await`后，完全可以去执行跳舞的任务
    let song = learn_song().await; //learn_and_sing()挂起2

    // 唱歌必须要在学歌之后
    sing_song(song).await;
}

async fn async_main() {
    let f1 = learn_and_sing();
    let f2 = dance();

    // `join!`可以并发的处理和等待多个`Future`，若`learn_and_sing Future`被阻塞，那`dance Future`可以拿过线程的所有权继续执行。若`dance`也变成阻塞状态，那`learn_and_sing`又可以再次拿回线程所有权，继续执行。
    // 若两个都被阻塞，那么`async main`会变成阻塞状态，然后让出线程所有权，并将其交给`main`函数中的`block_on`执行器
    futures::join!(f1, f2);
}

#[tokio::main()]
async 
fn main() {
    block_on(async_main());
}

```
