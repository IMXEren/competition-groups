use anyhow::Context;
use tokio::time::timeout;
use std::process::{ Command, Stdio };
use std::{ io::BufRead, time::Duration };
use port_selector::random_free_tcp_port;

fn get_port() -> anyhow::Result<u16> {
    random_free_tcp_port().context("failed to find any open port for chromedriver")
}

pub fn stop_chromedriver() {
    let _ = Command::new("bash")
        .arg("-c")
        .arg(
            r#"kill -9 $(ps -e -o pid,args | grep "chromedriver --port" | sed -E 's/^ +//g' | sed 's/ .*//g')"#
        )
        .spawn();
}

// Either use ./chromedriver/chromedriver relative to Cargo.toml (or executable)
// or PATH env variable
pub async fn start_chromedriver() -> anyhow::Result<u16> {
    let port = get_port()?;
    let chromedriver = get_chromedriver_binary_path();
    let mut child = Command::new(chromedriver)
        .arg(format!("--port={port}"))
        .stdout(Stdio::piped())
        .spawn()
        .context("failed to start chromedriver")?;
    let stdout = child.stdout.take().unwrap();
    let mut bufread = std::io::BufReader::new(stdout);
    let mut buf = String::new();
    let dtime = Duration::from_secs(10);
    let _ = timeout(dtime, async {
        while let Ok(n) = bufread.read_line(&mut buf) {
            if n > 0 {
                if buf.contains("started successfully") {
                    return Some(0u8);
                }
                buf.clear();
            } else {
                break;
            }
        }
        None
    }).await.context("failed to start chromedriver")?;
    Ok(port)
}

pub async fn get_chromedriver_server() -> anyhow::Result<String> {
    Ok(format!("http://localhost:{}", start_chromedriver().await?))
}

fn get_chromedriver_binary_path() -> String {
    let cdir = std::env::current_dir().unwrap();
    let chromedriver = cdir.join("chromedriver/chromedriver");
    if chromedriver.exists() {
        return chromedriver.to_string_lossy().to_string();
    }
    "chromedriver".to_string()
}
