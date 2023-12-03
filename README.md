### p2pkvdb

A simple powerful peer to peer key value database implemented in rust. This project also includes a CLI (Command Line Interface) that enables users to interact with the database effortlessly. Explore the power of distributed systems, emphasising the performance and reliability that Rust brings to the table.

### Features 
1. You can set a value to any one up node , it would be synced to oll other peers.
2. Every node have all data , its like each node is master of all data 
3. Currently It will only support In-memory data save so in case of db crash data may lost 
4. All nodes would be eventually consistent 


### Process 
1. Start a UDP listener and it would also discover all peers by broadcasting handshake to all peers 
2. Start a TCP listener , It would serve all functionality set data , get data , synchronize data with all other nodes discovered by udp broadcasting 
3. Support of CLI for communication

### How to Use

Clone repository:

git clone git@github.com:ankitsheoran1/p2pkvdb.git

cd p2pkvdb
cargo run 

cd cliInterface
cargo build --release
./target/release/cliInterface set   --node 0.0.0.0:9000 <key> <value>
./target/release/cliInterface set   --node 0.0.0.0:9000 <key>


## Need To Do - 
Currently its only supporting String type key and value , need to make generic
Currently its only in-memory , need to implement storage 
Need to explore how actaully broadcast a UDP to all nodes ina network or there any better way 

