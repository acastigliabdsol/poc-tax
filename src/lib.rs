pub mod domain;
pub mod infra;
pub mod app;

pub mod schema_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema_capnp.rs"));
}
