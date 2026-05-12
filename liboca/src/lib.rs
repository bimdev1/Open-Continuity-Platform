use ed25519_dalek::{SigningKey, VerifyingKey, Signer};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, KeyInit, aead::Aead, aead::OsRng as AeadRng};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::error::Error;
use rand::rngs::OsRng;
use rand::RngCore;

pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub enum PayloadType {
    HandshakeInit,
    HandshakeAck,
    ClipboardText,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OcaMessage {
    pub version: u8,
    pub payload_type: PayloadType,
    pub nonce: [u8; 12],
    pub data: Vec<u8>, 
    pub tag: [u8; 16],
}

pub struct OcaSession {
    pub keypair: SigningKey,
    pub shared_secret: Option<[u8; 32]>,
}

impl OcaSession {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        OcaSession {
            keypair: signing_key,
            shared_secret: None,
        }
    }

    pub async fn initiate_handshake(&mut self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let mut csprng = OsRng;
        let secret = EphemeralSecret::random_from_rng(&mut csprng);
        let public = X25519PublicKey::from(&secret);
        
        // 1. Send X25519 public key
        stream.write_all(public.as_bytes()).await?;
        
        // 2. Receive peer public key
        let mut peer_pubkey_bytes = [0u8; 32];
        stream.read_exact(&mut peer_pubkey_bytes).await?;
        let peer_public = X25519PublicKey::from(peer_pubkey_bytes);

        // 3. Derive shared secret
        let shared_secret = secret.diffie_hellman(&peer_public);
        self.shared_secret = Some(*shared_secret.raw_secret_bytes());

        Ok(())
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<OcaMessage, Box<dyn Error>> {
        let secret = self.shared_secret.ok_or("No session secret")?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&secret));
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let mut ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        let tag_start = ciphertext.len() - 16;
        let tag: [u8; 16] = ciphertext[tag_start..].try_into()?; // Extract the tag
        ciphertext.truncate(tag_start); // Remove tag from data buffer
        
        Ok(OcaMessage {
            version: PROTOCOL_VERSION,
            payload_type: PayloadType::ClipboardText,
            nonce: nonce_bytes,
            data: ciphertext,
            tag,
        })
    }

    pub fn decrypt(&self, msg: &OcaMessage) -> Result<Vec<u8>, Box<dyn Error>> {
        let secret = self.shared_secret.ok_or("No session secret")?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&secret));
        let nonce = Nonce::from_slice(&msg.nonce);

        // Combine data and tag
        let mut cipher_text_with_tag = msg.data.clone();
        cipher_text_with_tag.extend_from_slice(&msg.tag);

        let plaintext = cipher.decrypt(nonce, cipher_text_with_tag.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        Ok(plaintext)
    }
}

pub async fn send_oca_message(stream: &mut TcpStream, msg: &OcaMessage) -> Result<(), Box<dyn Error>> {
    let serialized = bincode::serialize(msg)?;
    let len = serialized.len() as u32;
    stream.write_u32(len).await?;
    stream.write_all(&serialized).await?;
    Ok(())
}

pub async fn receive_oca_message(stream: &mut TcpStream) -> Result<OcaMessage, Box<dyn Error>> {
    let len = stream.read_u32().await? as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    let msg: OcaMessage = bincode::deserialize(&buf)?;
    Ok(msg)
}

pub async fn start_listener(port: u16) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("liboca: Listening on port {}", port);
    
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            println!("liboca: New connection from {}", socket.peer_addr().unwrap());
        });
    }
}

pub async fn start_discovery() -> Result<tokio::sync::mpsc::Receiver<(String, String, u16)>, Box<dyn Error>> {
    use mdns_sd::{ServiceDaemon, ServiceInfo};

    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let mdns = ServiceDaemon::new()?;
    let service_type = "_oca._tcp.local.";
    let receiver = mdns.browse(service_type)?;

    tokio::spawn(async move {
        while let Ok(event) = receiver.recv() {
            match event {
                mdns_sd::ServiceEvent::ServiceResolved(info) => {
                    let addr = info.get_addresses().iter().next().unwrap().to_string();
                    let port = info.get_port();
                    let _ = tx.send((info.get_fullname().to_string(), addr, port)).await;
                }
                _ => {}
            }
        }
    });

    // Still perform registration (announce self)
    let instance_name = "oca_peer";
    let host_name = "oca_host.local.";
    let port = 5005;
    let my_service = ServiceInfo::new(service_type, instance_name, host_name, "127.0.0.1", port, None)?;
    mdns.register(my_service)?;

    Ok(rx)
}
