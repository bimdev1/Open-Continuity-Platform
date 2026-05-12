use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, KeyInit, aead::Aead};
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::error::Error;
use rand::rngs::OsRng;

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
    pub data: Vec<u8>, // Encrypted payload
    pub tag: [u8; 16], // AEAD tag
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClipboardPayload {
    pub text: String,
    pub timestamp: u64,
}

pub struct OcaSession {
    pub keypair: SigningKey,
    pub peer_pubkey: Option<VerifyingKey>,
    pub shared_secret: Option<[u8; 32]>,
}

impl OcaSession {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        OcaSession {
            keypair: signing_key,
            peer_pubkey: None,
            shared_secret: Some([0u8; 32]), // Dummy secret for MVP
        }
    }

    /// Simplified handshake initiation
    pub async fn initiate_handshake(&mut self, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        let pubkey_bytes = self.keypair.verifying_key().to_bytes();
        
        // 1. Send our public key
        stream.write_all(&pubkey_bytes).await?;
        
        // 2. Receive peer public key
        let mut peer_pubkey_bytes = [0u8; 32];
        stream.read_exact(&mut peer_pubkey_bytes).await?;
        let peer_verifying_key = VerifyingKey::from_bytes(&peer_pubkey_bytes)?;
        self.peer_pubkey = Some(peer_verifying_key);

        Ok(())
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<(Vec<u8>, [u8; 16]), Box<dyn Error>> {
        let secret = self.shared_secret.ok_or("No session secret")?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&secret));
        let nonce = Nonce::from_slice(&[0u8; 12]); 
        
        let mut ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        if ciphertext.len() < 16 {
            return Err("Encryption result too short".into());
        }

        let tag_start = ciphertext.len() - 16;
        let tag: [u8; 16] = ciphertext[tag_start..].try_into()?;
        ciphertext.truncate(tag_start);
        
        Ok((ciphertext, tag))
    }
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

pub async fn start_discovery() -> Result<(), Box<dyn Error>> {
    use mdns_sd::{ServiceDaemon, ServiceInfo};

    let mdns = ServiceDaemon::new()?;
    let service_type = "_oca._tcp.local.";
    let instance_name = "oca_peer";
    let host_name = "oca_host.local.";
    let port = 5005;
    
    let my_service = ServiceInfo::new(
        service_type,
        instance_name,
        host_name,
        "127.0.0.1",
        port,
        None,
    )?;
    
    mdns.register(my_service)?;
    Ok(())
}
