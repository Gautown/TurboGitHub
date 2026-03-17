use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, info, warn};

/// IPC客户端错误类型
#[derive(Debug, thiserror::Error)]
pub enum IpcClientError {
    #[error("连接失败: {0}")]
    ConnectionError(String),
    #[error("RPC调用失败: {0}")]
    RpcError(String),
    #[error("超时: {0}")]
    Timeout(String),
    #[error("JSON解析错误: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("WebSocket错误: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
}

/// IPC客户端
pub struct IpcClient {
    write: Option<futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>,
    read: Option<futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>,
    connected: bool,
    server_url: String,
    connection_attempts: u32,
    max_retries: u32,
}

impl IpcClient {
    /// 创建新的IPC客户端
    pub fn new(server_url: String) -> Self {
        Self {
            write: None,
            read: None,
            connected: false,
            server_url,
            connection_attempts: 0,
            max_retries: 3,
        }
    }
    
    /// 创建新的IPC客户端，可自定义重试次数
    #[allow(dead_code)]
    pub fn with_retries(server_url: String, max_retries: u32) -> Self {
        Self {
            write: None,
            read: None,
            connected: false,
            server_url,
            connection_attempts: 0,
            max_retries,
        }
    }

    /// 连接到IPC服务器
    pub async fn connect(&mut self) -> Result<(), IpcClientError> {
        self.connection_attempts = 0;
        
        while self.connection_attempts < self.max_retries {
            self.connection_attempts += 1;
            info!("尝试连接到IPC服务器 (尝试 {}): {}", self.connection_attempts, self.server_url);
            
            let url = format!("ws://{}", self.server_url);
            match connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    let (write, read) = ws_stream.split();
                    self.write = Some(write);
                    self.read = Some(read);
                    self.connected = true;
                    self.connection_attempts = 0; // 重置尝试次数
                    
                    info!("IPC连接已建立");
                    return Ok(());
                }
                Err(e) => {
                    warn!("连接尝试 {} 失败: {}", self.connection_attempts, e);
                    
                    if self.connection_attempts < self.max_retries {
                        let delay = std::time::Duration::from_secs(2u64.pow(self.connection_attempts - 1));
                        info!("等待 {} 秒后重试...", delay.as_secs());
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(IpcClientError::ConnectionError(format!(
            "连接失败，已达到最大重试次数: {}", 
            self.max_retries
        )))
    }

    /// 断开连接
    pub async fn disconnect(&mut self) -> Result<(), IpcClientError> {
        if let Some(mut write) = self.write.take() {
            write.close().await?;
        }
        self.read = None;
        self.connected = false;
        info!("IPC连接已断开");
        Ok(())
    }

    /// 检查连接状态
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// 发送心跳检测
    #[allow(dead_code)]
    pub async fn send_heartbeat(&mut self) -> Result<bool, IpcClientError> {
        if !self.connected {
            return Ok(false);
        }
        
        // 发送一个简单的ping消息来检测连接状态
        let _ping_message = json!({
            "jsonrpc": "2.0",
            "method": "ping",
            "params": {},
            "id": 0
        });
        
        match tokio::time::timeout(
            Duration::from_secs(5),
            self.send_request("ping", json!({}))
        ).await {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(_)) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    /// 发送RPC请求
    async fn send_request(&mut self, method: &str, params: Value) -> Result<Value, IpcClientError> {
        if !self.connected {
            return Err(IpcClientError::ConnectionError("未连接到服务器".to_string()));
        }

        let request_id = rand::random::<u64>();
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": request_id
        });

        let request_str = request.to_string();
        debug!("发送RPC请求: {}", request_str);

        if let Some(ref mut write) = self.write {
            write.send(Message::Text(request_str)).await?;
        }

        // 等待响应（带超时）
        let response = tokio::time::timeout(
            Duration::from_secs(10),
            self.receive_response(request_id)
        ).await
        .map_err(|_| IpcClientError::Timeout("等待响应超时".to_string()))??;

        Ok(response)
    }

    /// 接收RPC响应
    async fn receive_response(&mut self, expected_id: u64) -> Result<Value, IpcClientError> {
        if let Some(ref mut read) = self.read {
            while let Some(message) = read.next().await {
                let message = message?;
                
                if message.is_text() {
                    let text = message.to_text()?;
                    debug!("收到RPC响应: {}", text);
                    
                    let response: Value = serde_json::from_str(text)?;
                    
                    // 检查响应ID是否匹配
                    if let Some(response_id) = response["id"].as_u64() {
                        if response_id == expected_id {
                            // 检查错误
                            if let Some(error) = response.get("error") {
                                return Err(IpcClientError::RpcError(error.to_string()));
                            }
                            
                            // 返回结果
                            return Ok(response["result"].clone());
                        }
                    }
                }
            }
        }
        
        Err(IpcClientError::ConnectionError("连接已断开".to_string()))
    }

    /// 启动服务
    pub async fn start_service(&mut self) -> Result<bool, IpcClientError> {
        let result = self.send_request("start", json!({})).await?;
        Ok(result["success"].as_bool().unwrap_or(false))
    }

    /// 停止服务
    pub async fn stop_service(&mut self) -> Result<bool, IpcClientError> {
        let result = self.send_request("stop", json!({})).await?;
        Ok(result["success"].as_bool().unwrap_or(false))
    }

    /// 获取服务状态
    pub async fn get_status(&mut self) -> Result<Value, IpcClientError> {
        self.send_request("get_status", json!({})).await
    }

    /// 获取配置
    #[allow(dead_code)]
    pub async fn get_config(&mut self) -> Result<Value, IpcClientError> {
        self.send_request("get_config", json!({})).await
    }

    /// 设置配置
    #[allow(dead_code)]
    pub async fn set_config(&mut self, config: Value) -> Result<bool, IpcClientError> {
        let result = self.send_request("set_config", config).await?;
        Ok(result["success"].as_bool().unwrap_or(false))
    }

    /// 获取日志
    #[allow(dead_code)]
    pub async fn get_logs(&mut self, lines: u64) -> Result<Value, IpcClientError> {
        self.send_request("get_logs", json!({ "lines": lines })).await
    }

    /// 获取实时流量数据
    pub async fn get_realtime_traffic(&mut self, max_points: usize) -> Result<Value, IpcClientError> {
        self.send_request("get_realtime_traffic", json!({ "max_points": max_points })).await
    }

    // 代理设置功能已改为自动模式，无需手动设置
}

impl Drop for IpcClient {
    fn drop(&mut self) {
        if self.connected {
            let mut client = std::mem::replace(self, IpcClient::new(self.server_url.clone()));
            tokio::spawn(async move {
                if let Err(e) = client.disconnect().await {
                    warn!("断开连接时发生错误: {}", e);
                }
            });
        }
    }
}