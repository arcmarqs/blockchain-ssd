mod network;

use network::kad as kad;
use network::node as nd;
use network::key as key;


fn main() {
    let n = nd::Node::new();
    
    println!("Hello, world!");

}
