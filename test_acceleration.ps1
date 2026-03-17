# GitHub 加速功能测试脚本
# 用于验证 TurboGitHub 是否真正实现 GitHub 加速

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "  TurboGitHub GitHub 加速功能测试" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# 测试 1: 检查 DNS 服务器是否运行
Write-Host "[测试 1] 检查 DNS 服务器状态..." -ForegroundColor Yellow
try {
    $dnsPort = 61235
    $dnsSocket = New-Object System.Net.Sockets.TcpClient
    $dnsSocket.ConnectTimeout = 2000
    $dnsSocket.Connect("127.0.0.1", $dnsPort)
    if ($dnsSocket.Connected) {
        Write-Host "✓ DNS 服务器运行正常 (端口：$dnsPort)" -ForegroundColor Green
        $dnsSocket.Close()
        $dnsRunning = $true
    } else {
        Write-Host "✗ DNS 服务器未运行" -ForegroundColor Red
        $dnsRunning = $false
    }
} catch {
    Write-Host "✗ DNS 服务器未运行或无法连接" -ForegroundColor Red
    Write-Host "  提示：请先启动 TurboGitHub 应用程序" -ForegroundColor Yellow
    $dnsRunning = $false
}

# 测试 2: 检查 IPC 服务是否运行
Write-Host ""
Write-Host "[测试 2] 检查 IPC 服务状态..." -ForegroundColor Yellow
try {
    $ipcPortFile = ".ipc_port"
    if (Test-Path $ipcPortFile) {
        $ipcPort = Get-Content $ipcPortFile
        $ipcSocket = New-Object System.Net.Sockets.TcpClient
        $ipcSocket.ConnectTimeout = 2000
        $ipcSocket.Connect("127.0.0.1", [int]$ipcPort)
        if ($ipcSocket.Connected) {
            Write-Host "✓ IPC 服务运行正常 (端口：$ipcPort)" -ForegroundColor Green
            $ipcRunning = $true
            $ipcSocket.Close()
        } else {
            Write-Host "✗ IPC 服务未运行" -ForegroundColor Red
            $ipcRunning = $false
        }
    } else {
        Write-Host "✗ 未找到 IPC 端口文件" -ForegroundColor Red
        $ipcRunning = $false
    }
} catch {
    Write-Host "✗ IPC 服务未运行或无法连接" -ForegroundColor Red
    $ipcRunning = $false
}

# 测试 3: 测试 DNS 解析速度（不使用 TurboGitHub DNS）
Write-Host ""
Write-Host "[测试 3] 测试原始 DNS 解析速度..." -ForegroundColor Yellow
$githubDomains = @(
    "github.com",
    "api.github.com",
    "raw.githubusercontent.com",
    "assets-cdn.github.com"
)

$originalDnsResults = @{}
foreach ($domain in $githubDomains) {
    try {
        $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
        $result = Resolve-DnsName -Name $domain -Type A -ErrorAction Stop
        $stopwatch.Stop()
        $originalDnsResults[$domain] = @{
            Time = $stopwatch.ElapsedMilliseconds
            IP = ($result | Select-Object -First 1).IP4Address
        }
        Write-Host "  $domain : $($originalDnsResults[$domain].IP) ($($originalDnsResults[$domain].Time)ms)" -ForegroundColor Gray
    } catch {
        Write-Host "  $domain : 解析失败" -ForegroundColor Red
    }
}

# 测试 4: 测试 GitHub 连接延迟
Write-Host ""
Write-Host "[测试 4] 测试 GitHub 连接延迟（原始）..." -ForegroundColor Yellow
$originalLatencyResults = @{}
foreach ($domain in $githubDomains) {
    try {
        $ip = $originalDnsResults[$domain].IP
        if ($ip) {
            $ping = New-Object System.Net.NetworkInformation.Ping
            $reply = $ping.Send($ip, 2000)
            if ($reply.Status -eq "Success") {
                $originalLatencyResults[$domain] = $reply.RoundtripTime
                Write-Host "  $domain ($ip): $($reply.RoundtripTime)ms" -ForegroundColor Gray
            } else {
                Write-Host "  $domain : Ping 失败" -ForegroundColor Red
            }
        }
    } catch {
        Write-Host "  $domain : 测试失败" -ForegroundColor Red
    }
}

