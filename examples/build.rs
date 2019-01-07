use dirs::{home_dir};

use std::path::PathBuf;
use std::process::{Command};

fn main() {
  let cargo_dir = home_dir()
    .expect("Failed to get user directory")
    .join(".cargo");

  let grpc_rust_plugin_path = if cfg!(windows) {
    cargo_dir.join("grpc_rust_plugin.exe")
  } else {
    cargo_dir.join("grpc_rust_plugin")
  };

  let hello_proto_path = PathBuf::from("proto")
    .join("hello.proto");

  let protoc_result = Command::new("protoc")
    .arg("--rust_out=.")
    .arg("--grpc_out=.")
    .arg(format!("--plugin=protoc-gen-grpc={:?}", grpc_rust_plugin_path))
    .arg(format!("{:?}", hello_proto_path))
    .output()
    .expect("Failed to start protoc");

  if !protoc_result.status.success() {
    panic!("protoc failed with exit code {}:{}", protoc_result.status.code().unwrap(), String::from_utf8(protoc_result.stdout).unwrap());
  }

  panic!("");
}