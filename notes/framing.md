
`pub async fn read_frame(&self) -> Result<Option<Frame>> `   
<br>
read_frame 方法循环运行。 首先，调用 self.parse_frame()。 这将尝试从 self.buffer 中解析一个 redis 帧。 如果有足够的数据来解析一个帧，则该帧返回给 read_frame() 的调用者。否则，我们将尝试从套接字中读取更多数据到缓冲区中。 读取更多数据后，再次调用 parse_frame()。 这一次，如果接收到足够的数据，解析可能会成功。

从流中读取时，返回值 0 表示不再从对等方接收数据。 如果读取缓冲区中仍有数据，则表明已收到部分帧并且连接突然终止。 这是一个错误条件并返回 Err。

`self.stream.read_buf()`来读取socket中的tcpstream,该方法的参数是ByteMut。  

### 理解如何读取数据？
