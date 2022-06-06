use std::{sync::{Arc, atomic::AtomicU64}, collections::HashSet, cmp::Ordering};
use futures::future::join_all;
use parking_lot::{RwLock, Mutex};
use prost::Message;
use tonic::Request;
use std::sync::atomic::Ordering::SeqCst;

use crate::{auctions::auction::AuctionGossip, ledger::block::Data};

use super::{
    node::Contact, 
    key::{NodeID, NodeValidator}, 
    kad::KadNode, 
    signatures::Signer,
    util::{gen_cookie, format_address, to_auction_data, encode_store, to_gossip_vec, grpc_transaction}, K_MAX_ENTRIES, kademlia::{kademlia_client::KademliaClient, FValueReq, Header, StoreReq, FNodeReq, Kcontact, self, PingM, BroadcastReq}
};

const PARALLEL_LOOKUPS: i32 = 3;

static BOOTSTRAP_KEY: &'static [u8] = &[45, 45, 45, 45, 45, 66, 69, 71, 73, 78, 32, 80, 85, 66, 76, 73, 67, 32, 75, 69, 89, 45, 45, 45, 45, 45, 10, 77, 73, 73, 66, 73, 106, 65, 78, 
                                        66, 103, 107, 113, 104, 107, 105, 71, 57, 119, 48, 66, 65, 81, 69, 70, 65, 65, 79, 67, 65, 81, 56, 65, 77, 73, 73, 66, 67, 103, 75, 67, 65, 81, 
                                        69, 65, 121, 85, 65, 113, 99, 113, 69, 112, 79, 120, 56, 110, 86, 105, 101, 90, 79, 110, 81, 54, 10, 68, 118, 104, 101, 85, 72, 72, 114, 57, 50, 
                                        67, 55, 120, 107, 54, 99, 88, 122, 77, 120, 55, 87, 118, 66, 80, 102, 70, 116, 69, 122, 53, 97, 103, 122, 117, 87, 97, 119, 82, 99, 68, 105, 75, 
                                        102, 76, 114, 116, 76, 86, 84, 54, 72, 71, 117, 65, 122, 87, 68, 83, 97, 87, 43, 65, 47, 10, 88, 83, 116, 55, 82, 82, 87, 104, 99, 67, 111, 86, 
                                        110, 121, 88, 70, 74, 108, 89, 76, 88, 43, 79, 72, 82, 70, 104, 103, 82, 53, 57, 105, 111, 57, 113, 120, 109, 101, 122, 111, 102, 89, 107, 121, 
                                        114, 81, 120, 78, 108, 75, 65, 105, 118, 108, 52, 107, 43, 104, 74, 86, 70, 84, 119, 122, 10, 103, 83, 71, 51, 116, 50, 57, 121, 112, 98, 79, 
                                        119, 47, 99, 66, 77, 78, 118, 107, 51, 73, 74, 83, 111, 103, 74, 77, 113, 68, 68, 65, 111, 82, 57, 104, 99, 49, 89, 84, 112, 77, 85, 71, 75, 111, 
                                        90, 52, 99, 86, 104, 70, 90, 52, 67, 112, 122, 54, 82, 78, 74, 81, 121, 87, 114, 10, 80, 107, 66, 54, 56, 112, 71, 106, 48, 65, 67, 74, 55, 88, 
                                        47, 76, 50, 50, 51, 73, 85, 113, 116, 50, 103, 104, 87, 115, 102, 68, 56, 85, 90, 84, 100, 81, 116, 116, 51, 83, 49, 43, 104, 106, 109, 54, 66, 
                                        56, 110, 107, 73, 43, 98, 66, 115, 104, 43, 106, 89, 55, 89, 114, 65, 82, 10, 116, 67, 67, 121, 57, 87, 109, 72, 100, 43, 116, 108, 74, 102, 69, 85, 
                                        101, 80, 51, 75, 102, 57, 72, 119, 112, 86, 51, 56, 48, 78, 66, 110, 85, 97, 118, 56, 89, 70, 73, 107, 114, 106, 77, 81, 57, 114, 120, 116, 70, 
                                        81, 119, 101, 49, 119, 70, 68, 66, 103, 119, 47, 72, 55, 79, 89, 10, 55, 81, 73, 68, 65, 81, 65, 66, 10, 45, 45, 45, 45, 45, 69, 78, 68, 32, 80, 85, 
                                        66, 76, 73, 67, 32, 75, 69, 89, 45, 45, 45, 45, 45, 10];
                                
