use futures::{Future};
use ipc::{channel};
use protobuf::{Message};

use std::collections::{HashMap};
use std::sync::{Arc};
use std::thread;

use super::api_method::{
  ApiMethod
};

type MessageFuture = Box<Future<Output=Message>>;

pub struct ServerBuilder {
  unary_methods: HashMap<String, Box<FnMut() -> MessageFuture>>
}

pub enum MethodResult<T: Message> {
  Ok(T),
  Aborted(String)
}

impl ServerBuilder {
  fn new() -> ServerBuilder {
    ServerBuilder {
      unary_methods: HashMap::new()
    }
  }

  fn add_unary_method<Req: Message, Res: Message, F>(
    mut self,
    method: &str,
    mut callback: F
  ) -> ServerBuilder
    where F: FnMut() -> MessageFuture + Sync + Send + 'static
  {
    self.unary_methods.insert(
      method.to_owned(),
      Box::new(move || {
        callback()
      })
    );

    self
  }

  fn build(self) -> Server {
    Server {
      unary_methods: Arc::new(self.unary_methods)
    }
  }
}

pub struct Server {
  unary_methods: Arc<HashMap<String, Box<FnMut() -> MessageFuture>>>
}

impl Server {
  start(&self) {
    let unary_methods = self.unary_methods.clone();

    thread::spawn(move || {
      
    });
  }
}