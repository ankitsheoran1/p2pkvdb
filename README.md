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

