use bytes::Bytes;
use mini_redis::{
    Connection,
    Frame,
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        Mutex,
    },
};
use tokio::net::{
    TcpListener,
    TcpStream,
};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main()]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("listening");

    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        //accept是异步函数返回impl Future = Result<(TcpStream,SocketAddr),Error>
        let (stream, _) = listener.accept().await.unwrap();
        let db = db.clone();
        println!("Accepted");
        //引入多线程
        tokio::spawn(async move {
            process(stream, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{
        self,
        Get,
        Set,
    };

    //mini-redis对读写做了封装，已经将字节流转换为data frame(数据帧 = redis命令 + 数据)
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        println!("Got: {:?}", frame);
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();

                //let b:&Bytes = cmd.value();
                //不加clone的话，value返回的是&Bytes引用
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        //send error to client (Error: "unimplemented")
        connection.write_frame(&response).await.unwrap();
    }
}
