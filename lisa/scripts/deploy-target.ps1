<#
.SYNOPSIS
    Lisa -> webOS TV (ARM64) target deploy script (Windows)
.DESCRIPTION
    Deploy to webOS TV based on config/config.arm64.toml.
    No additional dependencies required - runs on default PowerShell (5.1+).
.PARAMETER TargetIP
    Target device IP address
.PARAMETER ProxyJump
    SSH ProxyJump host (e.g., user@jumphost or user@jumphost:port)
.EXAMPLE
    .\deploy-target.ps1 -TargetIP 192.168.0.10
.EXAMPLE
    .\deploy-target.ps1 -TargetIP 192.168.0.10 -ProxyJump user@jumphost
.NOTES
    If execution policy blocks this script, run:
      powershell -ExecutionPolicy Bypass -File deploy-target.ps1 -TargetIP <IP>
#>

[CmdletBinding()]
param(
    [string]$TargetIP,
    [string]$ProxyJump
)

# =====================================================================
# Constants
# =====================================================================
$ScriptDir       = Split-Path -Parent $MyInvocation.MyCommand.Path
$LisaDir         = Split-Path -Parent $ScriptDir
$ProfileDir      = Join-Path $LisaDir "profiles\lisa"
$ConfigFile      = Join-Path $LisaDir "config\config.arm64.toml"
$Binary          = Join-Path $LisaDir "release\arm64\zeroclaw"
$GogBinary       = Join-Path $LisaDir "release\arm64\gog"
$GogConfigLocal  = Join-Path $env:APPDATA "gogcli"
if (-not (Test-Path $GogConfigLocal)) {
    # Linux/macOS fallback
    $GogConfigLocal = Join-Path $HOME ".config/gogcli"
}
$LisaEnvFile     = Join-Path $LisaDir "profiles\lisa\lisa.env"

$TargetUser      = "root"
$TargetDeployDir = "/home/root/lisa"
$TargetZeroclawDir = "/home/root/.zeroclaw"
$TargetWorkspaceDir = "$TargetZeroclawDir/workspace"
$TargetGogConfig = "/home/root/.config/gogcli"

# Build common SSH/SCP options
$SSHOpts = @("-o", "ConnectTimeout=10", "-o", "StrictHostKeyChecking=accept-new")
if ($ProxyJump) {
    $SSHOpts += @("-J", $ProxyJump)
}

# =====================================================================
# Helper functions
# =====================================================================
function Write-OK   { Write-Host "  OK" -ForegroundColor Green }
function Write-Fail($msg) {
    Write-Host "  FAIL - $msg" -ForegroundColor Red
    exit 1
}

function Invoke-SSH {
    param([string]$Command)
    $output = & ssh.exe @script:SSHOpts "${TargetUser}@${TargetIP}" $Command 2>$null
    if ($null -eq $output) { return "" }
    if ($output -is [array]) { return ($output -join "`n").Trim() }
    return $output.ToString().Trim()
}

function Invoke-SCP {
    param([string]$LocalPath, [string]$RemotePath)
    # -O forces legacy SCP protocol (avoids SFTP-through-ProxyJump issues)
    & scp.exe -O @script:SSHOpts $LocalPath "${TargetUser}@${TargetIP}:${RemotePath}"
    return ($LASTEXITCODE -eq 0)
}

function Invoke-SCPRecursive {
    param([string]$LocalPath, [string]$RemotePath)
    & scp.exe -O -r @script:SSHOpts $LocalPath "${TargetUser}@${TargetIP}:${RemotePath}"
    return ($LASTEXITCODE -eq 0)
}

function Write-RemoteFile {
    param([string]$Content, [string]$RemotePath)
    $tmpFile = [System.IO.Path]::GetTempFileName()
    try {
        $utf8NoBom = New-Object System.Text.UTF8Encoding $false
        $lfContent = $Content.Replace("`r`n", "`n")
        [System.IO.File]::WriteAllText($tmpFile, $lfContent, $utf8NoBom)
        return (Invoke-SCP $tmpFile $RemotePath)
    }
    finally {
        Remove-Item $tmpFile -Force -ErrorAction SilentlyContinue
    }
}

