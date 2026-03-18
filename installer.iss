; TurboGitHub Inno Setup 安装脚本
; 创建专业的 Windows 安装包

#define MyAppName "TurboGitHub"
#define MyAppVersion "0.0.1"
#define MyAppPublisher "Gautown"
#define MyAppURL "https://github.com/Gautown/TurboGitHub"
#define MyAppExeName "turbogithub-gui.exe"
#define MyAppCopyright "Copyright © 2026 Gautown"

[Setup]
; 基本设置
AppId={{12345678-1234-1234-1234-123456789012}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
AppCopyright={#MyAppCopyright}

; 安装目录
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
DisableProgramGroupPage=yes

; 许可证
LicenseFile=LICENSE

; 输出设置
OutputDir=installer_output
OutputBaseFilename=TurboGitHub-{#MyAppVersion}-Setup
SetupIconFile=assets\icons\logo.ico
UninstallDisplayIcon={app}\{#MyAppExeName}

; 压缩
Compression=lzma2/max
SolidCompression=yes
LZMAUseSeparateProcess=yes
LZMADictionarySize=64MB
LZMANumFastBytes=273
LZMANumBlockThreads=2

; 向导
WizardStyle=modern
WizardResizable=no
SetupLockingSupported=yes

; 权限
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64

; 其他
RestartIfNeededByRun=no
ShowLanguageDialog=auto

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
chinesesimplified.LaunchProgram=启动 TurboGitHub
chinesesimplified.CreateDesktopIcon=创建桌面快捷方式
chinesesimplified.CreateQuickLaunchIcon=创建快速启动栏快捷方式
chinesesimplified.AddToPath=添加到系统 PATH

english.LaunchProgram=Launch TurboGitHub
english.CreateDesktopIcon=Create Desktop Icon
english.CreateQuickLaunchIcon=Create Quick Launch Icon
english.AddToPath=Add to system PATH

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 6.1; Check: not IsAdminInstallMode
Name: "addToPath"; Description: "{cm:AddToPath}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; 主程序
Source: "target\release\turbogithub-gui.exe"; DestDir: "{app}"; Flags: ignoreversion createallsubdirs
Source: "target\release\turbogithub-gui.pdb"; DestDir: "{app}"; Flags: ignoreversion dontcopy; Check: FileExists('target\release\turbogithub-gui.pdb')

; 配置文件
Source: "config.toml"; DestDir: "{app}"; Flags: ignoreversion
Source: "core\config.toml"; DestDir: "{app}\core"; Flags: ignoreversion

; 资源文件
Source: "assets\icons\*"; DestDir: "{app}\assets\icons"; Flags: ignoreversion recursesubdirs
Source: "assets\images\*"; DestDir: "{app}\assets\images"; Flags: ignoreversion recursesubdirs

; 脚本和文档
Source: "启动 TurboGitHub.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

; 额外文件（如果存在）
Source: "CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion dontcopy; Check: FileExists('CHANGELOG.md')
Source: "快速启动.bat"; DestDir: "{app}"; Flags: ignoreversion dontcopy; Check: FileExists('快速启动.bat')

[Icons]
; 开始菜单
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"

; 桌面快捷方式
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

; 快速启动栏
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: quicklaunchicon

[Run]
; 安装完成后运行
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent; WorkingDir: "{app}"

[Code]
// 检查是否已安装
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
  PrevVersion: String;
begin
  Result := True;
  
  // 检查是否已安装
  if RegQueryStringValue(HKLM, 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#SetupSetting("AppId")}_is1', 'DisplayVersion', PrevVersion) then
  begin
    if MsgBox('TurboGitHub ' + PrevVersion + ' 已经安装。是否要卸载旧版本并继续安装？', mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDNO then
      Result := False;
  end;
end;

// 安装完成后提示
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    // 可以在这里添加安装后的操作
  end;
end;

// 卸载确认
function InitializeUninstall(): Boolean;
begin
  Result := True;
  
  if not IsAdmin then
  begin
    if MsgBox('卸载 TurboGitHub 需要管理员权限。是否继续？', mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDNO then
      Result := False;
  end;
end;

// 注册文件关联（可选）
procedure RegisterFileTypes;
begin
  // 如果需要关联特定文件类型，可以在这里添加
end;

[UninstallDelete]
; 卸载时删除的文件
Type: filesandordirs; Name: "{app}"
Type: files; Name: "{userappdata}\TurboGitHub\*"
Type: dirs; Name: "{userappdata}\TurboGitHub"

[UninstallRun]
; 卸载时运行的程序
Filename: "{app}\{#MyAppExeName}"; Parameters: "/uninstall"; Flags: runhidden; StatusMsg: "Stopping TurboGitHub..."

[Registry]
; 注册表项
Root: HKLM; Subkey: "Software\{#MyAppPublisher}\{#MyAppName}"; ValueType: string; ValueName: "Version"; ValueData: "{#MyAppVersion}"; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\{#MyAppPublisher}\{#MyAppName}"; ValueType: string; ValueName: "InstallLocation"; ValueData: "{app}"; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\{#MyAppPublisher}\{#MyAppName}"; ValueType: dword; ValueName: "Installed"; ValueData: 1; Flags: uninsdeletekey

[INI]
; INI 文件配置（如果需要）
; Filename: "{app}\settings.ini"; Section: "General"; Key: "Version"; String: "{#MyAppVersion}"

[Messages]
; 自定义消息
chinesesimplified.SetupAppTitle=TurboGitHub 安装程序
chinesesimplified.SetupAppVersion=版本 {#MyAppVersion}
chinesesimplified.BeveledLabel=TurboGitHub {#MyAppVersion}

english.SetupAppTitle=TurboGitHub Setup
english.SetupAppVersion=Version {#MyAppVersion}
english.BeveledLabel=TurboGitHub {#MyAppVersion}

[LangOptions]
; 语言选项
chinesesimplified.LanguageName=简体中文
chinesesimplified.LanguageCodePage=65001

english.LanguageName=English
english.LanguageCodePage=65001
