# Distributed Auction Platform
Project for Systems and Data Security Course (CC4030) (2021/2022)

Very basic cli auction system that runs on top of a Kademlia DHT and implements a PoW Blockchain for recording transactions.

# How to run
On the root directory:

Bootstrap node: 

```
cargo run --bin network <ip:port> bootstrap
```

Another node: 
```
cargo run --bin network <ip:port> 
```

Possible command(direct rpcs are not possible because of Key parsing): 

    "bootstrap"  => Bootstraps node into network.

    "new_auction" => creates a new auction with this node.

    "search_auctions" => searches avaliable auctions.

    "bid" => places a bid on an auction.

    "transaction" => *Experimental* concludes an auction and generates a block.

    "print_blockchain" => Prints the nodes blockchain.

    "get_blockchain" =>  gets a chain from the network.

    "update_subscribed" => updates node on subscribed auctions.

    "exit" => shuts down node.
