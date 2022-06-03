use std::{sync::Arc, collections::HashSet, cmp::Ordering};
use chrono::Utc;
use futures::future::join_all;
use log::warn;
use parking_lot::{RwLock, Mutex};
use prost::Message;

use super::{
    node::Contact, 
    key::{NodeID, NodeValidator}, 
    kad::KadNode, 
    signatures::Signer,
    util::{gen_cookie, format_address}, K_MAX_ENTRIES, kademlia::{kademlia_client::KademliaClient, FValueReq, Header, StoreReq, FNodeReq, Kcontact, self, PingM}
};

const PARALLEL_LOOKUPS: i32 = 3;

static BOOTSTRAP_KEY: &'static [u8] = &[45, 45, 45, 45, 45, 66, 69, 71, 73, 78, 32, 80, 85, 66, 76, 73, 67, 32, 75, 69, 89, 45, 45, 45, 45, 45, 10, 77, 73, 73, 66, 73, 
                                        106, 65, 78, 66, 103, 107, 113, 104, 107, 105, 71, 57, 119, 48, 66, 65, 81, 69, 70, 65, 65, 79, 67, 65, 81, 56, 65, 77, 73, 73, 
                                        66, 67, 103, 75, 67, 65, 81, 69, 65, 48, 104, 108, 67, 67, 72, 97, 90, 103, 82, 122, 78, 72, 98, 107, 88, 67, 56, 57, 114, 10,
                                        97, 67, 51, 71, 112, 89, 78, 101, 118, 86, 54, 100, 54, 48, 115, 53, 104, 108, 90, 72, 70, 98, 107, 72, 106, 108, 66, 106, 113, 
                                        72, 65, 109, 56, 84, 97, 115, 73, 110, 97, 80, 109, 75, 98, 52, 90, 54, 112, 116, 86, 71, 107, 108, 97, 74, 121, 68, 99, 56,
                                        76, 118, 83, 102, 107, 89, 10, 113, 75, 99, 70, 110, 83, 113, 102, 110, 74, 107, 48, 79, 112, 102, 80, 75, 52, 107, 68, 89, 72, 
                                        98, 97, 54, 74, 55, 79, 73, 48, 104, 119, 49, 105, 79, 74, 99, 51, 89, 113, 71, 67, 79, 112, 80, 100, 53, 49, 84, 74, 109, 65,
                                        103, 118, 67, 120, 120, 87, 102, 69, 72, 55, 104, 114, 10, 78, 118, 82, 51, 68, 87, 112, 55, 82, 120, 89, 119, 56, 84, 87, 80, 
                                        70, 105, 52, 103, 108, 76, 117, 68, 112, 105, 76, 106, 111, 109, 51, 78, 98, 99, 99, 76, 87, 66, 85, 87, 50, 53, 97, 111, 103, 
                                        53, 80, 67, 76, 118, 106, 70, 105, 98, 112, 68, 118, 79, 81, 69, 120, 109, 102, 109, 10, 69, 68, 55, 84, 108, 51, 47, 112, 104,
                                        74, 65, 115, 83, 66, 69, 76, 74, 47, 99, 82, 53, 88, 75, 71, 106, 98, 71, 83, 115, 120, 53, 53, 114, 109, 50, 82, 74, 55, 102,
                                        73, 43, 55, 122, 68, 83, 107, 52, 113, 108, 90, 47, 83, 78, 120, 73, 78, 53, 86, 121, 107, 116, 52, 117, 57, 10, 65, 77, 68, 99, 
                                        102, 107, 106, 99, 73, 120, 100, 116, 50, 69, 69, 43, 97, 86, 51, 80, 114, 113, 118, 57, 78, 79, 50, 73, 81, 104, 67, 110, 89, 
                                        112, 55, 43, 71, 79, 121, 50, 81, 55, 112, 107, 66, 106, 47, 56, 104, 67, 103, 65, 90, 87, 102, 51, 55, 69, 74, 112, 54, 53, 73, 
                                        111, 10, 108, 119, 73, 68, 65, 81, 65, 66, 10, 45, 45, 45, 45, 45, 69, 78, 68, 32, 80, 85, 66, 76, 73, 67, 32, 75, 69, 89, 45, 45, 
                                        45, 45, 45, 10];
                                
