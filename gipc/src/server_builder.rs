use futures::{Future};
use futures::executor::{ThreadPoolBuilder};
use futures::task::{SpawnExt};
use ipc::{MessageIpcConnection, MessageIpcServer};
use log::{error};
use num_cpus;
use protobuf::{Message};

use std::collections::{HashMap};
use std::sync::{Arc};
use std::thread;

use super::api_method::{
  ApiMethod
};

type MessageFuture = Box<Future<Output=Message>>;

pub struct ServerBuilder {
  unary_methods: HashMap<String, Box<(FnMut()-> MessageFuture) + Sync + Send>>,
  ipc_id: Option<String>,
  thread_pool_size: usize,
}

pub enum MethodResult<T: Message> {
  Ok(T),
  Aborted(String)
}

pub enum BuildServerError {
  NoIpcIdSpecified,
}

impl ServerBuilder {
  pub fn new() -> ServerBuilder {
    ServerBuilder {
      unary_methods: HashMap::new(),
      ipc_id: None,
      thread_pool_size: num_cpus::get(),
    }
  }

  pub fn add_unary_method<Req: Message, Res: Message, F>(
    mut self,
    method: &str,
    mut callback: F
  ) -> Self
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

  pub fn thread_pool_size(mut self, n: usize) -> Self {
    self.thread_pool_size = n;

    self
  }

  pub fn bind(mut self, ipc_id: &str) -> Self {
    self.ipc_id = Some(ipc_id.to_owned());
    self
  }

  pub fn build(self) -> Result<Server, BuildServerError> {
    let ipc_id = match self.ipc_id {
      Some(id) => id,
      None => { return Err(BuildServerError::NoIpcIdSpecified); }
    };

    Ok(Server {
      unary_methods: Arc::new(self.unary_methods),
      ipc_id: ipc_id,
      thread_pool_size: self.thread_pool_size,
    })
  }
}

pub struct Server {
  unary_methods: Arc<HashMap<String, Box<(FnMut() -> MessageFuture) + Sync + Send>>>,
  ipc_id: String,
  thread_pool_size: usize,
}

impl Server {
  pub fn start(&self) {
    let unary_methods = self.unary_methods.clone();
    let ipc_id = self.ipc_id.to_owned();
    let thread_pool_size = self.thread_pool_size;

    thread::spawn(move || {
      let server = MessageIpcServer::new(&ipc_id);
      let mut thread_pool = ThreadPoolBuilder::new()
        .pool_size(thread_pool_size)
        .name_prefix("gIPC Thread pool")
        .create()
        .unwrap();

      let spawner = thread_pool.clone();

      thread_pool.run(async {
        let result = await!(Server::run_internal(
          spawner,
          ipc_id,
          unary_methods,
        ));

        match result {
          Err(err) => { error!("Failed to run gIPC server {}", err); },
          _ => { }
        };
      });
    });
  }

  
  async fn run_internal<S>(
    mut spawner: S,
    ipc_id: String,
    unary_methods: Arc<HashMap<String, Box<(FnMut() -> MessageFuture) + Sync + Send + 'static>>>
  ) -> std::io::Result<()> where S: SpawnExt {
    let mut server = MessageIpcServer::new(&ipc_id)?;

    loop {
      let (connection, new_server) = await!(server.wait_for_connection())?;

      async fn handle_connection(connection: MessageIpcConnection) -> std::io::Result<()> {
        let message_bytes = await!(connection.read())?;

        Ok(())
      }

      spawner.spawn(async move {
        let result = await!(handle_connection(connection));

        if let Err(err) = result {
          error!("Got error {}", err);
        }
      }).unwrap();

      server = new_server;
    }

    Ok(())
  }
}