# =====================================================================
# Main
# =====================================================================
Write-Host ""
Write-Host "Lisa -> webOS TV target deploy"
Write-Host "=============================="

# -- Validate --
if (-not (Test-Path $ConfigFile)) {
    Write-Host "Error: Target config not found: $ConfigFile" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $Binary)) {
    Write-Host "Error: ARM64 binary not found: $Binary" -ForegroundColor Red
    exit 1
}
if (-not $TargetIP) {
    $TargetIP = Read-Host "Enter target IP"
    if (-not $TargetIP) {
        Write-Host "Error: TargetIP is required." -ForegroundColor Red
        exit 1
    }
}

Write-Host "  Target: ${TargetUser}@${TargetIP}"
if ($ProxyJump) {
    Write-Host "  Proxy:  ${ProxyJump}"
}
Write-Host ""

$Total = 14

# ------------------------------------------------------------------
# Step 1: SSH connection test
# ------------------------------------------------------------------
Write-Host "[1/$Total] Testing SSH connection..."
$test = Invoke-SSH "echo ok"
if ($LASTEXITCODE -ne 0) {
    Write-Fail "Cannot connect to ${TargetUser}@${TargetIP}. Check device IP and SSH access."
}
Write-OK

# ------------------------------------------------------------------
# Step 2: Create target directories
# ------------------------------------------------------------------
Write-Host "[2/$Total] Creating target directories..."
Invoke-SSH "mkdir -p $TargetDeployDir $TargetZeroclawDir $TargetWorkspaceDir ${TargetWorkspaceDir}/skills" | Out-Null
Write-OK

# ------------------------------------------------------------------
# Step 3: Transfer binary
# ------------------------------------------------------------------
Write-Host "[3/$Total] Transferring binary..."
if (-not (Invoke-SCP $Binary "${TargetDeployDir}/zeroclaw")) {
    Write-Fail "Could not copy binary to device."
}
Invoke-SSH "chmod +x ${TargetDeployDir}/zeroclaw" | Out-Null
if (Test-Path $GogBinary) {
    if (Invoke-SCP $GogBinary "${TargetDeployDir}/gog") {
        Invoke-SSH "chmod +x ${TargetDeployDir}/gog" | Out-Null
        Write-Host "  gog (calendar CLI)"
    }
}
Write-OK

# ------------------------------------------------------------------
# Step 4: Transfer config.toml
# ------------------------------------------------------------------
Write-Host "[4/$Total] Transferring config.toml..."
# Backup existing config
Invoke-SSH "if [ -f ${TargetZeroclawDir}/config.toml ]; then cp ${TargetZeroclawDir}/config.toml ${TargetZeroclawDir}/config.toml.bak.`$(date +%s); echo '  Existing config backed up'; fi" | Out-Null
if (-not (Invoke-SCP $ConfigFile "${TargetZeroclawDir}/config.toml")) {
    Write-Fail "Could not copy config to device."
}
Invoke-SSH "chmod 600 ${TargetZeroclawDir}/config.toml" | Out-Null
Write-OK

# ------------------------------------------------------------------
# Step 5: Transfer workspace files
# ------------------------------------------------------------------
Write-Host "[5/$Total] Transferring workspace files..."
foreach ($f in @("SOUL.md", "AGENTS.md", "USER.md")) {
    $fp = Join-Path $ProfileDir $f
    if (Test-Path $fp) {
        if (Invoke-SCP $fp "${TargetWorkspaceDir}/$f") {
            Write-Host "  $f"
        }
    }
}
$skillsDir = Join-Path $ProfileDir "skills"
if (Test-Path $skillsDir) {
    Get-ChildItem -Path $skillsDir -Directory | ForEach-Object {
        Invoke-SCPRecursive $_.FullName "${TargetWorkspaceDir}/skills/" | Out-Null
    }
    $sc = (Get-ChildItem $skillsDir -Directory).Count
    Write-Host "  $sc skill(s)"
    Invoke-SSH "find ${TargetWorkspaceDir}/skills -name '*.sh' -exec chmod +x {} +" | Out-Null
}
Write-OK

