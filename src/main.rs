use std::sync::Arc;
use std::{io, net::SocketAddr};
mod p2p;
use p2p::client::Client;
use p2p::kad::KadNode;
use p2p::node::Contact;
use p2p::server;
use std::env;
use tokio::time;
use tokio::{signal, task, time::Duration};
mod auctions;
use auctions as auct;

use crate::p2p::client::send_ping;
use crate::p2p::key::{verify_puzzle, NodeID};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    //  let ip: Vec<&str>= args[1].split(":").collect();
    //  let port = ip[1].to_string().parse::<u16>().unwrap();
    let address: String = args[1].split('\n').collect();    
    let is_bootstrap: String = args[2].split('\n').collect();
    println!("address: {:?}", address);
    let node = Arc::new(KadNode::new(address.clone()));
    let cl = Client::new(node.clone());
    let svnode = node.clone();
    //let plsinsert = KadNode::new("192.0.0.3:50050".to_owned());
   // node.insert(plsinsert.as_contact());
    let addr = address.parse().unwrap();

    task::spawn(async move { 
      server::server(addr, svnode).await 
    });
    
    if is_bootstrap != "bootstrap" {
      let _ = cl.bootstrap().await;
    }
    let ping = send_ping(&address, node.get_validator(), node.as_contact()).await;
     println!("{:?}",ping);
    //let res = cl.send_fnode(node.uid).await;
   // println!("closest {:?}", res);

    //time::sleep(Duration::from_secs(5)).await;

    signal::ctrl_c().await?;
    Ok(())
}
