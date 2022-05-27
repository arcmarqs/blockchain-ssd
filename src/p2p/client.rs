
use tokio::{task, sync::{RwLock, Mutex}};
use tonic::{transport::{Server, Channel}, Request, Response, Status};
use log::{info, trace, warn};
use to_binary::BinaryString;
use std::{io, net::SocketAddr, sync::Arc, collections::HashSet, cmp::Ordering};
use super::{
    kad::KadNode,
    node::Contact,
    key::Key, K_MAX_ENTRIES,
};
use rand::Rng;
use kademlia::kademlia_client::KademliaClient;
use kademlia::{PingM,StoreReq,StoreRepl,FNodeReq,FNodeRepl,FValueReq,FValueRepl,Kcontact};

pub mod kademlia {
    tonic::include_proto!("kadproto");
}

#[derive(Debug,Clone)]
struct FNodeManager {
    k_closest: Arc<RwLock<Vec<Contact>>>,
    nodes_to_visit: Arc<Mutex<Vec<Contact>>>,
    visited_nodes : Arc<RwLock<HashSet<Key>>>,  
}
#[derive(Debug,Default,Clone)]
pub struct Client {
    node: Arc<KadNode>,
}


impl Client {
    pub fn new(node: Arc<KadNode>) -> Client {
        Client { 
            node
        }
    }

    pub async fn send_ping(self,contact: Contact) {
        self.node.print_rtable().await;
        let addr = format_address(contact.ip,contact.port);
        let mut client = KademliaClient::connect(addr).await.unwrap();  
        let mut rng = rand::thread_rng();
        let cookie: usize = rng.gen();
        let request = Request::new(
            PingM {
                cookie: cookie.to_string(),
                id : self.node.uid.as_bytes().to_owned(),
            }
        );

        let response = client.ping(request).await.unwrap();
        println!("{:?}", response.into_inner());
    }

    async fn send_fnode(self, key: Key) {
        let my_closest = self.node.lookup(key).await;
        let mut nodes_to_visit: Vec<Contact> = Vec::new();
        let mut visited_nodes = HashSet::<Key>::new();
    }

    async fn send_fvalue(mynode: Arc<KadNode>,contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format_address(contact.ip,contact.port);
        let client = KademliaClient::connect(addr).await?;  
        todo!();
    }

    async fn send_store(mynode: Arc<KadNode>,contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format_address(contact.ip,contact.port);
        let client = KademliaClient::connect(addr).await?;
        todo!();
    }

    async fn a_lookup(self,key: Key, info: FNodeManager) {
        let local_visit_node = info.nodes_to_visit.lock().await.pop();
        if local_visit_node.is_none() {
            return
        }
        let mut local_k_closest: Vec<Contact> = Vec::new();
        let mut local_visit = vec![local_visit_node.unwrap()];
        while !local_visit.is_empty() {
            let node = local_visit.pop().unwrap();
            if info.visited_nodes.read().await.contains(&node.uid){
                continue
            }

            let addr = format_address(node.ip,node.port);
            let remote = KademliaClient::connect(addr.clone()).await;
            match remote {
                Ok(mut remote) => {
                    let mut rng = rand::thread_rng();
                    let cookie: usize = rng.gen();
                    let request = FNodeReq {
                        cookie: cookie.to_string(),
                        uid: key.as_bytes().to_owned(),
                    };

                    let response = remote.find_node(request).await;
                    info.visited_nodes.write().await.insert(node.uid);

                    match response {
                        Ok(response) =>{
                            let response = response.into_inner();
                            let closest_to_contact = contact_list(response.knode);
                            let (lkl,lv,success) = insert_closest(local_k_closest.clone(),local_visit.clone(),closest_to_contact, key);
                                if !success {
                                    return;
                                }
                                local_k_closest = lkl;
                                local_visit = lv;
                        },
                        Err(err) => {
                            warn!("node {:?} didn't respond: {} ", addr, err);
                            // function to remove contact from route table
                        },
                    };
                }
                Err(err) =>{
                    warn!("address {:?} unreachable, removing from route table: {}",addr,err);
                    //node still in visited so it's not contacted again.
                    info.visited_nodes.write().await.insert(node.uid);
                    // need to implement function to remove contact from route table.
                }  
            }
        }   
    }
}


fn format_address(ip: String, port: u16) -> String {
    ("http://".to_owned() + &ip + ":" + &port.to_string()).to_owned()
}

fn format_kcontact(contact: Contact) -> Kcontact {
    Kcontact {
        uid : contact.uid.as_bytes().to_owned(),
        ip: contact.ip.clone(),
        port: contact.port.clone() as i32,
    }
}

fn contact_list(kcontact_list: Vec<Kcontact>) -> Vec<Contact> {
    let converter = |k: &Kcontact| {
        Contact {
            uid: Key::from_vec(k.uid.clone()),
            ip: k.ip.clone(),
            port: k.port as u16,
        }
    };

    kcontact_list.iter().map(|a| converter(a) ).collect()
}

// Inserts all contacts that are closest to the key relative to the ones already in the bucket and pushes them into the visiting list, if none are closer, returns false
fn insert_closest(mut local_k_closest: Vec<Contact>,mut local_visit: Vec<Contact>,mut closest_to_contact: Vec<Contact>,key: Key) -> (Vec<Contact>,Vec<Contact>,bool) {
    let prev_len = closest_to_contact.len();

    while local_k_closest.len() < K_MAX_ENTRIES && !closest_to_contact.is_empty() {
        let node = closest_to_contact.pop().unwrap();
        local_k_closest.push(node.clone());
        local_visit.push(node);
    }

    if closest_to_contact.is_empty() {
       return (local_k_closest,local_visit,true);
    }

    let dist = |a:&Contact, b: &Contact| {
        key.distance(a.uid).partial_cmp(&key.distance(b.uid))
    };

    for ctc_node in closest_to_contact.clone() {
        let mut _index = 0;
        for lkl_node in local_k_closest.clone() {
            match dist(&ctc_node, &lkl_node) {
                Some(Ordering::Less) => {
                    local_k_closest.push(ctc_node.clone());
                    local_visit.push(ctc_node.clone());
                    closest_to_contact.remove(_index);
                },
                Some(_) =>continue,
                None => panic!("Invalid key format"),
            };
        }
        _index += 1;
    }

    local_k_closest.sort_by(|a,b| dist(a,b).unwrap());
    local_visit.sort_by(|a,b| dist(a,b).unwrap());
    local_k_closest.truncate(K_MAX_ENTRIES);
    local_visit.truncate(K_MAX_ENTRIES);
    let success = !prev_len == closest_to_contact.len();
    (local_k_closest,local_visit,success)

}