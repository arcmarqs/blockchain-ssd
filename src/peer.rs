use std::sync::Arc;
use std::{io,net::SocketAddr};
mod p2p;
use p2p::server;
use p2p::node::Contact;
use p2p::client::Client;
use p2p::kad::KadNode;
use tokio::time;
use tokio::{task, signal, time::Duration};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let ip: Vec<&str>= args[1].split(":").collect();
    let port = ip[1].to_string().parse::<u16>().unwrap();
    let n = KadNode::new(ip[0].to_string(), port);
    println!("{:?}",n);
    /*
    let addr =SocketAddr::new(ip[0].to_string().parse()?, port);
    let node = Arc::new(KadNode::new(ip[0].to_string(), port));
    let svnode = node.clone();
    let cl = Client::new(node.clone());
    let plsinsert = KadNode::new("192.0.0.3".to_owned(),60064);
    node.rtable.write().insert(plsinsert.as_contact(),node.uid);
    task::spawn(async move {
        server::server(addr,svnode).await
    });
    

    let res = cl.send_fnode(node.uid).await;
    println!("{:?}",res);
    */
   //time::sleep(Duration::from_secs(5)).await;
 
   //cl.send_ping(node.as_contact()).await;

   signal::ctrl_c().await?; 
    Ok(())
}