static BOOT_ID : &'static [u8] = &[245, 83, 36, 87, 32, 76, 92, 218, 215, 197, 141, 139, 118, 64, 71, 198, 83, 109, 36, 28, 81, 38, 245, 179, 55, 172, 28, 49, 226, 8, 22, 84];
static BOOTSTRAP_IP : &str = "10.128.0.3:30030";


#[derive(Debug,Clone)]
struct FNodeManager {
    k_closest: Arc<Mutex<Vec<Contact>>>,
    nodes_to_visit: Arc<Mutex<Vec<Contact>>>,
    visited_nodes : Arc<RwLock<HashSet<NodeID>>>,  
    validator: NodeValidator,
    address: String,
}

impl FNodeManager {
    pub fn new(k_closest: Vec<Contact>, nodes_to_visit: Vec<Contact>, visited_nodes: HashSet<NodeID>, validator: NodeValidator, address: String) -> Self {
        Self {
            k_closest : Arc::new(Mutex::new(k_closest)),
            nodes_to_visit : Arc::new(Mutex::new(nodes_to_visit)),
            visited_nodes : Arc::new(RwLock::new(visited_nodes)),
            validator : validator,
            address : address,
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

    pub fn get_pubkey(&self) -> Vec<u8> {
        self.validator.get_pubkey()
    }

    pub fn get_nonce(&self) -> u64 {
        self.validator.get_nonce()
    }

    pub fn get_address(&self) -> String {
        self.address.clone()
    }

    pub fn get_uid(&self) -> Vec<u8> {
        self.validator.get_nodeid().as_bytes().to_owned()
    }

    pub fn get_validator(&self) -> &NodeValidator {
        &self.validator
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

    pub async fn bootstrap(&self) -> Result<(), &str> {
        self.node.insert(Contact::new(NodeID::from_vec(BOOT_ID.to_vec()), BOOTSTRAP_IP.to_owned(), BOOTSTRAP_KEY.to_vec()));
        let k_closest = self.send_fnode(self.node.uid).await;
        for node in k_closest {
            self.node.insert(node);
        }
        self.node.print_rtable();
        Ok(())
    }

    pub async fn send_fnode(&self, key: NodeID) -> Vec<Contact> {
        let my_closest = self.node.lookup(key);
        let nodes_to_visit:Vec<Contact> = my_closest.iter().map(|a| *a.clone()).collect();
        let k_closest= nodes_to_visit.clone();
        let mut visited_nodes = HashSet::<NodeID>::new();
        visited_nodes.insert(self.node.uid);
        let find_nodes = FNodeManager::new(k_closest,nodes_to_visit,visited_nodes,self.node.get_validator().clone(),self.node.address.clone());
        let info = Arc::new(find_nodes);
        let mut handles = Vec::with_capacity(PARALLEL_LOOKUPS.try_into().unwrap());
        for _i in 0..PARALLEL_LOOKUPS {
            handles.push(a_lookup(key,info.clone()));
        }
        
        join_all(handles.into_iter().map(tokio::spawn)).await;
        //println!("closest {:?}", info);
        info.get_k_closest()
    }

    async fn send_fvalue(&self, key: NodeID) -> Option<Vec<u8>> {
        if let Some(maybe_value) = self.node.retrieve(key) {
            return Some(maybe_value.as_bytes().to_owned());
        }
        
        let k_closest = self.send_fnode(key).await;
        for contact in k_closest {
            let address = format_address(contact.address.clone());
            let mut client = KademliaClient::connect(address.clone()).await.unwrap();  
            let timestamp = Utc::now().timestamp();
            let (hash,request_signature) = Signer::sign_strong_header_req(timestamp,&contact.get_pubkey(),&self.node.address,key.as_bytes().to_owned());
            let request = FValueReq {
                cookie: gen_cookie(),
                header: Some( Header {
                    my_id: self.node.uid.as_bytes().to_owned(),
                    address : self.node.address.to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature: request_signature.clone(),
                }),
                target_id: key.as_bytes().to_owned(),
            };

            match client.find_value(request).await {
                Ok(res) => {
                    let response = res.into_inner();
                    let header = response.header.unwrap();
                    let data = response.has_value.unwrap();
                    let mut databuf = Vec::new();
                    data.encode(&mut databuf);
                    if let Ok(()) = Signer::validate_strong_rep(self.node.get_validator(),&header,&contact.address,&databuf,&hash) {
                        match data {
                            kademlia::f_value_repl::HasValue::Node(_) => continue,
                            kademlia::f_value_repl::HasValue::Value(value) =>{
                                return Some(value);
                            }
                        }
                    }
                 }
                Err(_) => warn!("failed to unwrap message"),    
            }
        }
        None
    }

    async fn send_store(&self,key:NodeID, value:String, contact: Contact) -> Result<(),&'static str> {
        let address = format_address(contact.address.clone());
        let mut client = KademliaClient::connect(address.clone()).await.unwrap();  
        let timestamp = Utc::now().timestamp();
        let (hash,request_signature) = Signer::sign_strong_header_req(timestamp,&contact.get_pubkey(),&self.node.address,key.as_bytes().to_owned());
        let request = StoreReq {
                cookie: gen_cookie(),
                header: Some( Header {
                    my_id: self.node.uid.as_bytes().to_owned(),
                    address : self.node.address.to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature: request_signature.clone() ,
                }),
                target_id: key.as_bytes().to_owned(),
                value: value,
            };
        let response =  client.store(request).await;

        match response {
            Ok(res) => {
                let res = res.into_inner();
                let header = res.header.unwrap();
                if let Ok(()) = Signer::validate_weak_rep(self.node.get_validator(),&header,&contact.address,&hash) {
                    return Ok(());
                }
                Err("Failed to verify signature")
             },
            Err(_) => Err("Failed to unwrap response"),
        }
    }

}

async fn a_lookup(key: NodeID, info: Arc<FNodeManager>) {
    let mut local_visit = Vec::new();
     loop {
         if local_visit.is_empty(){
             if let Some(local_visit_node) = info.pop_ntv(){
            println!("here");
             local_visit.push(local_visit_node);
             } else {
                return;
             }
         }

         let node = local_visit.pop().unwrap();
         if info.contains_vn(&node.uid) {
             continue
         }

         let address = format_address(node.address.clone());
         let remote = KademliaClient::connect(address.clone()).await;  
         match remote {
             Ok(mut remote) => {
                let timestamp = Utc::now().timestamp();
                let (hash,request_signature) = Signer::sign_weak_header_req(timestamp,&info.get_pubkey(),&info.address);
                let request = FNodeReq {
                    cookie: gen_cookie(),
                    header: Some( Header {
                        my_id: info.get_uid(),
                        address : info.address.to_owned(),
                        pub_key: info.get_pubkey(),
                        nonce: info.get_nonce(),
                        timestamp,
                        signature: request_signature.clone(),
                    }),
                     target_id: key.as_bytes().to_owned(),
                 };

                 let response = remote.find_node(request).await;
                 info.insert_vn(node.uid);

                 match response {
                     Ok(response) =>{
                         let response = response.into_inner();
                         let header = response.header.unwrap();
                         let data = response.nodes.unwrap();
                         let mut databuf = Vec::new();
                         let _ = data.encode(&mut databuf);
                         if let Ok(()) = Signer::validate_strong_rep(info.get_validator(),&header,&node.address,&databuf,&hash) {
                            let closest_to_contact = contact_list(data.node);
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
                        }
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

pub async fn send_ping(my_address: &str,validator: &NodeValidator, contact: Contact) -> bool {
    let address = format_address(contact.address.clone());
    if let Ok(mut client) = KademliaClient::connect(address.clone()).await{  
    let timestamp = Utc::now().timestamp();
    let (hash,request_signature) = Signer::sign_weak_header_req(timestamp,&validator.get_pubkey(),my_address);
    let request = PingM {
            cookie: gen_cookie(),
            header: Some( Header {
                my_id: validator.get_nodeid().as_bytes().to_owned(),
                address : my_address.to_owned(),
                pub_key: validator.get_pubkey(),
                nonce: validator.get_nonce(),
                timestamp: timestamp,
                signature: request_signature.clone(),
            }),
        };

    match client.ping(request).await {
        Ok(response) => {
            let res = response.into_inner();
            let header = res.header.unwrap();
            if let Ok(()) = Signer::validate_weak_rep(validator,&header,&contact.address,&hash) {
                return true;
            } else {
                return false;
            }

        },
        Err(_) => false,
    }
    } else {
        false
    }
}