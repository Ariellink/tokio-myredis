use std::time::Duration;

use bytes::Bytes;
use mini_redis::client;
use tokio::{
    net::ToSocketAddrs,
    runtime::Runtime,
};
//use mini_redis::Result;

/// For performance reasons, boxing is avoided in any hot path. For example, in
/// `parse`, a custom error `enum` is defined. This is because the error is hit
/// and handled during normal execution when a partial frame is received on a
/// socket. `std::error::Error` is implemented for `parse::Error` which allows it to be converted
/// `Box<dyn std::error::Error>`.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for mini-redis operations.
///
/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;

pub struct BlockingClient {
    //
    inner: client::Client,
    //包含了tokio runtime的实例
    runtime: Runtime,
}

impl BlockingClient {
    //构造函数
    //建立一个到redis server的连接
    pub fn connect<T: ToSocketAddrs>(addr: T) -> Result<BlockingClient> {
        // 创建一个单线程的tokio runtime
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        // 在tokio runtime上调用block_on方法阻塞当前线程直到future执行完毕
        // 使用这个runtime来调用异步的connect连接方法
        let inner = runtime.block_on(client::connect(addr))?;

        Ok(BlockingClient { inner, runtime })
    }

    //同步接口：通过 block_on 将异步形式的 Client 的方法变成同步调用的形式。
    pub fn get(&mut self, key: &str) -> Result<Option<Bytes>> {
        self.runtime.block_on(self.inner.get(key))
    }

    pub fn set(&mut self, key: &str, value: Bytes) -> Result<()> {
        self.runtime.block_on(self.inner.set(key, value))
    }

    //mini-redis的set_expires方法:设置一个带有过期时间的key-value,
    // 过期时间是一个Duration类型的参数。过期的是key,而不是value,如果key不存在,则会自动创建,
    // 如果key已经存在,则会覆盖原来的value,但是过期时间不会覆盖,如果想要覆盖过期时间,
    // 则需要先del删除key,然后再set_expires设置
    pub fn set_expires(&mut self, key: &str, value: Bytes, expiration: Duration) -> Result<()> {
        self.runtime
            .block_on(self.inner.set_expires(key, value, expiration))
    }

    //mini-redis的publish方法:发布一个消息到指定的channel,返回值是一个u64类型的整数,
    // 表示有多少个客户端订阅了这个channel。 客户端为什么要订阅消息：
    // 因为客户端可能在执行某个操作的时候,需要等待某个资源准备好,
    // 而这个资源的准备可能是一个异步的过程,所以客户端需要订阅这个channel,一旦资源准备好,
    // 就会发布一个消息到这个channel,客户端就会收到这个消息,然后继续执行后续的操作。
    // channel为什么是字符串：因为channel是一个字符串,所以可以使用通配符来订阅多个channel,
    // 比如订阅所有以foo开头的channel,则可以订阅foo*这个channel,订阅所有以foo结尾的channel,
    // 则可以订阅*foo这个channel。 发布到channel的逻辑:首先需要获取到channel的订阅者列表,
    // 然后遍历这个列表,将消息发送给每一个订阅者。 channel的订阅者列表:
    // channel的订阅者列表是一个HashMap,其中key是channel的名字,而value是一个HashSet,
    // 里面存放了所有订阅了这个channel的客户端的id。
    pub fn publish(&mut self, channel: &str, message: Bytes) -> Result<u64> {
        self.runtime.block_on(self.inner.publish(channel, message))
    }

    pub fn subscribe() -> Result<()> {
        Ok(())
    }
}