# ------------------------------------------------------------------
# Step 6: Configure Target TV for tv-control skill
# ------------------------------------------------------------------
Write-Host "[6/$Total] Configuring Target TV..."
$tvSkillFile = "${TargetWorkspaceDir}/skills/tv-control/SKILL.md"
Write-Host "  Select TV location for tv-control skill:"
Write-Host "    1) N/A    - no target TV (skip tv-control)"
Write-Host "    2) local  - commands run directly on this device"
Write-Host "    3) remote - commands run via SSH to another TV"
$tvChoice = Read-Host "  Choice [1/2/3] (default: 1)"
if (-not $tvChoice) { $tvChoice = "1" }

switch ($tvChoice) {
    "2" {
        $tvLocation = "local"
        $tvIP = "N/A"
    }
    "3" {
        $tvLocation = "remote"
        $tvIP = Read-Host "  Remote TV IP address"
        if (-not $tvIP) {
            Write-Host "  WARNING: no IP provided, setting to N/A"
            $tvLocation = "N/A"
            $tvIP = "N/A"
        }
    }
    default {
        $tvLocation = "N/A"
        $tvIP = "N/A"
    }
}

$tvSedScript = "sed -i 's/^- \*\*Location\*\*:.*/- **Location**: ${tvLocation}/' '${tvSkillFile}'"
if ($tvLocation -eq "remote") {
    $tvSedScript += "; sed -i 's/^- \*\*IP\*\*:.*/- **IP**: ${tvIP}/' '${tvSkillFile}'"
} else {
    $tvSedScript += "; sed -i 's/^- \*\*IP\*\*:.*/- **IP**: N\/A/' '${tvSkillFile}'"
}
$tvResult = Invoke-SSH "if [ -f '${tvSkillFile}' ]; then ${tvSedScript}; echo '  Location: ${tvLocation}'; echo '  IP: ${tvIP}'; else echo '  SKIP - tv-control SKILL.md not found on target'; fi"
Write-Host $tvResult
Write-OK

# ------------------------------------------------------------------
# Step 7: gog (calendar) setup & transfer
# ------------------------------------------------------------------
Write-Host "[7/$Total] Setting up gog (calendar)..."

# Load existing GOG_ACCOUNT from lisa.env if available
$savedGogAccount = ""
if (Test-Path $LisaEnvFile) {
    $match = Select-String -Path $LisaEnvFile -Pattern '^(?:export )?GOG_ACCOUNT=(.+)$' -ErrorAction SilentlyContinue
    if ($match) { $savedGogAccount = $match.Matches[0].Groups[1].Value }
}

# Auto-setup: gog OAuth if no local tokens
$gogKeyringDir = Join-Path $GogConfigLocal "keyring"
$hasTokens = (Test-Path $gogKeyringDir) -and ((Get-ChildItem $gogKeyringDir -ErrorAction SilentlyContinue).Count -gt 0)

