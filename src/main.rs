mod network;

use network::kad as kad;
use network::node as nd;
use network::key as key;
use network::rtable as rt;
use to_binary::BinaryString;



fn main() {
    let mut rt = rt::Rtable::new();
    let ip1 = String::from("192.10.1");
    let port: u16 = 5050;
    let k1 = key::Key::new(ip1.clone() + &port.to_string());
    let con1 = nd::Contact::new(k1,ip1,port);
    rt.init(con1);

    let ip2 = String::from("192.10.2");
    let k2 = key::Key::new(ip2.clone() + &port.to_string());
    let con2 = nd::Contact::new(k2,ip2,port);
    let ip3 = String::from("192.10.2");
    let k3 = key::Key::new(ip3.clone() + &port.to_string());
    let con3 = nd::Contact::new(k3,ip3,port);

    rt.insert(con2);
    rt.insert(con3);

    println!("Hello, world!");
    println!("{:?}, {:?}",rt.lookup(k3),rt.lookup(k2));

}
