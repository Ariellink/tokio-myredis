# Stream 异步迭代器


- [Stream 异步迭代器](#stream-异步迭代器)
    - [异步迭代器 VS 同步迭代器](#异步迭代器-vs-同步迭代器)
  - [Stream trait](#stream-trait)
      - [`into_stream()`方法](#into_stream方法)
    - [适配器 Adaptor](#适配器-adaptor)


### 异步迭代器 VS 同步迭代器

在rust中，迭代器是同步的。如使用for循环或者迭代器方法map/filter/fold来处理一个集合时，每个元素都会阻塞直到前一个元素处理完成。
tokio和async-std提供了异步迭代器的支持。异步迭代器的特点是，返回的元素可以是`Future`对象或者`aync fn`异步函数。

## Stream trait
Tokio提供了 `stream trait`，可以在异步函数中对其进行迭代，同时[`StreamExt`](https://docs.rs/tokio-stream/0.1.8/tokio_stream/trait.StreamExt.html) 特征上定义了与迭代器类似的常用适配器（迭代器方法）。

stream实现了`StreamExt trait`的next方法，next方法返回`Option<T>`， 其中 `T` 是从 `stream` 中获取的值的类型。Rust 语言还不支持异步的 `for` 循环，因此我们需要 `while let` 循环和 [`StreamExt::next()`](https://docs.rs/tokio-stream/0.1.8/tokio_stream/trait.StreamExt.html#method.next) 一起使用来实现迭代的目的:
```rust
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let mut stream = tokio_stream::iter(&[1, 2, 3]);

    while let Some(v) = stream.next().await {
        println!("GOT = {:?}", v);
    }
}

```

#### `into_stream()`方法
将某个对象转换为`Stream`返回
要对`Stream`调用`next()`方法，`stream`必须被pin在栈上，使用`tokio::pin!`

```rust
async fn subscribe() -> mini_redis::Result<()> {
    let client = client::connect("127.0.0.1:6379").await?;
    let subscriber: Subscriber = client.subscribe(vec!["numbers".to_string()]).await?;
    //into_stream() 消费了Subscriber, 将Subscriber转换为Stream并返回
    //这个stream对象会按照消息到达的顺序逐个yield消息
    let messages: impl Stream<Item = Result<Message, Box<dyn Error + Send + Sync>>> = subscriber.into_stream();
    tokio::pin!(messages);
    while let Some(msg) = messages.next().await {
       println!("got = {:?}", msg);
    }
    Ok(())
}
```

### 适配器 Adaptor
我们也可以用适配器来编辑`Stream`：
```rust
let messages = subscriber
    .into_stream()
    .filter(|msg| match msg {
	    //如果消息`msg`是`Ok`类型且其内容长度为1，`match`表达式为`true`，表示满足过滤条件。表示该消息满足过滤条件，将被保留在流中。
		//如果消息`msg`不满足上述条件，或者是`Err`类型的错误消息，`match`表达式为`false`，表示不满足过滤条件，将被过滤掉，不会出现在流中。
        Ok(msg) if msg.content.len() == 1 => true,
        _ => false,
    })
    .take(3);
```