pub struct RpcContext {
  
}

pub struct Channel {

}

pub enum MethodType {
  Unary
}

pub struct CallOption {

}

pub struct Service {

}

pub struct Client {

}

pub struct Marshaller<T> {
  ser: fn(_: &T, _: &mut Vec<u8>) -> (),
  de: fn(_: &[u8]) -> Result<T>,
}

pub struct ServiceBuilder {

}

impl ServiceBuilder {
  pub fn add_unary_handler<Req, Res>(method: Method<Req, Res>) {

  }
}

pub struct Method<Req, Res> {
  ty: MethodType,
  name: &'static str,
  req_mar: Marshaller<Req>,
  resp_mar: Marshaller<Res>,
}

pub fn pb_ser<T>(_: &T, _: &mut Vec<u8>) { }
pub fn pb_de<T>(_: &[u8]) -> Result<T> { 
  Err(Error::Horse)
}

pub struct UnarySink<T> {
  dummy: std::marker::PhantomData<T>,
}

pub struct ClientUnaryReceiver<T> {
  dummy: std::marker::PhantomData<T>,
}

pub enum Error {
  Horse
}

pub type Result<T> = core::result::Result<T, Error>;