if (-not $hasTokens) {
    Write-Host "  No gog OAuth tokens found locally."
    # Install gog if not available
    if (-not (Get-Command gog -ErrorAction SilentlyContinue)) {
        Write-Host "  gog CLI not found."
        $installGog = Read-Host "  Install gog now? [Y/n]"
        if ($installGog -notmatch '^[nN]$') {
            if (Get-Command brew -ErrorAction SilentlyContinue) {
                Write-Host "  Installing via Homebrew..."
                & brew install steipete/tap/gogcli
            } elseif (Get-Command go -ErrorAction SilentlyContinue) {
                Write-Host "  Installing via go install..."
                & go install github.com/steipete/gogcli/cmd/gog@latest
                # go install puts binary in GOPATH/bin — add to PATH
                $gobin = & go env GOPATH
                $gobin = Join-Path $gobin "bin"
                if ((Test-Path $gobin) -and ($env:PATH -notmatch [regex]::Escape($gobin))) {
                    $env:PATH = "$gobin;$env:PATH"
                }
            } else {
                Write-Host "  Neither brew nor go found. Install one first:"
                Write-Host "    brew: https://brew.sh"
                Write-Host "    go:   https://go.dev/dl"
            }
        }
    }

    # Auth setup if gog is available
    if (Get-Command gog -ErrorAction SilentlyContinue) {
        $setupGog = Read-Host "  Set up Google Calendar (gog) now? [Y/n]"
        if ($setupGog -notmatch '^[nN]$') {
            # Step A: register OAuth client credentials (requires client_secret.json from Google Cloud Console)
            $credsJson = Join-Path $GogConfigLocal "credentials.json"
            if (-not (Test-Path $credsJson)) {
                $clientSecretPath = Read-Host "  Path to client_secret.json (from Google Cloud Console)"
                if ($clientSecretPath -and (Test-Path $clientSecretPath)) {
                    & gog auth credentials $clientSecretPath
                    if ($LASTEXITCODE -ne 0) { Write-Host "  WARNING: credentials setup failed" }
                } else {
                    Write-Host "  Skipped - file not found"
                }
            }

            # Step B: set keyring backend to file (for headless target)
            Write-Host "  Setting keyring backend to file..."
            & gog auth keyring file
            if ($LASTEXITCODE -ne 0) { Write-Host "  WARNING: keyring setup failed" }

            # Step C: OAuth authentication
            $credsJson = Join-Path $GogConfigLocal "credentials.json"
            if (Test-Path $credsJson) {
                if ($savedGogAccount) {
                    $gogEmail = Read-Host "  Google account email [$savedGogAccount]"
                    if (-not $gogEmail) { $gogEmail = $savedGogAccount }
                } else {
                    $gogEmail = Read-Host "  Google account email"
                }
                if ($gogEmail) {
                    # Set keyring password as env var so gog uses it without prompting
                    $gogKrPass = Read-Host "  Keyring password (for encrypting OAuth tokens)" -AsSecureString
                    $gogKrPassPlain = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($gogKrPass))
                    $env:GOG_KEYRING_PASSWORD = $gogKrPassPlain
                    Write-Host "  OAuth URL will be printed below."
                    Write-Host "  Open it in any browser, authorize, then paste the redirect URL back here."
                    & gog auth add $gogEmail --services calendar --manual
                }
            }
        }
    }
}

# Transfer gog credentials to target
if (Test-Path $GogConfigLocal) {
    $gogFileCount = (Get-ChildItem $GogConfigLocal -Recurse -File -ErrorAction SilentlyContinue).Count
    if ($gogFileCount -gt 0) {
        Invoke-SSH "mkdir -p ${TargetGogConfig}" | Out-Null
        Invoke-SCPRecursive "$GogConfigLocal" "/home/root/.config/" | Out-Null
        Invoke-SSH "chmod -R 600 ${TargetGogConfig}; chmod 700 ${TargetGogConfig} ${TargetGogConfig}/keyring 2>/dev/null" | Out-Null
        Write-Host "  $gogFileCount file(s) transferred"
    } else {
        Write-Host "  SKIP - no gog credentials to transfer"
    }
} else {
    Write-Host "  SKIP - no gog credentials to transfer"
}

# Auto-setup: lisa.env — create or update
$defaultAccount = if ($gogEmail) { $gogEmail } elseif ($savedGogAccount) { $savedGogAccount } else { "" }
$defaultPassword = if ($gogKrPassPlain) { $gogKrPassPlain } else { "" }
$savedGogPassword = ""
if (Test-Path $LisaEnvFile) {
    $pwMatch = Select-String -Path $LisaEnvFile -Pattern '^(?:export )?GOG_KEYRING_PASSWORD=(.+)$' -ErrorAction SilentlyContinue
    if ($pwMatch) { $savedGogPassword = $pwMatch.Matches[0].Groups[1].Value }
}

