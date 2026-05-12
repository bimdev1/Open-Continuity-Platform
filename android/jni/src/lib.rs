use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use liboca::OcaSession;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref SESSION: Mutex<OcaSession> = Mutex::new(OcaSession::new());
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
) {
    let input: String = env.get_string(&text).expect("Couldn't get java string!").into();
    println!("JNI: Sending to peers: {}", input);
    
    let session = SESSION.lock().unwrap();
    match session.encrypt(input.as_bytes()) {
        Ok((_ciphertext, _tag)) => {
            println!("JNI: Successfully encrypted payload for transmission");
        }
        Err(e) => println!("JNI: Encryption error: {}", e),
    }
}
