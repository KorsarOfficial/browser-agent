use std::path::PathBuf;
use std::time::Duration;

use chromiumoxide::Browser;
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

use crate::err::Error;

type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub const PORT: u16 = 9222;
const PROFILE: &str = r"C:\Users\Артур\.browser-agent-profile";

fn find_exe() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("BROWSER_PATH") {
        let pb = PathBuf::from(p);
        if pb.exists() {
            return Ok(pb);
        }
    }
    let candidates = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ];
    for c in candidates {
        let pb = PathBuf::from(c);
        if pb.exists() {
            return Ok(pb);
        }
    }
    Err(Error::Chrome("msedge not found".into()))
}

#[inline(always)]
async fn port_alive(p: u16) -> bool {
    TcpStream::connect(format!("127.0.0.1:{p}")).await.is_ok()
}

fn launch(p: u16) -> Result<Option<std::process::Child>> {
    if std::net::TcpStream::connect(format!("127.0.0.1:{p}")).is_ok() {
        return Ok(None);
    }
    let e = find_exe()?;
    let c = std::process::Command::new(e)
        .arg(format!("--remote-debugging-port={p}"))
        .arg(format!("--user-data-dir={PROFILE}"))
        .arg("--window-size=1280,720")
        .spawn()
        .map_err(Error::Launch)?;
    Ok(Some(c))
}

#[inline(always)]
async fn poll_ready(p: u16) -> Result<()> {
    const ATTEMPTS: u8 = 20;
    for i in 0..ATTEMPTS {
        tracing::debug!("cdp poll {i}/{ATTEMPTS} port={p}");
        if port_alive(p).await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Err(Error::Timeout { port: p, attempts: ATTEMPTS })
}

pub async fn init(p: u16) -> Result<(Browser, JoinHandle<()>)> {
    let _c = launch(p)?;
    poll_ready(p).await?;
    let (b, h) = Browser::connect(format!("http://127.0.0.1:{p}"))
        .await
        .map_err(|e| Error::Chrome(e.to_string()))?;
    let jh = tokio::spawn(async move { h.for_each(|_| async {}).await; });
    tracing::info!("cdp connected port={p}");
    Ok((b, jh))
}
