use zbus::{interface, connection, SignalContext};
use tokio::sync::Mutex;
use std::sync::Arc;
use liboca::OcaSession;

struct OcaDaemon {
    session: Arc<Mutex<OcaSession>>,
}

#[interface(name = "org.oca.ocad")]
impl OcaDaemon {
    /// Send clipboard text to peers
    async fn send_clipboard(&self, text: String) -> zbus::fdo::Result<()> {
        println!("ocad: Sending clipboard data: {}", text);
        // Bridge to liboca networking logic here
        Ok(())
    }

    /// Signal emitted when clipboard text is received from a peer
    #[zbus(signal)]
    async fn clipboard_received(ctxt: &SignalContext<'_>, text: String) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let daemon = OcaDaemon {
        session: Arc::new(Mutex::new(OcaSession::new())),
    };

    let _conn = connection::Builder::session()?
        .name("org.oca.ocad")?
        .serve_at("/org/oca/ocad", daemon)?
        .build()
        .await?;

    println!("ocad: D-Bus service started on org.oca.ocad at /org/oca/ocad");
    
    // Keep the daemon running
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
