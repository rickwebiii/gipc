struct WriteFuture<T: PipeIo> {
  handle: Option<WriteHandle<'static, T>>,
}

impl <T: PipeIo + Unpin> Future for WriteFuture<T> {
  type Output = std::io::Result<()>;

  fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
    let future: WriteFuture<T> = self.get_mut();
    
    let io_result = future.handle.take().wait();

    match io_result {
      Ok(x) => Poll::Ready(Ok(())),
      Err(err) => {
        match err.kind() {
          std::io::ErrorKind::TimedOut => {
            future.handle = Some()
            lw.wake();
            Poll::Pending
          }
          _ => { Poll::Ready(Err(err)) }
        }
      }
    }
  }
}


struct PipeConnectionFuture {
  pipe: HANDLE,
}

impl PipeConnectionFuture {
  pub fn new(pipe: HANDLE) -> PipeConnectionFuture {
    PipeConnectionFuture {
      pipe: pipe
    }
  }
}

impl Future for PipeConnectionFuture {
  type Output = std::io::Result<PipeServer>;

  fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
    let server = self.get_mut();


    match io_result {
      Ok(connection_result) => {
        match connection_result {
          Ok(pipe_server) => Poll::Ready(Ok(pipe_server)),
          Err(connection_server) => {
            server.connecting_server = Some(connection_server);
            lw.wake();
            Poll::Pending
          }
        }
      },
      Err(err) => {      
        Poll::Ready(Err(err))
      }
    }
  }
}