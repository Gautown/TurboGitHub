use std::process::Command;
use tracing::{info, warn, error, debug};

/// TurboGitHub DNS 代理服务器配置管理器
#[allow(dead_code)]
pub struct DnsProxyConfig {
    pub dns_proxy_server: String,
}

#[allow(dead_code)]
impl DnsProxyConfig {
    pub fn new(dns_proxy_server: String) -> Self {
        Self { dns_proxy_server }
    }

    /// 设置系统 DNS 代理服务器
    pub fn set_dns_proxy(&self) -> anyhow::Result<()> {
        info!("设置系统 DNS 代理服务器为：{}", self.dns_proxy_server);
        
        // 首先尝试使用 PowerShell 获取活动网络接口并设置 DNS
        let ps_output = Command::new("powershell")
            .args(&["-Command", &format!(
                "$adapter = Get-NetAdapter | Where-Object {{$_.Status -eq 'Up'}} | Select-Object -First 1; \
                 if ($adapter) {{ \
                     Set-DnsClientServerAddress -InterfaceIndex $adapter.ifIndex -ServerAddresses '{}' \
                 }} else {{ \
                     Write-Error 'No active network adapter found' \
                 }}", 
                self.dns_proxy_server
            )])
            .output();
        
        match ps_output {
            Ok(output) => {
                if output.status.success() {
                    info!("使用 PowerShell 设置系统 DNS 代理服务器成功");
                    return Ok(());
                } else {
                    let ps_error_msg = String::from_utf8_lossy(&output.stderr);
                    warn!("PowerShell 设置 DNS 代理服务器失败：{}", ps_error_msg);
                }
            }
            Err(e) => {
                warn!("PowerShell 命令执行失败：{}", e);
            }
        }
        
        // 备用方法：使用 netsh 命令，尝试常见的网络接口名称
        let interface_names = vec!["以太网", "本地连接", "Ethernet", "Wi-Fi", "WLAN"];
        
        for interface_name in interface_names {
            let output = Command::new("netsh")
                .args(&["interface", "ip", "set", "dns", interface_name, "static", &self.dns_proxy_server])
                .output();
            
            match output {
                Ok(out) => {
                    if out.status.success() {
                        info!("使用 netsh 在接口 '{}' 上设置 DNS 成功", interface_name);
                        return Ok(());
                    } else {
                        let error_msg = String::from_utf8_lossy(&out.stderr);
                        debug!("在接口 '{}' 上设置 DNS 失败：{}", interface_name, error_msg);
                    }
                }
                Err(e) => {
                    debug!("在接口 '{}' 上执行 netsh 命令失败：{}", interface_name, e);
                }
            }
        }
        
        error!("所有设置 DNS 的方法都失败了");
        Err(anyhow::anyhow!("无法设置系统 DNS 代理服务器"))
    }

    /// 清除系统 DNS 代理设置
    pub fn clear_dns_proxy(&self) -> anyhow::Result<()> {
        info!("清除系统 DNS 代理设置");
        
        // 首先尝试使用 PowerShell 清除 DNS
        let ps_output = Command::new("powershell")
            .args(&["-Command", "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Select-Object -First 1 | Set-DnsClientServerAddress -ResetServerAddresses"])
            .output();
        
        match ps_output {
            Ok(output) => {
                if output.status.success() {
                    info!("使用 PowerShell 清除系统 DNS 代理设置成功");
                    return Ok(());
                } else {
                    let ps_error_msg = String::from_utf8_lossy(&output.stderr);
                    warn!("PowerShell 清除 DNS 代理设置失败：{}", ps_error_msg);
                }
            }
            Err(e) => {
                warn!("PowerShell 命令执行失败：{}", e);
            }
        }
        
        // 备用方法：使用 netsh 命令，尝试常见的网络接口名称
        let interface_names = vec!["以太网", "本地连接", "Ethernet", "Wi-Fi", "WLAN"];
        
        for interface_name in interface_names {
            let output = Command::new("netsh")
                .args(&["interface", "ip", "set", "dns", interface_name, "dhcp"])
                .output();
            
            match output {
                Ok(out) => {
                    if out.status.success() {
                        info!("使用 netsh 在接口 '{}' 上清除 DNS 成功", interface_name);
                        return Ok(());
                    } else {
                        let error_msg = String::from_utf8_lossy(&out.stderr);
                        debug!("在接口 '{}' 上清除 DNS 失败：{}", interface_name, error_msg);
                    }
                }
                Err(e) => {
                    debug!("在接口 '{}' 上执行 netsh 命令失败：{}", interface_name, e);
                }
            }
        }
        
        warn!("所有清除 DNS 的方法都失败了");
        Ok(()) // 即使失败也返回成功，避免影响用户体验
    }
}
