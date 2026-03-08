use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[path = "../app_config.rs"]
mod app_config;

#[path = "../evaluation.rs"]
mod evaluation;

#[path = "../collection.rs"]
mod collection;

fn server_addr() -> SocketAddr {
    let port = env::var("CLAUDE_CODE_LAUNCH_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|port| *port > 0)
        .unwrap_or(8787);
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = server_addr();
    let app_cfg = app_config::load_app_config().map_err(|error| {
        eprintln!("{error}");
        error
    })?;
    let db_path = app_cfg.db_path;
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let abs_db_path = {
        let p = std::path::Path::new(&db_path);
        if p.is_absolute() {
            db_path.clone()
        } else {
            env::current_dir()
                .map(|cwd| cwd.join(p).display().to_string())
                .unwrap_or_else(|_| db_path.clone())
        }
    };
    println!("sqlite db path: {abs_db_path}");
    println!("local api listening on http://{addr}");
    collection::serve(addr, db_path).await?;
    Ok(())
}
