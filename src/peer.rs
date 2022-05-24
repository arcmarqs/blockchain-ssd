use std::sync::Arc;
use std::{io,net::SocketAddr};
mod p2p;
use p2p::server;
use p2p::client;
use p2p::kad::KadNode;
use tokio::time;
use tokio::{task, signal, time::Duration};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("Hello");
    let args: Vec<String> = env::args().collect();
    let ip: Vec<&str>= args[1].split(":").collect();
    let port = ip[1].to_string().parse::<u16>().unwrap();
    println!("{:?} {:?}",ip,port);
    let addr =SocketAddr::new(ip[0].to_string().parse()?, port);
    let node = Arc::new(KadNode::new(ip[0].to_string(), port));
    let svnode = node.clone();
    let clnode = node.clone();
    task::spawn(async move {
        server::server(addr,svnode).await
    });
    
   client::Client::send_ping(clnode.clone(),node.as_contact()).await;

   time::sleep(Duration::from_secs(5)).await;

   client::Client::send_ping(clnode,node.as_contact()).await;

   signal::ctrl_c().await?;

    Ok(())
}