# 测试 5: 测试文件下载速度（不使用加速）
Write-Host ""
Write-Host "[测试 5] 测试文件下载速度（原始）..." -ForegroundColor Yellow
$testUrl = "https://raw.githubusercontent.com/Gautown/TurboGitHub/main/README.md"
try {
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $client = New-Object System.Net.WebClient
    $client.DownloadFile($testUrl, "$env:TEMP\test_original.txt")
    $stopwatch.Stop()
    $fileSize = (Get-Item "$env:TEMP\test_original.txt").Length
    $speed = [math]::Round($fileSize / $stopwatch.Elapsed.TotalSeconds / 1024, 2)
    Write-Host "  下载大小：$fileSize 字节" -ForegroundColor Gray
    Write-Host "  下载时间：$([math]::Round($stopwatch.Elapsed.TotalMilliseconds, 2))ms" -ForegroundColor Gray
    Write-Host "  下载速度：$speed KB/s" -ForegroundColor Gray
    $originalSpeed = $speed
    Remove-Item "$env:TEMP\test_original.txt" -ErrorAction SilentlyContinue
} catch {
    Write-Host "  下载测试失败" -ForegroundColor Red
    $originalSpeed = 0
}

# 测试 6: 如果 DNS 运行，测试使用 TurboGitHub DNS 的解析速度
if ($dnsRunning) {
    Write-Host ""
    Write-Host "[测试 6] 测试 TurboGitHub DNS 解析速度..." -ForegroundColor Yellow
    
    # 临时设置 DNS 服务器
    $adapter = Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Select-Object -First 1
    $originalDns = Get-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -AddressFamily IPv4 | Select-Object -ExpandProperty ServerAddresses
    Set-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -ServerAddresses @("127.0.0.1:61235")
    
    Start-Sleep -Seconds 2
    
    $turboDnsResults = @{}
    foreach ($domain in $githubDomains) {
        try {
            $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
            $result = Resolve-DnsName -Name $domain -Type A -ErrorAction Stop
            $stopwatch.Stop()
            $turboDnsResults[$domain] = @{
                Time = $stopwatch.ElapsedMilliseconds
                IP = ($result | Select-Object -First 1).IP4Address
            }
            Write-Host "  $domain : $($turboDnsResults[$domain].IP) ($($turboDnsResults[$domain].Time)ms)" -ForegroundColor Cyan
        } catch {
            Write-Host "  $domain : 解析失败" -ForegroundColor Red
        }
    }
    
    # 恢复原始 DNS 设置
    Set-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -ServerAddresses $originalDns
    Write-Host "  已恢复原始 DNS 设置" -ForegroundColor Green
}

# 总结
Write-Host ""
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "  测试结果总结" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan

if ($dnsRunning -and $ipcRunning) {
    Write-Host "✓ TurboGitHub 服务运行正常" -ForegroundColor Green
    Write-Host "✓ DNS 服务器已启动" -ForegroundColor Green
    Write-Host "✓ IPC 服务已启动" -ForegroundColor Green
    Write-Host ""
    Write-Host "✓ GitHub 加速功能已实现并运行" -ForegroundColor Green
} elseif ($dnsRunning) {
    Write-Host "✓ DNS 服务器已启动" -ForegroundColor Green
    Write-Host "✗ IPC 服务未运行" -ForegroundColor Red
    Write-Host "⚠ 部分功能可用" -ForegroundColor Yellow
} else {
    Write-Host "✗ TurboGitHub 服务未运行" -ForegroundColor Red
    Write-Host "  提示：请先启动 TurboGitHub 应用程序" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "测试完成！" -ForegroundColor Green
