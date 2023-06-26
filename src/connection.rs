use bytes::{
    Buf,
    BytesMut,
};
use mini_redis::Frame; //frame.rs
use mini_redis::Result;
use std::io::Cursor;
use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
};
use mini_redis::frame::Error::Incomplete;

pub struct Connection {
    stream: TcpStream,
    // 底层调用的Tcpstream::read方法的读取stream的行为是不确定的
    //所以我们要为Connection增加一个read buffer: socket->buffer->parse to freme -> remove the data
    // from buffer 这里使用 BytesMut 作为缓冲区类型，它是 Bytes 的可变版本。
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            //Allocate the buffer with 4kb of capacity
            buffer: BytesMut::with_capacity(4096),
        }
    }

    //read_frame 内部使用循环的方式读取数据，直到一个完整的帧被读取到时，才会返回。
    //当远程的对端关闭了连接后，也会返回。
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            //tokio::tcpstream.read_buf(): Pulls some bytes from this source into the specified
            // buffer, advancing the buffer's internal cursor.
            // 将stream中的bytes推进缓冲区，推进缓冲区的内部cursor， return 读入的字节数。
            // A nonzero n value indicates that the buffer buf has been filled in with n bytes of
            // data from this source. If n is 0, then it can indicate one of two scenarios:

            // 1. This reader has reached its "end of file" and will likely no longer be able to
            //    produce bytes. Note that this does not mean that the reader will always no longer
            //    be able to produce bytes.
            // 2. The buffer specified had a remaining capacity of zero.
            //在这里是指当读取成功时，read_buf会返回读取成功的字节数；如果是0代表到了stream的末尾
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                //如果代码执行到这里，说明remote close了connection.
                //下面的代码是为了检查是否是一个clean shutdown:
                // 1. if yes, buffer中应该没有readbuffer中应该没有任何数据
                // 2. 如果readbuffer中有数据，代表对端peer在send frame的过程中close了socket
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection was reset by peer".into());
                }
            }
        }
    }

    //Parsing is done in two steps:
    // 1. Ensure a full frame is buffered and find the end index of the frame.
    // 2. Parse the frame.
    //non-async
    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        //Cursor::new(&self.buffer[..]) 创建了一个新的 Cursor 对象，该对象可以对 self.buffer
        // 中的数据进行读写操作，并且初始位置位于数据的开头。
        let mut buf = Cursor::new(&self.buffer[..]);
        //Frame::check 使用了 Buf 的字节迭代风格的
        // API。例如，为了解析一个帧，首先需要检查它的第一个字节，该字节用于说明帧的类型。
        // 这种首字节检查是通过 Buf::get_u8
        // 函数完成的，该函数会获取游标所在位置的字节，然后将游标位置向右移动一个字节。
        match Frame::check(&mut buf) {
            Ok(_) => {
                //check()会将cursor从0移到frame末尾，当前cursor的位置就是frame的字节数
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = Frame::parse(&mut buf)?;
                //解析完成后，将缓冲区中的该frame移出
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            // 缓冲区的数据不足以解析出一个完整的帧
            Err(Incomplete) => Ok(None),
            // 遇到一个错误
            Err(e) => Err(e.into()),
        }
    }
}
