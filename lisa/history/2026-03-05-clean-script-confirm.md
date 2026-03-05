# 2026-03-05: clean-target.sh 개별 확인 + tv-control 타겟 TV 설정

## 1. clean-target.sh 개별 확인 프롬프트

### 배경

`clean-target.sh` 실행 시 타겟 초기화와 로컬 토큰 삭제를 한 번에 묻고 있어, 부분 실행이 불가능했음.

### 변경 사항

기존 단일 확인(`Clean target ... and local tokens? [y/N]`)을 두 개의 개별 확인으로 분리:

1. **타겟 초기화**: `Clean target root@<IP>? [y/N]` — 프로세스 종료, hosts/profile 정리, 디렉토리 삭제
2. **로컬 토큰 삭제**: `Remove local gog tokens? [y/N]` — `~/.config/gogcli/keyring/` 삭제

- 둘 다 N이면 취소
- 하나만 Y이면 해당 부분만 실행
- 완료 메시지도 실행된 항목만 표시

## 2. tv-control 스킬 Target TV 설정

### 배경

tv-control 스킬에 타겟 TV 정보(Location, IP)가 추가됨. 배포 시 사용자가 선택할 수 있어야 함.

### 변경 사항

- `SKILL.md`에 Target TV 섹션 추가 (Location: N/A/local/remote, IP)
- 배포 스크립트에 step 7 (Target TV 설정) 추가
  - 사용자가 1) N/A, 2) local, 3) remote 중 선택
  - remote 선택 시 IP 주소 입력
  - 타겟의 `SKILL.md`를 `sed`로 업데이트
- 기존 step 7~12 → step 8~13으로 번호 이동 (TOTAL: 13→14)

## 3. deploy-linux.sh — Ubuntu Linux 배포 스크립트 추가

### 배경

기존 `deploy-target.sh`는 webOS TV (임베디드 리눅스) 전용. Ubuntu Linux에도 배포할 수 있도록 별도 스크립트 필요.

### 변경 사항

- `deploy-linux.sh` 신규 생성 — 로컬/원격 Ubuntu 배포 지원
  - IP 파라미터 없음 → 로컬 설치
  - IP 파라미터 있음 → 원격 SSH 설치
- `run_cmd()` / `copy_file()` 헬퍼로 로컬/원격 투명 처리
- webOS 전용 기능 제거: bind mount, `.profile` hook, luna-send 테스트
- Ubuntu 전용 기능 추가: `cargo build --release` 자동 빌드 제안, `sudo` /etc/hosts 편집, `.bashrc` PATH
- 11단계 배포 (webOS 14단계 대비 간소화)
- 가이드 문서 구조 변경: "Lisa 배포 가이드"로 상위 제목 변경, Linux/webOS 섹션 분리

## 4. clean-linux.sh — Ubuntu Linux 초기화 스크립트 추가

### 배경

`clean-target.sh`는 webOS TV 전용 (bind mount, `.profile` hook 등). Linux 배포 초기화를 위한 별도 스크립트 필요.

### 변경 사항

- `clean-linux.sh` 신규 생성 — 로컬/원격 Ubuntu 초기화 지원
  - IP 파라미터 없음 → 로컬 초기화
  - IP 파라미터 있음 → 원격 SSH 초기화
- `clean-target.sh`와의 차이:
  - bind mount 해제 → `/etc/hosts` sed 직접 삭제 (sudo)
  - `.profile` hook → `.bashrc` PATH 라인 삭제
  - 로컬 토큰 삭제는 원격 모드에서만 물어봄 (로컬 모드에서는 gog 디렉토리가 배포 파일에 포함)

## 5. x86_64 빌드 및 멀티 아키텍처 지원

### 배경

기존 release 바이너리는 ARM64만 있었음. Intel/AMD Linux에도 배포하려면 x86_64 바이너리 필요.

### 변경 사항

- `lisa/release/x86_64/` 디렉토리 추가 (zeroclaw 24MB, gog 23MB)
- zeroclaw: `cargo build --release --target x86_64-unknown-linux-gnu` (dynamically linked)
- gog: `CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w"` (statically linked)
- `deploy-linux.sh` 아키텍처 감지 로직 변경:
  - 원격 설치 시 타겟의 `uname -m`으로 감지 (기존: 로컬 머신 기준)
  - release 바이너리 → cargo build output → 빌드 제안 순서로 탐색
  - 크로스 아키텍처 원격 설치 시 release 바이너리 필수

## 수정 파일

| 파일 | 변경 |
|------|------|
| `lisa/scripts/clean-target.sh` | 단일 확인 → 타겟/로컬 개별 확인으로 분리 |
| `lisa/profiles/lisa/skills/tv-control/SKILL.md` | Target TV 섹션 추가 (N/A/local/remote) |
| `lisa/scripts/deploy-target.sh` | step 7 Target TV 설정 추가, step 번호 조정 |
| `lisa/scripts/deploy-target.ps1` | 동일 변경 |
| `lisa/scripts/deploy-linux.sh` | 신규 + 멀티 아키텍처 지원 |
| `lisa/scripts/clean-linux.sh` | 신규 — Ubuntu Linux 초기화 스크립트 |
| `lisa/docs/deploy-linux-guide.md` | 신규 — Linux 배포 가이드 (아키텍처 섹션 추가) |
| `lisa/docs/deploy-target-guide.md` | webOS TV 전용으로 복원 |
| `lisa/release/x86_64/zeroclaw` | 신규 — x86_64 바이너리 |
| `lisa/release/x86_64/gog` | 신규 — x86_64 gog 바이너리 |
| `lisa/config/config.linux.toml` | 신규 — Linux 전용 config (config.arm64.toml 기반) |
| `lisa/docs/setup-guide.md` | 프로젝트 구조에 config.linux.toml 추가 |