if (-not (Test-Path $LisaEnvFile)) {
    $createEnv = Read-Host "  Create lisa.env? [Y/n]"
    if ($createEnv -notmatch '^[nN]$') {
        if ($defaultAccount) {
            $inputAccount = Read-Host "  GOG_ACCOUNT (email) [$defaultAccount]"
            if (-not $inputAccount) { $inputAccount = $defaultAccount }
        } else {
            $inputAccount = Read-Host "  GOG_ACCOUNT (email)"
        }
        if ($defaultPassword) {
            $inputPassword = $defaultPassword
            Write-Host "  GOG_KEYRING_PASSWORD: (auto-filled from keyring setup)"
        } else {
            $inputPassword = Read-Host "  GOG_KEYRING_PASSWORD"
        }
        $today = Get-Date -Format "yyyy-MM-dd"
        $envContent = @"
# Lisa target environment variables
# Generated by deploy-target.ps1 on $today

# --- Google Calendar (gog) ---
export GOG_ACCOUNT=$inputAccount
export GOG_KEYRING_PASSWORD=$inputPassword
export GOG_KEYRING_BACKEND=file
"@
        $utf8NoBom = New-Object System.Text.UTF8Encoding $false
        [System.IO.File]::WriteAllText($LisaEnvFile, $envContent.Replace("`r`n", "`n"), $utf8NoBom)
        Write-Host "  lisa.env created"
    }
} else {
    # lisa.env exists — update missing or changed values

    # GOG_KEYRING_PASSWORD: prompt if empty and no default from this session
    if (-not $savedGogPassword -and -not $defaultPassword) {
        $secPw = Read-Host "  GOG_KEYRING_PASSWORD is empty in lisa.env. Enter password" -AsSecureString
        $defaultPassword = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($secPw))
    }
    if ($defaultPassword -and ($defaultPassword -ne $savedGogPassword)) {
        $content = Get-Content $LisaEnvFile -Raw
        $content = $content -replace '(?m)^(?:export )?GOG_KEYRING_PASSWORD=.*$', "export GOG_KEYRING_PASSWORD=$defaultPassword"
        $utf8NoBom = New-Object System.Text.UTF8Encoding $false
        [System.IO.File]::WriteAllText($LisaEnvFile, $content.Replace("`r`n", "`n"), $utf8NoBom)
        Write-Host "  lisa.env updated (GOG_KEYRING_PASSWORD)"
    }

    # GOG_ACCOUNT: update if changed
    if ($defaultAccount -and ($defaultAccount -ne $savedGogAccount)) {
        $content = Get-Content $LisaEnvFile -Raw
        $content = $content -replace '(?m)^(?:export )?GOG_ACCOUNT=.*$', "export GOG_ACCOUNT=$defaultAccount"
        $utf8NoBom = New-Object System.Text.UTF8Encoding $false
        [System.IO.File]::WriteAllText($LisaEnvFile, $content.Replace("`r`n", "`n"), $utf8NoBom)
        Write-Host "  lisa.env updated (GOG_ACCOUNT)"
    }
}

# Transfer lisa.env
if (Test-Path $LisaEnvFile) {
    if (Invoke-SCP $LisaEnvFile "${TargetDeployDir}/lisa.env") {
        Invoke-SSH "chmod 600 ${TargetDeployDir}/lisa.env" | Out-Null
        Write-Host "  lisa.env"
    }
}
Write-OK

# ------------------------------------------------------------------
# Step 8: /etc/hosts setup
# ------------------------------------------------------------------
Write-Host "[8/$Total] Setting up /etc/hosts..."
$hostsContent = Invoke-SSH "cat /home/root/hosts 2>/dev/null"
if ($hostsContent -notmatch "tvdevops.openai.azure.com") {
    if (-not $hostsContent) {
        Invoke-SSH "cp /etc/hosts /home/root/hosts" | Out-Null
    }
    Invoke-SSH "printf '\n# Azure OpenAI endpoint for Lisa\n10.182.173.75 tvdevops.openai.azure.com\n' >> /home/root/hosts" | Out-Null
}
Invoke-SSH "mount --bind /home/root/hosts /etc/hosts 2>/dev/null" | Out-Null
Write-OK

