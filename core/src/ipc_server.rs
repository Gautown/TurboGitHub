use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use crate::scanner::Scanner;

pub struct IpcServer {
    scanner: Arc<Scanner>,
    config: Arc<crate::config::Config>,
}

impl IpcServer {
    pub fn new(scanner: Arc<Scanner>, config: Arc<crate::config::Config>) -> Self {
        Self { scanner, config }
    }

    pub async fn start(&self, listen_addr: String) -> anyhow::Result<()> {
        let socket: SocketAddr = listen_addr.parse()?;
        let listener = TcpListener::bind(socket).await?;
        
        info!("IPC server listening on {}", socket);
        
        while let Ok((stream, _)) = listener.accept().await {
            let scanner = Arc::clone(&self.scanner);
            let config = Arc::clone(&self.config);
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, scanner, config).await {
                    error!("Connection error: {}", e);
                }
            });
        }
        
        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        scanner: Arc<Scanner>,
        config: Arc<crate::config::Config>,
    ) -> anyhow::Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        
        info!("New IPC connection established");
        
        while let Some(message) = read.next().await {
            let message = message?;
            
            if message.is_text() {
                let text = message.to_text()?;
                debug!("Received message: {}", text);
                
                match Self::handle_message(text, &scanner, &config).await {
                    Ok(response) => {
                        write.send(Message::Text(response)).await?;
                    }
                    Err(e) => {
                        let error_response = json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32603,
                                "message": e.to_string()
                            },
                            "id": null
                        });
                        write.send(Message::Text(error_response.to_string())).await?;
                    }
                }
            }
        }
        
        info!("IPC connection closed");
        Ok(())
    }

    async fn handle_message(
        message: &str,
        scanner: &Scanner,
        config: &crate::config::Config,
    ) -> anyhow::Result<String> {
        let request: Value = serde_json::from_str(message)?;
        
        let id = request["id"].clone();
        let method = request["method"].as_str().unwrap_or("");
        let params = request["params"].clone();
        
        let result = match method {
            "start" => Self::handle_start(params).await?,
            "stop" => Self::handle_stop(params).await?,
            "get_status" => Self::handle_get_status(params, scanner).await?,
            "get_config" => Self::handle_get_config(params, config).await?,
            "set_config" => Self::handle_set_config(params, config).await?,
            "get_logs" => Self::handle_get_logs(params).await?,
            _ => return Err(anyhow::anyhow!("Unknown method: {}", method)),
        };
        
        let response = json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        });
        
        Ok(response.to_string())
    }

    async fn handle_start(_params: Value) -> anyhow::Result<Value> {
        info!("Received start command");
        Ok(json!({ "success": true }))
    }

    async fn handle_stop(_params: Value) -> anyhow::Result<Value> {
        info!("Received stop command");
        Ok(json!({ "success": true }))
    }

    async fn handle_get_status(_params: Value, scanner: &Scanner) -> anyhow::Result<Value> {
        let ip_pool = scanner.get_ip_pool().await;
        let mut current_ips = Vec::new();
        
        for (domain, ips) in &ip_pool {
            if let Some(best_ip) = ips.iter().find(|ip| ip.reachable && ip.https_available) {
                current_ips.push(json!({
                    "domain": domain,
                    "ip": best_ip.ip.to_string(),
                    "rtt": best_ip.rtt.as_millis()
                }));
            }
        }
        
        Ok(json!({
            "running": true,
            "current_ips": current_ips,
            "stats": {
                "domains_scanned": ip_pool.len(),
                "total_ips": ip_pool.values().map(|v| v.len()).sum::<usize>()
            }
        }))
    }

    async fn handle_get_config(_params: Value, config: &crate::config::Config) -> anyhow::Result<Value> {
        Ok(json!({
            "domains": config.domains,
            "scan_interval": config.scan_interval,
            "scan_concurrency": config.scan_concurrency,
            "upstream_dns": config.upstream_dns,
            "listen_addr": config.listen_addr,
            "log_level": config.log_level
        }))
    }

    async fn handle_set_config(_params: Value, _config: &crate::config::Config) -> anyhow::Result<Value> {
        warn!("Set config not implemented yet");
        Ok(json!({ "success": false, "message": "Not implemented" }))
    }

    async fn handle_get_logs(params: Value) -> anyhow::Result<Value> {
        let lines = params["lines"].as_u64().unwrap_or(100);
        warn!("Get logs not implemented yet, requested {} lines", lines);
        Ok(json!({ "logs": [] }))
    }
}