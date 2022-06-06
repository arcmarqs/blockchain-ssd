pub mod client;
pub mod kad;
pub mod key;
pub mod node;
pub mod protocol;
pub mod rtable;
pub mod server;
mod signatures;
mod util;
mod kademlia {
    tonic::include_proto!("kadproto");
}
pub const K_MAX_ENTRIES: usize = 5;
pub const C1: u32 = 8;
pub const C2: u32 = 16;