# ------------------------------------------------------------------
# Step 9: Auto bind mount setup
# ------------------------------------------------------------------
Write-Host "[9/$Total] Setting up auto bind mount..."

$bindScript = @"
#!/bin/sh
HOSTS_RW="/home/root/hosts"
if [ -f "`$HOSTS_RW" ] && ! mount | grep -q "/etc/hosts"; then
    mount --bind "`$HOSTS_RW" /etc/hosts
fi
"@
Write-RemoteFile $bindScript "${TargetDeployDir}/bind-hosts.sh" | Out-Null
Invoke-SSH "chmod +x ${TargetDeployDir}/bind-hosts.sh" | Out-Null

# Add to .profile if not already present
$profileContent = Invoke-SSH "cat ~/.profile 2>/dev/null"
if ($profileContent -notmatch "bind-hosts.sh") {
    Invoke-SSH "printf '\n# Lisa: auto bind mount\n[ -x /home/root/lisa/bind-hosts.sh ] && /home/root/lisa/bind-hosts.sh 2>/dev/null\n' >> ~/.profile" | Out-Null
}
if ($profileContent -notmatch '/home/root/lisa.*PATH') {
    Invoke-SSH "printf '\n# Lisa: add /home/root/lisa to PATH\nexport PATH=""/home/root/lisa:`$PATH""\n' >> ~/.profile" | Out-Null
}
Write-OK

# ------------------------------------------------------------------
# Step 10: Create start scripts (daemon + agent)
# ------------------------------------------------------------------
Write-Host "[10/$Total] Creating start scripts..."

$daemonScript = @"
#!/bin/sh
cd /home/root/lisa
[ -x /home/root/lisa/bind-hosts.sh ] && /home/root/lisa/bind-hosts.sh 2>/dev/null
export ZEROCLAW_CONFIG_DIR="/home/root/.zeroclaw"
export PATH="/home/root/lisa:`$PATH"
[ -f /home/root/lisa/lisa.env ] && . /home/root/lisa/lisa.env
exec /home/root/lisa/zeroclaw daemon
"@
Write-RemoteFile $daemonScript "${TargetDeployDir}/start-lisa.sh" | Out-Null
Invoke-SSH "chmod +x ${TargetDeployDir}/start-lisa.sh" | Out-Null
Write-Host "  start-lisa.sh (daemon)"

# agent mode - --temperature default(0.7) ignores config, so -t 1.0 is explicit
$agentScript = @"
#!/bin/sh
cd /home/root/lisa
[ -x /home/root/lisa/bind-hosts.sh ] && /home/root/lisa/bind-hosts.sh 2>/dev/null
export ZEROCLAW_CONFIG_DIR="/home/root/.zeroclaw"
export PATH="/home/root/lisa:`$PATH"
[ -f /home/root/lisa/lisa.env ] && . /home/root/lisa/lisa.env
if [ -n "`$1" ]; then
    exec /home/root/lisa/zeroclaw agent -t 1.0 -m "`$*"
else
    exec /home/root/lisa/zeroclaw agent -t 1.0
fi
"@
Write-RemoteFile $agentScript "${TargetDeployDir}/lisa-agent.sh" | Out-Null
Invoke-SSH "chmod +x ${TargetDeployDir}/lisa-agent.sh" | Out-Null
Write-Host "  lisa-agent.sh (agent)"
Write-OK

# ------------------------------------------------------------------
# Step 11: Verify deployment
# ------------------------------------------------------------------
Write-Host "[11/$Total] Verifying deployment..."
$binSize = Invoke-SSH "ls -lh ${TargetDeployDir}/zeroclaw | awk '{print `$5}'"
$cfgSize = Invoke-SSH "ls -lh ${TargetZeroclawDir}/config.toml | awk '{print `$5}'"
$hostsCnt = Invoke-SSH "grep -c tvdevops.openai.azure.com /etc/hosts"
Write-Host "  Binary: $binSize  Config: $cfgSize  Hosts: $hostsCnt entries"
Write-OK