static BOOT_ID : &'static [u8] = &[35, 137, 227, 194, 190, 169, 155, 124, 206, 201, 32, 172, 3, 111, 113, 242, 103, 105, 209, 148, 214, 187, 197, 229, 94, 67, 96, 37, 29, 70, 185, 149];
static BOOTSTRAP_IP : &str = "10.128.0.3:30030";

#[derive(Debug)]
struct FNodeManager {
    k_closest: Arc<Mutex<Vec<Contact>>>,
    nodes_to_visit: Arc<Mutex<Vec<Contact>>>,
    visited_nodes : Arc<RwLock<HashSet<NodeID>>>,  
    timestamp: Arc<AtomicU64>,
    validator: NodeValidator,
    address: String,
}

impl FNodeManager {
    pub fn new(k_closest: Vec<Contact>, nodes_to_visit: Vec<Contact>, visited_nodes: HashSet<NodeID>,timestamp: Arc<AtomicU64> ,validator: NodeValidator, address: String) -> Self {
        Self {
            k_closest : Arc::new(Mutex::new(k_closest)),
            nodes_to_visit : Arc::new(Mutex::new(nodes_to_visit)),
            visited_nodes : Arc::new(RwLock::new(visited_nodes)),
            timestamp,
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

/* UNUSED 
    pub fn get_address(&self) -> String {
        self.address.clone()
    }
*/
    pub fn get_uid(&self) -> Vec<u8> {
        self.validator.get_nodeid().as_bytes().to_owned()
    }

    pub fn get_validator(&self) -> &NodeValidator {
        &self.validator
    }

    pub fn increment(&self) -> u64 {
        self.timestamp.fetch_add(1, SeqCst)
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

    pub fn print_rtable(&self) { 
        self.node.print_rtable();
    }
    pub fn get_uid(&self) -> NodeID {
        self.node.get_uid()
    }

/* UNUSED
    pub fn get_address(&self) -> String {
        self.node.address.clone()
    }
*/

    pub fn print_blockchain(&self) {
        self.node.print_blockchain()
    }

    pub async fn bootstrap(&self) -> Result<(), &str> {
        let boot_key = NodeID::from_vec(BOOT_ID.to_vec());
        self.node.insert(Contact::new(boot_key, BOOTSTRAP_IP.to_owned(), BOOTSTRAP_KEY.to_vec()));
        let k_closest = self.send_fnode(self.node.uid).await;
        for node in k_closest {
            self.node.insert(node);
        }
        self.node.print_rtable();
        Ok(())
    }

    pub async fn annouce_auction(&self, auct: AuctionGossip) -> Result<(), &'static str> {
        let boot_key = NodeID::from_vec(BOOT_ID.to_vec());
        let bootstrap_closest = self.send_fnode(boot_key).await;

        for con in bootstrap_closest {
            let _ = self.send_store(boot_key,auct.clone(),con).await;
        }
        Ok(())
    }

    pub async fn req_blockchain(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.node.request_chain().await
    }

    pub async fn get_avaliable_auctions(&self) -> Option<Vec<AuctionGossip>> {
        let boot_key = NodeID::from_vec(BOOT_ID.to_vec());
        self.send_fvalue(boot_key).await
    }

    pub async fn subscribe_auction(&self, gossip: AuctionGossip) { 
        let k_closest = self.send_fnode(gossip.get_seller()).await;
        
        for con in k_closest {
            let _ = self.send_store(self.get_uid(),gossip.clone(),con);
        }
    }

    pub async fn send_fnode(&self, key: NodeID) -> Vec<Contact> {
        let my_closest = self.node.lookup(key);
        let nodes_to_visit:Vec<Contact> = my_closest.iter().map(|a| *a.clone()).collect();
        let k_closest= nodes_to_visit.clone();
        let mut visited_nodes = HashSet::<NodeID>::new();
        visited_nodes.insert(self.node.uid);
        let find_nodes = FNodeManager::new(
            k_closest,nodes_to_visit,
            visited_nodes,
            self.node.get_timestamp(),
            self.node.get_validator().clone(),
            self.node.address.clone()
        );

        let info = Arc::new(find_nodes);
        let mut handles = Vec::with_capacity(PARALLEL_LOOKUPS.try_into().unwrap());
        for _i in 0..PARALLEL_LOOKUPS {
            handles.push(a_lookup(key,info.clone()));
        }
        
        join_all(handles.into_iter().map(tokio::spawn)).await;
        //println!("closest {:?}", info);
        info.get_k_closest()
    }

   pub async fn send_fvalue(&self, key: NodeID) -> Option<Vec<AuctionGossip>> {
        if let Some(maybe_value) = self.node.retrieve(key) {
            return Some(maybe_value);
        }
        
        let k_closest = self.send_fnode(key).await;
        for contact in k_closest {
            let address = format_address(contact.address.clone());
            let mut client = KademliaClient::connect(address.clone()).await.unwrap();  
            let timestamp = self.node.increment();
            let (hash,request_signature) = Signer::sign_strong_header_req(timestamp,contact.get_pubkey(),&self.node.address,key.as_bytes().to_owned());
            let request = FValueReq {
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
                            kademlia::f_value_repl::HasValue::Auction(val) =>{
                                return Some(to_gossip_vec(val.list));
                            }
                        }
                    }
                 }
                Err(_) => println!("failed to unwrap message"),    
            }
        }
        None
    }

    async fn send_store(&self,key:NodeID, value: AuctionGossip, contact: Contact) -> Result<(),&'static str> {
        let address = format_address(contact.address.clone());
        let mut client = KademliaClient::connect(address.clone()).await.unwrap();  
        let formated_value = to_auction_data(value);
        let timestamp =  self.node.increment();
        let databuf: Vec<u8> = encode_store(&formated_value,key);
        let (hash,request_signature) = Signer::sign_strong_header_req(timestamp,contact.get_pubkey(),&self.node.address,databuf);
        let request = StoreReq {
                header: Some( Header {
                    my_id: self.node.uid.as_bytes().to_owned(),
                    address : self.node.address.to_owned(),
                    pub_key: self.node.get_pubkey(),
                    nonce: self.node.get_nonce(),
                    timestamp,
                    signature: request_signature.clone() ,
                }),
                target_id: key.as_bytes().to_owned(),
                value: Some(formated_value),
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

    pub async fn broadcast_transaction(&self, data: Data) {
        let my_closest = self.node.lookup(self.get_uid());
        let timestamp = self.node.increment_broadcast();
        let data = grpc_transaction(data.clone());
    
        for contact in my_closest {
            let connection = KademliaClient::connect(format_address(contact.address)).await; 

            match connection {
                Ok(mut channel) => {
                    let broadcast_message = Request::new(BroadcastReq { 
                        timestamp, 
                        rdata:  Some(super::kademlia::broadcast_req::Rdata::Transaction(data.clone())),
                    });
                    let _ = channel.broadcast(broadcast_message);
                },
                Err(_) => continue,
            }
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
                let timestamp = info.increment();
                let (hash,request_signature) = Signer::sign_weak_header_req(timestamp,node.get_pubkey(),&info.address);
                let request = FNodeReq {
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
                         println!("node {:?} didn't respond: {} ", &node.address, err);
                     },
                 };
             }
             Err(err) =>{
                 println!("address {:?} unreachable, removing from route table: {}", &node.address,err);
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
    let timestamp =  gen_cookie();
    let (hash,request_signature) = Signer::sign_weak_header_req(timestamp,contact.get_pubkey(),my_address);
    let request = PingM {
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