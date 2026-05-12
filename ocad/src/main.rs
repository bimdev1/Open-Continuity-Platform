use zbus::{interface, connection, SignalContext};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::net::TcpStream;
use liboca::{OcaSession, send_oca_message};

struct OcaDaemon {
    session: Arc<Mutex<OcaSession>>,
    peers: Arc<Mutex<HashMap<String, TcpStream>>>,
}

#[interface(name = "org.oca.ocad")]
impl OcaDaemon {
    async fn send_clipboard(&self, text: String) -> zbus::fdo::Result<()> {
        println!("ocad: Sending clipboard data: {}", text);
        
        // 1. Encrypt
        let session = self.session.lock().await;
        let msg = session.encrypt(text.as_bytes())
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
            
        // 2. Send to all peers
        let mut peers = self.peers.lock().await;
        for (_, stream) in peers.iter_mut() {
            send_oca_message(stream, &msg).await
                .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        }
        
        Ok(())
    }

    #[zbus(signal)]
    async fn clipboard_received(ctxt: &SignalContext<'_>, text: String) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let daemon = OcaDaemon {
        session: Arc::new(Mutex::new(OcaSession::new())),
        peers: Arc::new(Mutex::new(HashMap::new())),
    };

    let _conn = connection::Builder::session()?
        .name("org.oca.ocad")?
        .serve_at("/org/oca/ocad", daemon)?
        .build()
        .await?;

    println!("ocad: D-Bus service started on org.oca.ocad at /org/oca/ocad");
    
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