# ------------------------------------------------------------------
# Step 12-14: Post-deploy functional tests
# ------------------------------------------------------------------
Write-Host ""
Write-Host "[12-14/$Total] Running post-deploy tests..."
$TestPass = 0
$TestFail = 0

function Run-Test {
    param([string]$Name, [string]$Result, [int]$ExitCode)
    if ($ExitCode -eq 0 -and $Result) {
        Write-Host "  [PASS] $Name" -ForegroundColor Green
        $script:TestPass++
    } else {
        Write-Host "  [FAIL] $Name" -ForegroundColor Red
        if ($Result) { Write-Host "     $Result" }
        $script:TestFail++
    }
}

# 10) Agent mode: single message
Write-Host ""
Write-Host "  [agent mode]"
$agentResult = Invoke-SSH "/home/root/lisa/lisa-agent.sh hi"
Run-Test "agent: single message" $agentResult $LASTEXITCODE

# 11) Skills
Write-Host ""
Write-Host "  [device-control skill]"
$dcResult = Invoke-SSH "luna-send -n 1 luna://com.webos.applicationManager/getForegroundAppInfo '{}'"
Run-Test "device-control: getForegroundAppInfo" $dcResult $LASTEXITCODE

$volResult = Invoke-SSH "luna-send -n 1 luna://com.webos.service.audio/master/getVolume '{}'"
Run-Test "device-control: getVolume" $volResult $LASTEXITCODE

Write-Host ""
Write-Host "  [weather skill]"
Invoke-SSH "curl -s --max-time 5 -o /dev/null 'http://wttr.in'" | Out-Null
$hasInet = $LASTEXITCODE -eq 0
if ($hasInet) {
    $weatherResult = Invoke-SSH "curl -s --max-time 10 'wttr.in/Seoul?format=%c+%t'"
    Run-Test "weather: wttr.in query" $weatherResult $LASTEXITCODE
} else {
    Write-Host "  [SKIP] weather: target has no internet access"
}

Write-Host ""
Write-Host "  [calendar skill]"
$calCheck = Invoke-SSH "test -x /home/root/lisa/gog && echo ok"
if (-not $calCheck) {
    Write-Host "  [SKIP] calendar: gog not installed"
} else {
    $calEnv = ". /home/root/lisa/lisa.env 2>/dev/null; export PATH=/home/root/lisa:`$PATH"

    $calResult = Invoke-SSH "$calEnv; gog calendar calendars 2>/dev/null | head -5"
    Run-Test "calendar: gog calendar calendars" $calResult $LASTEXITCODE

    $calEvents = Invoke-SSH "$calEnv; gog calendar events primary --from `$(date +%Y-%m-%dT00:00:00) --to `$(date +%Y-%m-%dT23:59:59) --json 2>/dev/null | head -10"
    Run-Test "calendar: today's events" $calEvents $LASTEXITCODE
}

# 12) Daemon mode + gateway tests
Write-Host ""
Write-Host "  [daemon mode]"
Invoke-SSH "cd /home/root/lisa; if [ -x bind-hosts.sh ]; then ./bind-hosts.sh 2>/dev/null; fi; ZEROCLAW_CONFIG_DIR=/home/root/.zeroclaw nohup ./zeroclaw daemon > /tmp/lisa-daemon-test.log 2>&1 &" | Out-Null
Start-Sleep -Seconds 3
$daemonStatus = Invoke-SSH "/home/root/lisa/zeroclaw status"
Run-Test "daemon: zeroclaw status" $daemonStatus $LASTEXITCODE

# Gateway /health
$gwPort = Invoke-SSH "grep '^port' ${TargetZeroclawDir}/config.toml 2>/dev/null | head -1 | sed 's/[^0-9]//g'"
if (-not $gwPort) { $gwPort = "42617" }
$healthResult = Invoke-SSH "curl -s --max-time 5 http://127.0.0.1:${gwPort}/health"
if ($healthResult -match '"status"') {
    Run-Test "gateway: /health (port $gwPort)" $healthResult 0
} else {
    Run-Test "gateway: /health (port $gwPort)" $healthResult 1
}

