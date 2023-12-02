use std::collections::HashMap;
use tokio::{net::{TcpListener, TcpStream, UdpSocket}, sync::RwLock};
use std::net::SocketAddr;
use std::sync::Arc;
use mac_address::MacAddressError;
use serde::{Deserialize, Serialize};
use tokio::io::{ AsyncReadExt, AsyncWriteExt };

const BROAD_CAST_ADDRESS: &str = "255.255.255.255:8888";
const TCP_PORT: u16 = 9000;

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    HandShake { node_name: String, tcp_addr: SocketAddr },
    Greeting,
    HeartBeat, 
    HeartBeatResponse,
    HeartBeatResp,
    SetValue { key: String, value: String },
    GetValue { key: String },
    ValueResponse { value: Option<String> },
    Sync { key: String, value: String },
}

struct NodeInfo {
    last_seen: std::time::Instant,
    tcp_addr: SocketAddr,
}

struct KeyValueStore {
    store: RwLock<HashMap<String, String>>,
}

impl KeyValueStore {
    fn new() -> Self {
        KeyValueStore { store:  RwLock::new(HashMap::new()) }
    }

    async fn set(&self, key: String, value: String) {
        let mut store = self.store.write().await;
        store.insert(key, value);
    }

    async fn get(&self, key: &str) ->  Option<String> {
        let store = self.store.read().await;
        store.get(key).cloned()
    }
}

fn get_mac_addr() -> Result<String, MacAddressError> {
    let mac = mac_address::get_mac_address()?;
    match mac {
        Some(address) => Ok(address.to_string()),
        None => Err(MacAddressError::InternalError),
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   let local_addr: SocketAddr = "0.0.0.0:8888".parse()?;
   let socket = UdpSocket::bind(&local_addr).await?;
   socket.set_broadcast(true)?;

   // Intialize key value store
   let store = Arc::new(KeyValueStore::new());

   // nodes for storing all nodes who are alive and in this node knowledge
   let nodes = Arc::new(RwLock::new(HashMap::<String, NodeInfo>::new()));
   
   let socket = Arc::new(socket);
   let socket_for_broadcast = socket.clone();

   tokio::spawn(async move {
    match get_mac_addr() {
        Ok(node_name) => {
            let tcp_addr = format!("{}:{}", "0.0.0.0", TCP_PORT).parse().unwrap();
            let msg = Message::HandShake {
                node_name,
                tcp_addr,
            };

            let serialized_msg = serde_json::to_string(&msg).unwrap();

            loop {
                println!("Sending UDP broadcast...");
                socket_for_broadcast.send_to(serialized_msg.as_bytes(), BROAD_CAST_ADDRESS).await.unwrap();
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
        Err(e) => {
            eprintln!("Error fetching MAC address: {:?}", e);
        }

    }
   });

   let nodes_clone = nodes.clone();
   
   tokio::spawn(async move {
    let listener = TcpListener::bind(("0.0.0.0", TCP_PORT)).await.unwrap();
    println!("TCP listener started.");
    while let Ok((stream, _)) = listener.accept().await {
       println!("Accepted new TCP connection.");
       tokio::spawn(handle_tcp_stream(stream, nodes_clone.clone(), store.clone()));
    }

   });

   let mut buf = vec![0u8; 1024];
   loop {
     let (len , addr) = socket.recv_from(&mut buf).await?;
     println!("Received data on UDP from {}", addr);
     let received_msg: Message = serde_json::from_slice(&buf[..len])?;

     let local_node_name = get_mac_addr()?;
     if let Message::HandShake { node_name, tcp_addr } = received_msg {

        if local_node_name == node_name {
            continue;
        }
        println!("Received handshake from: {}", node_name);
        {
            let mut nodes_guard = nodes.write().await;
            nodes_guard.insert(node_name, NodeInfo{ tcp_addr, last_seen: std::time::Instant::now() });
        }

        let greeting = Message::Greeting;
        let serialized_greeting = serde_json::to_string(&greeting).unwrap();
        let _ = socket.send_to(serialized_greeting.as_bytes(), &addr).await;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                println!("Sending heartbeat to {}", tcp_addr);
                let mut stream = TcpStream::connect(tcp_addr).await.unwrap();
                let heartbeat_msg = Message::HeartBeat;
                let serialized_msg = serde_json::to_string(&heartbeat_msg).unwrap();
                stream.write_all(serialized_msg.as_bytes()).await.unwrap();
            }
        });
    }
 }
}

async fn handle_tcp_stream(mut stream: TcpStream, nodes: Arc<RwLock<HashMap<String, NodeInfo>>>, store: Arc<KeyValueStore>) {
    let mut buf = vec![0u8; 1024];
    let len = stream.read(&mut buf).await.unwrap();
    let received_msg: Message = serde_json::from_slice(&buf[..len]).unwrap();
    match received_msg {
        Message::HeartBeat => {
            println!("Received Heartbeat");
            let response = Message::HeartBeatResponse;
            let serialized_response = serde_json::to_string(&response).unwrap();
            stream.write_all(serialized_response.as_bytes()).await.unwrap();
        }
        Message::SetValue { key, value } => {
            println!("Received SetValue");
            store.set(key.clone(), value.clone()).await;
            let nodes_guard = nodes.read().await;
            for (_, node_info) in nodes_guard.iter() {
                let mut stream = match TcpStream::connect(node_info.tcp_addr).await {
                    Ok(stream) => stream,
                    Err(_) => continue,
                };
                let sync_msg = Message::Sync { key: key.clone(), value: value.clone() };
                let serialized_msg = serde_json::to_string(&sync_msg).unwrap();
                let _ = stream.write_all(serialized_msg.as_bytes()).await;
            }
            let response = Message::ValueResponse { value: Some("Value set successfully.".to_string()) };
            let serialized_response = serde_json::to_string(&response).unwrap();
            stream.write_all(serialized_response.as_bytes()).await.unwrap();
        }
        Message::GetValue { key } => {
            println!("Received GetValue");
            let value = store.get(&key).await;
            let response = Message::ValueResponse { value };
            let serialized_response = serde_json::to_string(&response).unwrap();
            stream.write_all(serialized_response.as_bytes()).await.unwrap();
        }
        Message::Sync { key, value } => {
            println!("Received Sync");
            store.set(key, value).await;
        }
        _ => {}

        
    }

}