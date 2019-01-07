use dirs::home_dir;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let cargo_dir = home_dir()
        .expect("Failed to get user directory")
        .join(".cargo")
        .join("bin");

    let grpc_rust_plugin_path = if cfg!(windows) {
        cargo_dir.join("grpc_rust_plugin.exe")
    } else {
        cargo_dir.join("grpc_rust_plugin")
    };

    let protoc_path = env::current_dir()
        .unwrap()
        .join("..")
        .join("tools")
        .join("out")
        .join("protoc")
        .join("bin")
        .join("protoc");

    let proto_dir = env::current_dir()
      .unwrap()
      .join("proto");

    let hello_proto_path = proto_dir
      .join("hello.proto");

    let protoc_out = env::current_dir()
      .unwrap()
      .join("src")
      .join("gen");

    println!("{:?}", grpc_rust_plugin_path);

    let protoc_result = Command::new(protoc_path)
        .arg(format!("--rust_out={}", protoc_out.to_string_lossy()))
        .arg(format!("--grpc_out={}", protoc_out.to_string_lossy()))
        .arg(format!(
            "--plugin=protoc-gen-grpc={}",
            grpc_rust_plugin_path.to_string_lossy()
        ))
        .arg(format!("--proto_path={}", proto_dir.to_string_lossy()))
        .arg(format!("{}", hello_proto_path.to_string_lossy()))
        .output()
        .expect("Failed to start protoc");
   
    println!("protoc stdout:{}", String::from_utf8_lossy(&protoc_result.stdout));
    println!("protoc stderr:{}", String::from_utf8_lossy(&protoc_result.stderr));

    if !protoc_result.status.success() {
        panic!(
            "protoc failed with exit code {}",
            protoc_result.status.code().unwrap()
        );
    }
}
