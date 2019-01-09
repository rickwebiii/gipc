//use futures::io::{AsyncRead, AsyncWrite};

pub struct LocalClient {
  
}

pub struct Channel {

}
/*
impl AsyncRead for Channel {

}

impl AsyncWrite for Channel {

}*/

pub trait Client {
  fn connect(name: &str) -> Channel;

}