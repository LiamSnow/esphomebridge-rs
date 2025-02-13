
// trait ReadVarint32: Read {
//     fn read_varint32(&mut self) -> io::Result<u32>;
// }
//
// impl ReadVarint32 for TcpStream {
//     fn read_varint32(&mut self) -> io::Result<u32> {
//         let mut result: u32 = 0;
//         let mut shift: u32 = 0;
//
//         loop {
//             let mut buf = [0u8; 1];
//             self.read_exact(&mut buf)?;
//             let byte = buf[0];
//
//             result |= ((byte & 0x7f) as u32) << shift;
//             shift += 7;
//
//             if byte & 0x80 == 0 {
//                 break;
//             }
//
//             if shift >= 32 {
//                 return Err(io::Error::new(io::ErrorKind::InvalidData, "Varint32 too long"));
//             }
//         }
//
//         Ok(result)
//     }
// }

use std::{sync::Arc, task::{Context, Poll, Wake, Waker}};

use tokio::net::TcpStream;


struct NoopWaker;
impl Wake for NoopWaker {
    fn wake(self: Arc<Self>) { }
    fn wake_by_ref(self: &Arc<Self>) { }
}


pub trait BufferEmpty {
    async fn buffer_empty(&mut self) -> bool;
}

impl BufferEmpty for TcpStream {
    async fn buffer_empty(&mut self) -> bool {
        let waker = Waker::from(Arc::new(NoopWaker));
        let mut cx = Context::from_waker(&waker);
        return matches!(self.poll_read_ready(&mut cx), Poll::Ready(Ok(())))
    }
}
