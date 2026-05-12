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

    RUNTIME.spawn(async move {
        // Here we would implement the listener loop and handshake
        // When message received, invoke:
        // env.call_method(&service_ref, "onMessageReceived", "(Ljava/lang/String;)V", &[data.into()])
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
