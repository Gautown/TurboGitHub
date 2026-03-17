
function FindProxyForURL(url, host) {
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
    
    // 检查域名是否匹配
    for (var i = 0; i < githubDomains.length; i++) {
        if (dnsDomainIs(host, "." + githubDomains[i])) {
            return "SOCKS5 127.0.0.1:7890; SOCKS 127.0.0.1:7890; PROXY 127.0.0.1:7890; DIRECT";
        }
    }
    
    // 其他域名直接连接
    return "DIRECT";
}
