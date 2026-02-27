use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub running: bool,
    pub current_ips: Vec<DomainIpInfo>,
    pub stats: ServiceStats,
}

#[derive(Debug, Clone)]
pub struct DomainIpInfo {
    pub domain: String,
    pub ip: String,
    pub rtt: u64,
}

#[derive(Debug, Clone)]
pub struct ServiceStats {
    pub domains_scanned: usize,
    pub total_ips: usize,
}

pub struct DaemonClient {
    connection: Arc<Mutex<Option<(
        futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            Message,
        >,
        futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        >,
    )>>>,
}

impl DaemonClient {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn connect(&self) -> Result<(), String> {
        let mut conn = self.connection.lock().await;
        
        if conn.is_some() {
            return Ok(());
        }
        
        match connect_async("ws://127.0.0.1:3030").await {
            Ok((ws_stream, _)) => {
                let (write, read) = ws_stream.split();
                *conn = Some((write, read));
                Ok(())
            }
            Err(e) => Err(format!("Failed to connect to daemon: {}", e)),
        }
    }

    // 断开连接方法（暂时未使用）
    // pub async fn disconnect(&self) {
    //     let mut conn = self.connection.lock().await;
    //     *conn = None;
    // }

    pub async fn is_connected(&self) -> bool {
        let conn = self.connection.lock().await;
        conn.is_some()
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, String> {
        let mut conn = self.connection.lock().await;
        
        let (write, read) = conn.as_mut().ok_or("Not connected to daemon")?;
        
        let id = 1;
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });
        
        if let Err(e) = write.send(Message::Text(request.to_string())).await {
            return Err(format!("Failed to send request: {}", e));
        }
        
        while let Some(msg) = read.next().await {
            let msg = msg.map_err(|e| format!("WebSocket error: {}", e))?;
            
            if msg.is_text() {
                let resp: Value = serde_json::from_str(msg.to_text().unwrap())
                    .map_err(|e| format!("Invalid JSON response: {}", e))?;
                
                if resp["id"] == id {
                    return Ok(resp["result"].clone());
                }
            }
        }
        
        Err("No response received".into())
    }

    pub async fn start_service(&self) -> Result<bool, String> {
        let result = self.call("start", json!({})).await?;
        Ok(result["success"].as_bool().unwrap_or(false))
    }

    pub async fn stop_service(&self) -> Result<bool, String> {
        let result = self.call("stop", json!({})).await?;
        Ok(result["success"].as_bool().unwrap_or(false))
    }

    pub async fn get_status(&self) -> Result<ServiceStatus, String> {
        let result = self.call("get_status", json!({})).await?;
        
        let running = result["running"].as_bool().unwrap_or(false);
        
        let current_ips = if let Some(ips_array) = result["current_ips"].as_array() {
            ips_array.iter().map(|ip_info| {
                DomainIpInfo {
                    domain: ip_info["domain"].as_str().unwrap_or("unknown").to_string(),
                    ip: ip_info["ip"].as_str().unwrap_or("0.0.0.0").to_string(),
                    rtt: ip_info["rtt"].as_u64().unwrap_or(0),
                }
            }).collect()
        } else {
            Vec::new()
        };
        
        let stats = ServiceStats {
            domains_scanned: result["stats"]["domains_scanned"].as_u64().unwrap_or(0) as usize,
            total_ips: result["stats"]["total_ips"].as_u64().unwrap_or(0) as usize,
        };
        
        Ok(ServiceStatus {
            running,
            current_ips,
            stats,
        })
    }

    // 获取配置方法（暂时未使用）
    // pub async fn get_config(&self) -> Result<Value, String> {
    //     self.call("get_config", json!({})).await
    // }

    pub async fn scan_now(&self) -> Result<bool, String> {
        // 这里可以添加一个 scan_now 方法到 IPC 服务器
        // 暂时返回成功
        Ok(true)
    }
}