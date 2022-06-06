use std::io::Write;
use std::sync::Arc;
mod p2p;
use auctions::peer::AuctionPeer;
use p2p::{
  kad::KadNode,
  server
};
use std::env;
use tokio::task;
mod auctions;
mod ledger;



fn read_terminal() -> String {
  let mut line = String::new();
  std::io::stdout().flush().unwrap();
  std::io::stdin().read_line(&mut line).expect("Error: Could not read a line");

  line.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let address: String = args[1].split('\n').collect();    
    let is_bootstrap: String = args[2].split('\n').collect();
    let node = Arc::new(KadNode::new(address.clone()));
    let svnode = node.clone();
    let mut auctpeer = AuctionPeer::new(node.clone());
    let addr = address.parse().unwrap();
    task::spawn(async move { 
      server::server(addr, svnode).await 
    });

    loop {
      println!("Insert command");
      let raw_command = read_terminal();
      let command: Vec<&str> = raw_command.split(' ').collect();
      match command[0] {
        "bootstrap" => {
          assert_eq!(command.len(),1);
          if &is_bootstrap == "bootstrap"{
            println!("Currently on bootstrap node");
          } else {
            let _ = auctpeer.client.bootstrap().await;
          }
        },
        "new_auction" => {
          assert_eq!(command.len(),4);
          let title = command[1].to_string();
          let initial_price = command[2].parse::<f32>().unwrap();
          let duration = command[3].parse::<i64>().unwrap();
          auctpeer.new_auction(title,duration,initial_price).await;
        },
        "search_auctions" => {
          assert_eq!(command.len(),1);
          let _ = auctpeer.get_avaliable_auctions().await;
        },
        "bid" => {
          assert_eq!(command.len(),3);
          let index = command[1].parse::<i32>().unwrap();
          let bid = command[2].parse::<f32>().unwrap();
          let _ = auctpeer.bid_auction(index, bid).await;
        },
        "transaction" => {
          assert_eq!(command.len(),2);
         let index = command[1].parse::<i32>().unwrap();
         auctpeer.fulfill_transaction(index).await;
        },
        "print_blockchain" => {
          assert_eq!(command.len(),1);
            auctpeer.client.print_blockchain()
        }
        "get_blockchain" => {
          assert_eq!(command.len(),1);
          auctpeer.client.req_blockchain().await?
        }
        "update_subscribed" => {
          assert_eq!(command.len(),1);
          let _ = auctpeer.update_subscribed().await;
        },

        "exit" => {
          return Ok(());
        }

        _ => println!("Invalid Command"),
      }
    }
}
