pub mod kad;
pub mod key;
pub mod node;
pub mod rtable;
pub mod protocol;
pub mod server;
pub mod client;
mod util;
mod signatures;
mod kademlia {
    tonic::include_proto!("kadproto");
}
pub const K_MAX_ENTRIES: usize = 12;
pub const C1: u32= 8;
pub const C2: u32 = 16;