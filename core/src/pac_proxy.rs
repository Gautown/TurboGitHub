use std::fs;
use std::path::PathBuf;
use tracing::info;

/// PAC 代理配置管理器
pub struct PacProxy {
    pac_file_path: PathBuf,
    proxy_server: String,
}

impl PacProxy {
    pub fn new(proxy_server: String) -> Self {
        let pac_file_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("turbogithub.pac");
        
        Self {
            pac_file_path,
            proxy_server,
        }
    }
    
    /// 生成 PAC 文件内容
    fn generate_pac_content(&self) -> String {
        format!(r#"
function FindProxyForURL(url, host) {{
    // GitHub 相关域名列表
    var githubDomains = [
        "github.com",
        "www.github.com",
        "api.github.com",
        "raw.githubusercontent.com",
        "gist.github.com",
        "github.io",
        "githubusercontent.com",
        "githubassets.com",
        "githubapp.com",
        "assets-cdn.github.com",
        "avatars.githubusercontent.com",
        "camo.githubusercontent.com",
        "collector.github.com",
        "education.github.com",
        "lab.github.com",
        "status.github.com",
        "support.github.com",
        "token.actions.githubusercontent.com",
        "vscode-auth.github.com"
    ];
    
    // AO3 (Archive of Our Own) 相关域名列表
    var ao3Domains = [
        "archiveofourown.org",
        "www.archiveofourown.org",
        "archiveofourown.com",
        "www.archiveofourown.com"
    ];
    
    // Pixiv 相关域名列表
    var pixivDomains = [
        "pixiv.net",
        "www.pixiv.net",
        "dic.pixiv.net",
        "fanbox.cc",
        "www.fanbox.cc"
    ];
    
    // 合并所有域名列表
    var allDomains = githubDomains.concat(ao3Domains).concat(pixivDomains);
    
    // 检查域名是否匹配
    for (var i = 0; i < allDomains.length; i++) {{
        if (dnsDomainIs(host, "." + allDomains[i])) {{
            return "SOCKS5 {proxy}; SOCKS {proxy}; PROXY {proxy}; DIRECT";
        }}
    }}
    
    // 其他域名直接连接
    return "DIRECT";
}}
"#, proxy = self.proxy_server)
    }
    
    /// 创建 PAC 文件
    pub fn create_pac_file(&self) -> anyhow::Result<()> {
        let content = self.generate_pac_content();
        
        fs::write(&self.pac_file_path, content)?;
        info!("✅ PAC 文件已创建：{:?}", self.pac_file_path);
        info!("📄 代理服务器：{}", self.proxy_server);
        
        Ok(())
    }
    
    /// 获取 PAC 文件 URL（用于配置浏览器或系统）
    pub fn get_pac_url(&self) -> String {
        let file_url = self.pac_file_path
            .to_string_lossy()
            .replace("\\", "/");
        
        format!("file:///{}", file_url.trim_start_matches('/'))
    }
    
    /// 删除 PAC 文件
    pub fn remove_pac_file(&self) -> anyhow::Result<()> {
        if self.pac_file_path.exists() {
            fs::remove_file(&self.pac_file_path)?;
            info!("✅ PAC 文件已删除");
        }
        Ok(())
    }
    
    /// 获取 PAC 文件路径
    pub fn get_pac_path(&self) -> &PathBuf {
        &self.pac_file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_pac_content() {
        let pac = PacProxy::new("127.0.0.1:7890".to_string());
        let content = pac.generate_pac_content();
        
        assert!(content.contains("FindProxyForURL"));
        assert!(content.contains("github.com"));
        assert!(content.contains("127.0.0.1:7890"));
    }
}
