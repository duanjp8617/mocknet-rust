#![cfg_attr(feature = "cargo-clippy", allow(clippy::wrong_self_convention))]
include!(concat!(env!("OUT_DIR"), "/capnp/indradb_capnp.rs"));

pub mod hello_world {
    tonic::include_proto!("helloworld");
}