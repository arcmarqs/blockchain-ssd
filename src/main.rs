mod network;

use network::kad as kad;
use network::node as nd;
use network::key as key;

fn main() {
    let k =key::Key::new(String::from("192.4.5.6"));
    let other_key = key::Key::new(String::from("192.4.5.7"));
    println!("Hello, world!");
    println!("{:?}",k.dist(other_key));
    
}
