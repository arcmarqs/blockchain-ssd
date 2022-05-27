
use parking_lot::{Mutex,RwLock};
use tokio::{task::{self, JoinHandle}, try_join};
use tonic::{transport::{Server, Channel}, Request, Response, Status};
use log::{info, trace, warn};
use to_binary::BinaryString;
use std::{io, net::SocketAddr, sync::{Arc}, collections::HashSet, cmp::Ordering};
use futures::future::join_all;
use super::{
    kad::KadNode,
    node::Contact,
    key::Key, K_MAX_ENTRIES,
};
use rand::Rng;
use kademlia::kademlia_client::KademliaClient;
use kademlia::{PingM,StoreReq,StoreRepl,FNodeReq,FNodeRepl,FValueReq,FValueRepl,Kcontact};
const PARALLEL_LOOKUPS: usize = 3;
pub mod kademlia {
    tonic::include_proto!("kadproto");
}

#[derive(Debug,Clone)]
struct FNodeManager {
    k_closest: Arc<Mutex<Vec<Contact>>>,
    nodes_to_visit: Arc<Mutex<Vec<Contact>>>,
    visited_nodes : Arc<RwLock<HashSet<Key>>>,  
}

impl FNodeManager {
    pub fn new(k_closest: Vec<Contact>,nodes_to_visit: Vec<Contact>, visited_nodes: HashSet<Key>) -> FNodeManager {
        FNodeManager {
            k_closest: Arc::new(Mutex::new(k_closest)),
            nodes_to_visit: Arc::new(Mutex::new(nodes_to_visit)),
            visited_nodes: Arc::new(RwLock::new(visited_nodes)),
        }
    }

    pub fn pop_ntv(&self) -> Option<Contact> {
        self.nodes_to_visit.lock().pop()
    }

    pub fn insert_vn(&self,id: Key) -> bool {
        self.visited_nodes.write().insert(id)
    }

    pub fn contains_vn(&self,id: &Key) -> bool {
        self.visited_nodes.read().contains(id)
    }


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
        self.node.print_rtable();
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

    async fn send_fnode (self, key: Key) -> Result<(), Box<dyn std::error::Error>> {
        let my_closest = self.node.lookup(key);
        let nodes_to_visit:Vec<Contact> = my_closest.iter().map(|a| *a.clone()).collect();
        let k_closest= nodes_to_visit.clone();
        let mut visited_nodes = HashSet::<Key>::new();
        visited_nodes.insert(self.node.uid);
        let info = Arc::new(FNodeManager::new(k_closest,nodes_to_visit,visited_nodes));
        let mut handles = Vec::with_capacity(PARALLEL_LOOKUPS);
        
        for i in 0..PARALLEL_LOOKUPS {
            let lookup_info = info.clone();
            handles.push(a_lookup(key.clone(), lookup_info));
        }
        
        join_all(handles.into_iter().map(tokio::spawn));

        Ok(())
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
}

async fn a_lookup(key: Key, info: Arc<FNodeManager>) {
    let mut local_visit = Vec::new();
     loop {

         if local_visit.is_empty(){
             if let Some(local_visit_node) = info.pop_ntv(){
             local_visit.push(local_visit_node);
             } else {
                 return;
             }
         }

         let node = local_visit.pop().unwrap();
         if info.contains_vn(&node.uid) {
             continue
         }

         let addr = format_address(node.ip,node.port);
         let remote = KademliaClient::connect(addr.clone()).await;
         match remote {
             Ok(mut remote) => {
                 let request = FNodeReq {
                     cookie: gen_cookie(),
                     uid: key.as_bytes().to_owned(),
                 };

                 let response = remote.find_node(request).await;
                 info.insert_vn(node.uid);

                 match response {
                     Ok(response) =>{
                         let response = response.into_inner();
                         let closest_to_contact = contact_list(response.knode);
                         let (lv,success):(Vec<Contact>,bool);
                         // limiting the range of the lock
                         {
                             let mut lock_k_closest = info.k_closest.lock();
                             (lv,success) = insert_closest(&mut *lock_k_closest,local_visit.clone(),closest_to_contact, key);
                         }
                         if !success {
                             return;
                         }
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
                 info.insert_vn(node.uid);
                 //info.insert_visited(node.uid);
                 // need to implement function to remove contact from route table.
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

fn gen_cookie() -> String {
    let mut rng = rand::thread_rng();
    let cookie: usize = rng.gen();
    cookie.to_string()
}

// Inserts all contacts that are closest to the key relative to the ones already in the bucket and pushes them into the visiting list, if none are closer, returns false
fn insert_closest(k_closest:&mut Vec<Contact>, mut local_visit: Vec<Contact>,mut closest_to_contact: Vec<Contact>,key: Key) -> (Vec<Contact>,bool) {
    let prev_len = closest_to_contact.len();
    let dist = |a:&Contact, b: &Contact| {
        key.distance(a.uid).partial_cmp(&key.distance(b.uid))
    };

    while k_closest.len() < K_MAX_ENTRIES && !closest_to_contact.is_empty() {
        let node = closest_to_contact.pop().unwrap();
        k_closest.push(node.clone());
        local_visit.push(node);
    }

    if closest_to_contact.is_empty() {
       return (local_visit,true);
    }

    for ctc_node in closest_to_contact.clone() {
        let mut _index = 0;
        for lkl_node in k_closest.clone() {
            match dist(&ctc_node, &lkl_node) {
                Some(Ordering::Less) => {
                    k_closest.push(ctc_node.clone());
                    local_visit.push(ctc_node.clone());
                    closest_to_contact.remove(_index);
                },
                Some(_) =>continue,
                None => panic!("Invalid key format"),
            };
        }
        _index += 1;
    }

    k_closest.sort_by(|a,b| dist(a,b).unwrap());
    local_visit.sort_by(|a,b| dist(a,b).unwrap());
    k_closest.truncate(K_MAX_ENTRIES);
    local_visit.truncate(K_MAX_ENTRIES);
    let success = !prev_len == closest_to_contact.len();
    (local_visit,success)
}