use std::{net::SocketAddr, sync::Arc};

use tonic::transport::Server;

use super::{kad::KadNode, protocol};


pub async fn server(addr: SocketAddr, node: Arc<KadNode>) {
   /* 
   // kadn.print_rtable();
    let key = key::Key::new("25".to_owned() + &"1616".to_owned());
    println!("{:?}", kadn.lookup(key));
    */
    //let ip = buffer.split(':').collect();
    let protocol = protocol::KademliaProtocol::new(node);

    /* for i in 0..50 {
        let k = kad::KadNode::new(i.to_string(),1616);
        protocol.node.insert(k.as_contact());
    } */
    let svc = protocol.create_server();

    Server::builder()
        .add_service(svc)
        .serve(addr).await.unwrap();
}
