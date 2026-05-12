use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject, JGlobalRef};
use liboca::{OcaSession, send_oca_message};
use std::sync::Mutex;
use lazy_static::lazy_static;
use tokio::runtime::{Runtime, Builder};
use tokio::net::TcpStream;
use std::collections::HashMap;

struct PeerManager {
    peers: HashMap<String, TcpStream>,
    sessions: HashMap<String, OcaSession>,
}

lazy_static! {
    static ref PEER_MANAGER: Mutex<PeerManager> = Mutex::new(PeerManager {
        peers: HashMap::new(),
        sessions: HashMap::new(),
    });
    static ref RUNTIME: Runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
}

#[no_mangle]
pub extern "system" fn Java_dev_oca_OcaService_initRustCore(
    mut env: JNIEnv,
    _class: JClass,
    service: JObject,
) {
    let service_ref = env.new_global_ref(service).unwrap();
    let jvm = env.get_java_vm().unwrap();

    RUNTIME.spawn(async move {
        // 1. Listen for incoming connections
        let listener = liboca::start_listener(5005).await.expect("Failed to start listener");
        let jvm_clone = jvm.clone();
        let service_ref_clone = service_ref.clone();

        tokio::spawn(async move {
            while let Ok((mut stream, addr)) = listener.accept().await {
                let peer_addr = addr.to_string();
                let mut session = OcaSession::new();
                if session.accept_handshake(&mut stream).await.is_ok() {
                    let mut manager = PEER_MANAGER.lock().unwrap();
                    manager.sessions.insert(peer_addr.clone(), session);
                    manager.peers.insert(peer_addr.clone(), stream.try_clone().unwrap());
                    
                    // Start receiver loop
                    let inner_jvm = jvm_clone.clone();
                    let inner_service_ref = service_ref_clone.clone();
                    let peer_addr_inner = peer_addr.clone();
                    tokio::spawn(async move {
                        let mut stream_read = stream;
                        loop {
                            if let Ok(msg) = liboca::receive_oca_message(&mut stream_read).await {
                                let manager = PEER_MANAGER.lock().unwrap();
                                if let Some(session) = manager.sessions.get(&peer_addr_inner) {
                                    if let Ok(plaintext) = session.decrypt(&msg) {
                                        let text = String::from_utf8_lossy(&plaintext).to_string();
                                        // Callback to Kotlin
                                        let mut env = inner_jvm.attach_current_thread().unwrap();
                                        let j_text = env.new_string(text).unwrap();
                                        let _ = env.call_method(&inner_service_ref, "onMessageReceived", "(Ljava/lang/String;)V", &[(&j_text).into()]);
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                    });
                }
            }
        });

        // 2. Discover other peers
        let mut discovery_rx = liboca::start_discovery().await.expect("Failed to start discovery");
        while let Some((name, addr, port)) = discovery_rx.recv().await {
            let full_addr = format!("{}:{}", addr, port);
            let mut manager = PEER_MANAGER.lock().unwrap();
            if !manager.peers.contains_key(&full_addr) {
                drop(manager);
                if let Ok(mut stream) = TcpStream::connect(&full_addr).await {
                    let mut session = OcaSession::new();
                    if session.initiate_handshake(&mut stream).await.is_ok() {
                        let mut manager = PEER_MANAGER.lock().unwrap();
                        manager.sessions.insert(full_addr.clone(), session);
                        manager.peers.insert(full_addr.clone(), stream.try_clone().unwrap());
                        
                        // Receiver loop (same as above)
                        let inner_jvm = jvm.clone();
                        let inner_service_ref = service_ref.clone();
                        let full_addr_inner = full_addr.clone();
                        tokio::spawn(async move {
                            let mut stream_read = stream;
                            loop {
                                if let Ok(msg) = liboca::receive_oca_message(&mut stream_read).await {
                                    let manager = PEER_MANAGER.lock().unwrap();
                                    if let Some(session) = manager.sessions.get(&full_addr_inner) {
                                        if let Ok(plaintext) = session.decrypt(&msg) {
                                            let text = String::from_utf8_lossy(&plaintext).to_string();
                                            let mut env = inner_jvm.attach_current_thread().unwrap();
                                            let j_text = env.new_string(text).unwrap();
                                            let _ = env.call_method(&inner_service_ref, "onMessageReceived", "(Ljava/lang/String;)V", &[(&j_text).into()]);
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                        });
                    }
                }
            }
        }
    });
}

#[no_mangle]
pub extern "system" fn Java_dev_oca_OcaService_sendToPeers(
    mut env: JNIEnv,
    _class: JClass,
    text: JString,
    peer_addr: JString,
) {
    let input: String = env.get_string(&text).expect("Couldn't get java string!").into();
    let addr: String = env.get_string(&peer_addr).expect("Couldn't get addr!").into();
    
    let mut manager = PEER_MANAGER.lock().unwrap();
    
    if let Some(session) = manager.sessions.get(&addr) {
        if let Ok(msg) = session.encrypt(input.as_bytes()) {
            if let Some(stream) = manager.peers.get_mut(&addr) {
                 let mut stream_clone = stream.try_clone().unwrap();
                 tokio::spawn(async move {
                    let _ = send_oca_message(&mut stream_clone, &msg).await;
                 });
            }
        }
    }
}
