# 2026-03-04: 배포 스크립트 개선

## 배경

Windows에서 deploy-target.ps1 실행 시 여러 문제 발생:
- PowerShell 5.x에서 `&&`/`||` 연산자 파서 에러
- `$ErrorActionPreference = "Stop"`으로 SSH stderr가 terminating exception 발생
- 한글/이모지가 PowerShell 콘솔에서 깨짐
- ProxyJump 환경에서 SCP 전송 실패 (SFTP 프로토콜 비호환)
- JSON 이스케이프가 PowerShell → SSH → remote shell 다중 이스케이프 과정에서 깨짐

## 변경 사항

### 1. PicoClaw 패턴 리팩토링 (deploy-target.ps1)

PicoClaw의 `setup_picoclaw.ps1`을 참고하여 전면 리팩토링:

- `Invoke-SSH`: `& ssh.exe @script:SSHOpts` + `2>$null`
- `Invoke-SCP`: `& scp.exe -O @script:SSHOpts` (legacy SCP protocol)
- `Invoke-SCPRecursive`: SCP recursive 전송
- `Write-RemoteFile`: 로컬 임시파일 → UTF-8 NoBOM/LF 인코딩 → SCP 전송
- `Write-OK`/`Write-Fail`: 색상 출력 헬퍼
- `$ErrorActionPreference = "Stop"` 제거
- `#Requires -Version 5.1` 제거
- `[CmdletBinding()]` param 블록 추가

### 2. ProxyJump 지원 (deploy-target.ps1)

```powershell
[CmdletBinding()]
param(
    [string]$TargetIP,
    [string]$ProxyJump
)

$SSHOpts = @("-o", "ConnectTimeout=10", "-o", "StrictHostKeyChecking=accept-new")
if ($ProxyJump) {
    $SSHOpts += @("-J", $ProxyJump)
}
```

- 모든 SSH/SCP 호출에서 `@script:SSHOpts` splatting
- Summary 출력에 Proxy 정보 표시
- `.bat`은 `%*`로 모든 인자 전달하므로 변경 불필요

### 3. SCP `-O` 플래그

OpenSSH 9.0+에서 SCP가 SFTP 프로토콜을 기본으로 사용하여 ProxyJump 환경에서 전송 실패. `-O` 플래그로 legacy SCP 프로토콜 강제:

```powershell
function Invoke-SCP {
    param([string]$LocalPath, [string]$RemotePath)
    & scp.exe -O @script:SSHOpts $LocalPath "${TargetUser}@${TargetIP}:${RemotePath}"
    return ($LASTEXITCODE -eq 0)
}
```

### 4. Chat 테스트 JSON 이스케이프 우회

PowerShell → SSH → remote shell 다중 이스케이프 문제:
- `'{""message"":""hi""}'` → 실패
- `'{`"message`":`"hi`"}'` → 실패
- `echo '{"message":"hi"}' > /tmp/file` via SSH → 실패

최종 해결: `Write-RemoteFile`로 JSON 파일을 로컬에서 생성 후 SCP 전송:

```powershell
Write-RemoteFile '{"message":"hi"}' "/tmp/lisa-chat-test.json" | Out-Null
$chatResult = Invoke-SSH "curl -s ... -d @/tmp/lisa-chat-test.json"
```

### 5. 한글/이모지 → 영문/ASCII 변환

모든 배포 스크립트(`.sh`, `.ps1`, `.bat`)의 한글 텍스트와 이모지를 영문/ASCII로 변환.

### 6. 단계별 진행 표시 (deploy-target.sh)

`TOTAL=12` 변수와 `[1/$TOTAL]` ~ `[11-12/$TOTAL]` 형식의 진행 표시 추가.

## 수정 파일

| 파일 | 변경 |
|------|------|
| `lisa/scripts/deploy-target.ps1` | 전면 리팩토링 |
| `lisa/scripts/deploy-target.sh` | 영문화, 단계 표시 |
| `lisa/scripts/deploy-target.bat` | 영문화 |
| `lisa/docs/deploy-target-guide.md` | 트러블슈팅 업데이트 |

## 테스트 결과

ProxyJump 경유 배포: 8/8 pass (2 skip)

```
  [PASS] agent: single message
  [PASS] device-control: getForegroundAppInfo
  [PASS] device-control: getVolume
  [PASS] weather: wttr.in query
  [SKIP] calendar: gog not installed
  [PASS] daemon: zeroclaw status
  [PASS] gateway: /health (port 42617)
  [PASS] gateway: /pair
  [PASS] gateway: /api/chat
  [SKIP] telegram: bot_token not configured
```
