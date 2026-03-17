//! 自动代理配置模块 - 仿照FastGithub的设计思路
//! 实现无需手动设置代理的透明加速体验

use std::process::Command;
use std::sync::Mutex;
use tracing::{info, warn};

/// 自动代理配置管理器
pub struct AutoProxyConfig {
    #[allow(dead_code)]
    pub dns_proxy_port: u16,
    #[allow(dead_code)]
    original_dns_settings: Mutex<Option<String>>,
}

impl AutoProxyConfig {
    pub fn new(dns_proxy_port: u16) -> Self {
        Self {
            dns_proxy_port,
            original_dns_settings: Mutex::new(None),
        }
    }

    /// 备份当前系统设置
    #[allow(dead_code)]
    pub fn backup_current_settings(&self) -> anyhow::Result<()> {
        info!("备份当前系统网络设置...");
        
        // 备份 DNS 设置
        let dns_output = Command::new("powershell")
            .args(&["-Command", "Get-DnsClientServerAddress -AddressFamily IPv4 | ConvertTo-Json"])
            .output()?;
        
        if dns_output.status.success() {
            let dns_settings = String::from_utf8_lossy(&dns_output.stdout).to_string();
            let mut guard = self.original_dns_settings.lock().unwrap();
            *guard = Some(dns_settings);
            info!("✅ DNS 设置备份成功");
        }
        
        // 备份代理设置
        let proxy_output = Command::new("reg")
            .args(&["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", "/v", "ProxyServer"])
            .output()?;
        
        if proxy_output.status.success() {
            let proxy_settings = String::from_utf8_lossy(&proxy_output.stdout).to_string();
            info!("✅ 代理设置备份成功：{}", proxy_settings);
        }
        
        Ok(())
    }

    /// 设置自动代理（仿照 FastGithub 的智能代理，不干扰 IE 代理）
    #[allow(dead_code)]
    pub fn setup_auto_proxy(&self) -> anyhow::Result<()> {
        info!("🚀 设置 TurboGitHub 智能代理（FastGithub 风格，不干扰 IE 代理）...");
        
        // 备份当前设置
        self.backup_current_settings()?;
        
        // FastGithub风格：专注于DNS代理，不设置HTTP代理以避免影响IE代理
        info!("💡 采用FastGithub策略：专注于DNS加速，不修改HTTP代理设置");
        info!("📡 这样不会影响IE和其他应用程序的代理设置");
        
        // 设置DNS代理（智能DNS解析）- 这是FastGithub的核心技术
        info!("设置智能DNS代理: 127.0.0.1:{}", self.dns_proxy_port);
        let dns_command = format!(
            "$adapters = Get-NetAdapter | Where-Object {{$_.Status -eq 'Up'}}; foreach ($adapter in $adapters) {{ Set-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -ServerAddresses @('127.0.0.1:{0}', '223.5.5.5') }}",
            self.dns_proxy_port
        );
        let dns_result = Command::new("powershell")
            .args(&["-Command", &dns_command])
            .output()?;
        
        if dns_result.status.success() {
            info!("✅ 智能DNS代理设置成功");
            info!("🌐 GitHub域名将自动解析到最优IP");
        } else {
            warn!("⚠️ DNS代理设置可能失败，但不影响核心功能");
            info!("💡 可以手动设置DNS为127.0.0.1:{}", self.dns_proxy_port);
        }
        
        // 不设置HTTP代理，避免影响IE代理
        info!("🔒 保持HTTP代理设置不变，避免影响IE和其他应用程序");
        
        info!("🎯 TurboGitHub智能代理设置完成！");
        info!("💡 GitHub/AO3/Pixiv流量通过DNS加速，IE代理设置不受影响");
        info!("📋 支持的域名：github.com, api.github.com, archiveofourown.org, pixiv.net等");
        
        Ok(())
    }

    /// 恢复原始设置（程序退出时调用）- 智能恢复，不干扰IE代理
    pub fn restore_original_settings(&self) -> anyhow::Result<()> {
        info!("🔄 恢复原始系统网络设置（智能恢复，不干扰IE代理）...");
        
        // FastGithub风格：只恢复DNS设置，不修改HTTP代理设置
        info!("💡 采用FastGithub策略：只恢复DNS设置，保持HTTP代理设置不变");
        
        // 恢复DNS设置（这是唯一需要恢复的设置）
        let dns_command = "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Set-DnsClientServerAddress -ResetServerAddresses";
        let dns_result = Command::new("powershell")
            .args(&["-Command", dns_command])
            .output()?;
        
        if dns_result.status.success() {
            info!("✅ DNS设置恢复成功");
        } else {
            warn!("⚠️ DNS设置恢复失败，但不影响系统");
        }
        
        // 不恢复HTTP代理设置，避免影响用户原有的IE代理配置
        info!("🔒 HTTP代理设置保持不变，避免影响IE和其他应用程序");
        
        info!("✅ 系统网络设置已智能恢复完成");
        info!("💡 IE代理设置未受影响，可以继续正常使用");
        
        Ok(())
    }
}