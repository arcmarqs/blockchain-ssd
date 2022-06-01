use std::{sync::Arc, collections::HashSet, cmp::Ordering};
use chrono::Utc;
use futures::future::join_all;
use log::warn;
use parking_lot::{RwLock, Mutex};

use super::{
    node::Contact, 
    key::NodeID, 
    kad::KadNode, 
    signatures::Signer,
    util::gen_cookie, K_MAX_ENTRIES, kademlia::{kademlia_client::KademliaClient, FValueReq, Header, StoreReq, FNodeReq, Kcontact, self}
};

const PARALLEL_LOOKUPS: i32 = 3;


#[derive(Debug,Clone)]
struct FNodeManager {
    k_closest: Arc<Mutex<Vec<Contact>>>,
    nodes_to_visit: Arc<Mutex<Vec<Contact>>>,
    visited_nodes : Arc<RwLock<HashSet<NodeID>>>,  
}

impl FNodeManager {
    pub fn new(k_closest: Vec<Contact>, nodes_to_visit: Vec<Contact>, visited_nodes: HashSet<NodeID>) -> Self {
        Self {
            k_closest : Arc::new(Mutex::new(k_closest)),
            nodes_to_visit : Arc::new(Mutex::new(nodes_to_visit)),
            visited_nodes : Arc::new(RwLock::new(visited_nodes)),
        }
    }

    pub fn pop_ntv(&self) -> Option<Contact> {
        self.nodes_to_visit.lock().pop()
    }

    pub fn insert_vn(&self,id: NodeID) -> bool {
        self.visited_nodes.write().insert(id)
    }

    pub fn contains_vn(&self,id: &NodeID) -> bool {
        self.visited_nodes.read().contains(id)
    }

    pub fn get_k_closest(&self) -> Vec<Contact> {
       (*self.k_closest.lock()).clone()
    }
}
#[derive(Debug,Clone)]
pub struct Client {
    node: Arc<KadNode>,
}


impl Client {
    pub fn new(node: Arc<KadNode>) -> Client {
        Client { 
            node
        }
    }

    pub async fn send_fnode(&'static self, key: NodeID) -> Vec<Contact> {
        let my_closest = self.node.lookup(key);
        println!("close: {:?}", my_closest);
        let nodes_to_visit:Vec<Contact> = my_closest.iter().map(|a| *a.clone()).collect();
        let k_closest= nodes_to_visit.clone();
        let mut visited_nodes = HashSet::<NodeID>::new();
        visited_nodes.insert(self.node.uid);
        let info = Arc::new(FNodeManager::new(k_closest,nodes_to_visit,visited_nodes));
        let mut handles = Vec::with_capacity(PARALLEL_LOOKUPS.try_into().unwrap());
        for _i in 0..PARALLEL_LOOKUPS {
            handles.push(self.a_lookup(key,info.clone()));
        }
        
        join_all(handles.into_iter().map(tokio::spawn)).await;
        //println!("closest {:?}", info);
        info.get_k_closest()
    }

    async fn send_fvalue(&'static self, key: NodeID) -> Option<Vec<u8>> {
        if let Some(maybe_value) = self.node.retrieve(key) {
            return Some(maybe_value.as_bytes().to_owned());
        }
        
        let k_closest = self.send_fnode(key).await;
        for contact in k_closest {
            let mut client = KademliaClient::connect(contact.address.clone()).await.unwrap();  
            let timestamp = Utc::now().timestamp();
            let request = FValueReq {
                cookie: gen_cookie(),
                header: Some( Header {
                    my_id: self.node.uid.as_bytes().to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature: Signer::sign_strong_header_req(timestamp,&contact.get_pubkey(),&self.node.address,key.as_bytes().to_owned()),
                }),
                target_id: key.as_bytes().to_owned(),
            };

            match client.find_value(request).await {
                Ok(res) => {
                    let response = res.into_inner();
                    match response.has_value.unwrap() {
                        kademlia::f_value_repl::HasValue::Node(n) => {
                            continue;
                        },
                        kademlia::f_value_repl::HasValue::Value(v) => {
                            return Some(v);
                        },
                    }
                },
                Err(_) => {
                    warn!("failed to contact node");
                    return None;
                },
            };
        }
        None
    }

    async fn send_store(&self,key:NodeID, value:String, contact: Contact) -> Result<(), Box<dyn std::error::Error>> {
        let mut client = KademliaClient::connect(contact.address.clone()).await?;
        let timestamp = Utc::now().timestamp();
        let request = StoreReq {
                cookie: gen_cookie(),
                header: Some( Header {
                    my_id: self.node.uid.as_bytes().to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature: Signer::sign_strong_header_req(timestamp,&contact.get_pubkey(),&self.node.address,key.as_bytes().to_owned()),
                }),
                target_id: key.as_bytes().to_owned(),
                value: value,
            };
        let rep =  client.store(request).await;
        

        Ok(())
    }

async fn a_lookup(&self, key: NodeID, info: Arc<FNodeManager>) {
    let mut local_visit = Vec::new();
     loop {
         if local_visit.is_empty(){
             if let Some(local_visit_node) = info.pop_ntv(){
            println!("{:?}", local_visit_node);
             local_visit.push(local_visit_node);
             } else {
                return;
             }
         }

         let node = local_visit.pop().unwrap();
         if info.contains_vn(&node.uid) {
             continue
         }

         let remote = KademliaClient::connect(node.address.clone()).await;
         match remote {
             Ok(mut remote) => {
                let timestamp = Utc::now().timestamp();
                 let request = FNodeReq {
                     cookie: gen_cookie(),
                     header: Some( Header {
                        my_id: self.node.uid.as_bytes().to_owned(),
                        pub_key: self.node.get_pubkey(),
                        nonce: self.node.get_nonce(),
                        timestamp,
                        signature: Signer::sign_strong_header_req(timestamp,&node.get_pubkey(),&self.node.address,key.as_bytes().to_owned()),
                    }),
                     target_id: key.as_bytes().to_owned(),
                 };

                 let response = remote.find_node(request).await;
                 info.insert_vn(node.uid);

                 match response {
                     Ok(response) =>{
                         let response = response.into_inner();
                         let closest_to_contact = contact_list(response.nodes.unwrap().node);
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
                         warn!("node {:?} didn't respond: {} ", &node.address, err);
                     },
                 };
             }
             Err(err) =>{
                 warn!("address {:?} unreachable, removing from route table: {}", &node.address,err);
                 //node still in visited so it's not contacted again.
                 info.insert_vn(node.uid);
             }  
         }
     }   
 }
}

// Inserts all contacts that are closest to the key relative to the ones already in the bucket and pushes them into the visiting list, if none are closer, returns false
fn insert_closest(k_closest:&mut Vec<Contact>, mut local_visit: Vec<Contact>,mut closest_to_contact: Vec<Contact>,key: NodeID) -> (Vec<Contact>,bool) {
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

pub fn contact_list(kcontact_list: Vec<Kcontact>) -> Vec<Contact> {
    let converter = |k: &Kcontact| {
        Contact::new(
            NodeID::from_vec(k.uid.clone()),
            k.address.clone(),
            k.pub_key.clone()
        )
    };
    kcontact_list.iter().map(|a| converter(a) ).collect()
}
