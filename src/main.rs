mod network;

use network::kad as kad;
use network::node as nd;
use network::key as key;
use network::rtable as rt;
use to_binary::BinaryString;



fn main() {
    let ip = String::from("127.0.0.");
    let port = 5050;
    let mut origin = kad::KadNode::new(String::from("192.0.0.1"),port);
    for i in 0..20 {
        let insertip = ip.clone() + &i.to_string();
        let k = kad::KadNode::new(insertip,port);
        origin.insert(k.as_contact());
    } 

    let insertip = String::from("192.0.0.2");
    let k = kad::KadNode::new(insertip,port);
    let look = key::Key::new(String::from("127.0.0.1"));

    origin.insert(k.as_contact());
    //origin.print_rtable();
    println!("{:?}",origin.lookup(look).bucket.as_ref().unwrap().len());
    println!("{:?}",origin.lookup(k.uid).bucket.as_ref().unwrap().len());


}
