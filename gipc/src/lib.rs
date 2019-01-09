pub mod ipc;

pub use self::ipc::{
  CallOption,
  Client,
  ClientUnaryReceiver,
  Channel,
  Error,
  Marshaller,
  Method,
  MethodType,
  pb_de,
  pb_ser,
  Result,
  RpcContext,
  Service,
  UnarySink
};