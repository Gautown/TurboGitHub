mod config;
mod dns_server;
mod ipc_server;
mod scanner;

use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting TurboGitHub Core Daemon");
    
    let config = Arc::new(load_config().await?);
    info!("Configuration loaded successfully: listen_addr={}, upstream_dns={}", 
          config.listen_addr, config.upstream_dns);
    
    let scanner = Arc::new(scanner::Scanner::new(Arc::clone(&config)));
    scanner.start().await;
    info!("IP scanner started");
    
    info!("Creating DNS server...");
    let dns_server = dns_server::DnsServer::new(
        Arc::clone(&scanner),
        config.upstream_dns.clone(),
    )?;
    info!("DNS server created successfully");
    
    info!("Creating IPC server...");
    let ipc_server = ipc_server::IpcServer::new(
        Arc::clone(&scanner),
        Arc::clone(&config),
    );
    info!("IPC server created successfully");
    
    info!("Starting servers with tokio::select...");
    tokio::select! {
        dns_result = dns_server.start(config.listen_addr.clone()) => {
            if let Err(e) = dns_result {
                error!("DNS server error: {}", e);
            } else {
                info!("DNS server exited normally");
            }
        }
        ipc_result = ipc_server.start("127.0.0.1:3030".to_string()) => {
            if let Err(e) = ipc_result {
                error!("IPC server error: {}", e);
            } else {
                info!("IPC server exited normally");
            }
        }
    }
    
    info!("TurboGitHub Core Daemon shutting down");
    Ok(())
}

async fn load_config() -> anyhow::Result<config::Config> {
    let config_path = std::env::current_dir()?
        .join("config.toml");
    
    if config_path.exists() {
        config::Config::load(config_path)
    } else {
        let default_config = config::Config::default();
        default_config.save(&config_path)?;
        info!("Created default configuration file: {:?}", config_path);
        Ok(default_config)
    }
}