# Gateway /pair + /api/chat
$pairCode = Invoke-SSH "sed -n 's/.*X-Pairing-Code: *\([0-9]*\).*/\1/p' /tmp/lisa-daemon-test.log | head -1"
if ($pairCode -and $pairCode -match '^\d+$') {
    $pairResult = Invoke-SSH "curl -s -X POST http://127.0.0.1:${gwPort}/pair -H 'X-Pairing-Code: ${pairCode}'"
    if ($pairResult -match '"paired":true') {
        Run-Test "gateway: /pair" "paired (code $pairCode)" 0
        if ($pairResult -match '"token":"([^"]+)"') {
            $gwToken = $Matches[1]
            Write-RemoteFile '{"message":"hi"}' "/tmp/lisa-chat-test.json" | Out-Null
            $chatResult = Invoke-SSH "curl -s --max-time 30 -X POST http://127.0.0.1:${gwPort}/api/chat -H 'Content-Type: application/json' -H 'Authorization: Bearer ${gwToken}' -d @/tmp/lisa-chat-test.json"
            Invoke-SSH "rm -f /tmp/lisa-chat-test.json" | Out-Null
            if ($chatResult -match '"reply"') {
                if ($chatResult -match '"reply":"([^"]{0,60})') {
                    Run-Test "gateway: /api/chat" $Matches[1] 0
                } else {
                    Run-Test "gateway: /api/chat" $chatResult 0
                }
            } else {
                Run-Test "gateway: /api/chat" $chatResult 1
            }
        }
    } else {
        Run-Test "gateway: /pair" $pairResult 1
    }
} else {
    Write-Host "  [SKIP] gateway: failed to extract pairing code"
}

# Cleanup daemon
Invoke-SSH "pkill -f 'zeroclaw daemon'; rm -f /tmp/lisa-daemon-test.log" | Out-Null

# Telegram
Write-Host ""
Write-Host "  [telegram channel]"
$tgToken = Invoke-SSH "grep 'bot_token' ${TargetZeroclawDir}/config.toml 2>/dev/null | sed 's/.*= *""//;s/"".*//' | head -1"
if (-not $tgToken -or $tgToken -eq "YOUR_BOT_TOKEN") {
    Write-Host "  [SKIP] telegram: bot_token not configured"
} elseif (-not $hasInet) {
    Write-Host "  [SKIP] telegram: target has no internet access"
} else {
    $tgResult = Invoke-SSH "curl -s --max-time 10 'https://api.telegram.org/bot${tgToken}/getMe'"
    if ($tgResult -match '"ok":true') {
        Run-Test "telegram: Bot API getMe" $tgResult 0
    } else {
        Run-Test "telegram: Bot API getMe" $tgResult 1
    }
}

# Test summary
Write-Host ""
Write-Host "  ------------------------"
Write-Host "  Results: $TestPass passed / $TestFail failed"
if ($TestFail -gt 0) {
    Write-Host "  WARNING: Some tests failed. Check logs above." -ForegroundColor Yellow
}

# ------------------------------------------------------------------
# Summary
# ------------------------------------------------------------------
$proxyFlag = ""
$proxyLine = ""
if ($ProxyJump) {
    $proxyFlag = " -J ${ProxyJump}"
    $proxyLine = "`n  Proxy:   ${ProxyJump}"
}

Write-Host @"

========================================================
  Lisa target deploy complete!
========================================================

  Target:  ${TargetUser}@${TargetIP}${proxyLine}
  Binary:  ${TargetDeployDir}/zeroclaw
  Config:  ${TargetZeroclawDir}/config.toml

To use:
  ssh${proxyFlag} ${TargetUser}@${TargetIP} '/home/root/lisa/start-lisa.sh'      # daemon
  ssh${proxyFlag} ${TargetUser}@${TargetIP} '/home/root/lisa/lisa-agent.sh'      # agent
  ssh${proxyFlag} ${TargetUser}@${TargetIP} '/home/root/lisa/lisa-agent.sh hi!'  # message
  ssh${proxyFlag} ${TargetUser}@${TargetIP} '/home/root/lisa/zeroclaw status'    # status

"@
