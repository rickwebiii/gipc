use protobuf::{Message};
use futures::{Future};

#[derive(Hash)]
pub enum MethodKind {
  Unary,
  ClientStreaming,
  ServerStreaming,
  Duplex
}

impl PartialEq for MethodKind {
  fn eq(&self, other: &MethodKind) -> bool {
      self == other
  }
}

impl Eq for MethodKind { }

#[derive(Hash)]
pub struct ApiMethod {
  name: String,
  kind: MethodKind,
}

impl PartialEq for ApiMethod {
  fn eq(&self, other: &ApiMethod) -> bool {
    self.name == other.name &&
      self.kind == other.kind
  }
}

impl Eq for ApiMethod { }
