@echo off
REM TurboGitHub启动脚本

cd /d "%~dp0"
echo 正在启动TurboGitHub...
echo 请稍候，第一次启动可能需要较长时间...

REM 运行GUI应用
cargo run --package turbogithub-gui

pause