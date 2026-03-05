# 2026-03-03: agent 모드 지원 추가

## Summary

타겟에서 daemon 모드 외에 agent 모드(인터랙티브 채팅/1회성 메시지)도 사용할 수 있도록 배포 스크립트 및 가이드 업데이트.

## 배경

- ZeroClaw는 `daemon`(백그라운드 서비스)과 `agent`(대화형/1회성) 두 가지 실행 모드 지원
- 기존 배포 스크립트는 `start-lisa.sh` (daemon 모드)만 생성
- 타겟에서 직접 대화하거나 1회성 명령을 보내려면 agent 모드 필요

## 실행 모드 비교

| 모드 | 스크립트 | 용도 |
|------|----------|------|
| daemon | `start-lisa.sh` | 백그라운드 서비스 (gateway + 채널 + 스케줄러) |
| agent | `lisa-agent.sh` | 인터랙티브 채팅 또는 1회성 메시지 실행 |

## Changes

### Modified

| # | Path | Description |
|---|---|---|
| 1 | `lisa/scripts/deploy-target.sh` | `lisa-agent.sh` 생성 로직 추가, 배포 완료 메시지에 agent 모드 안내 추가 |
| 2 | `lisa/scripts/deploy-target.ps1` | 동일 변경 |
| 3 | `lisa/docs/deploy-target-guide.md` | daemon/agent 모드 비교 표, agent 실행 예시, 디렉토리 구조에 `lisa-agent.sh` 추가. config 예시도 수정된 스키마 반영 (`[memory]` backend, `[security.sandbox]`) |

### 타겟에 추가되는 파일

| 파일 | 설명 |
|------|------|
| `/home/root/lisa/lisa-agent.sh` | agent 모드 시작 스크립트. 인자 없으면 인터랙티브, 인자 있으면 1회성 메시지 |
