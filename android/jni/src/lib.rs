use jni::JNIEnv;
use jni::objects::{JClass, JString};
use liboca::{OcaSession, send_oca_message};
use std::sync::Mutex;
use lazy_static::lazy_static;
use tokio::runtime::{Runtime, Builder};
use tokio::net::TcpStream;

lazy_static! {
    static ref SESSION: Mutex<OcaSession> = Mutex::new(OcaSession::new());
    static ref RUNTIME: Runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
}

#[no_mangle]
pub extern "system" fn Java_dev_oca_OcaService_initRustCore(
    _env: JNIEnv,
    _class: JClass,
) {
    // Start mDNS discovery or listener
    println!("JNI: Initializing Rust Core");
    // In a real app, you'd spawn a tokio runtime here
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
    
    println!("JNI: Sending to {}: {}", addr, input);
    
    let session = SESSION.lock().unwrap();
    match session.encrypt(input.as_bytes()) {
        Ok(msg) => {
            RUNTIME.spawn(async move {
                if let Ok(mut stream) = TcpStream::connect(addr).await {
                    let _ = send_oca_message(&mut stream, &msg).await;
                }
            });
        }
        Err(e) => println!("JNI: Encryption error: {}", e),
    }
}
