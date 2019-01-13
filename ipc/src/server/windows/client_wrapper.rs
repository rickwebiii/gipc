
pub struct IpcConnection {
}




pub struct IpcClientWrapper {
  client: PipeClient,
  pipe_name: OsString,
}

impl IpcClientWrapper {
  pub fn new(name: &str) -> std::io::Result<IpcClientWrapper> {
    let pipe_name = OsString::from(r"\\.\pipe\".to_owned() + name);
    let mut client = PipeClient::connect(&pipe_name)?;
    client.set_read_timeout(Some(Duration::from_millis(20)));
    client.set_write_timeout(Some(Duration::from_millis(20)));

    Ok(IpcClientWrapper {
      client: client,
      pipe_name: pipe_name
    })
  }

  pub async fn write(mut self, data: &[u8]) -> std::io::Result<()> {
    let handle = self.client.write_async_owned(data.to_owned())?;

    let future = WriteFuture { handle: Some(handle) };

    await!(future)?;

    Ok(())
  }
}