---
name: weather
description: "현재 날씨와 예보를 wttr.in으로 조회. 사용자가 날씨, 기온, 비/눈 여부를 물어볼 때 사용."
version: "1.0.0"
always: true
---

# 날씨 스킬

wttr.in API로 날씨를 조회한다. API 키 불필요.

## 사용 시점

- "날씨 어때?"
- "오늘 비 와?"
- "내일 기온은?"
- "이번 주 날씨"

## 기본 위치

USER.md의 날씨 위치를 참조. 위치를 안 말하면 기본 위치 사용.

## 명령어

### 현재 날씨
```bash
curl -s "wttr.in/서울+강서구?format=%l:+%c+%t+(체감+%f),+바람+%w,+습도+%h&lang=ko"
```

### 오늘/내일/모레
```bash
# 오늘
curl -s "wttr.in/서울+강서구?0&lang=ko"

# 내일
curl -s "wttr.in/서울+강서구?1&lang=ko"

# 3일 예보
curl -s "wttr.in/서울+강서구?lang=ko"
```

### JSON (파싱용)
```bash
curl -s "wttr.in/서울+강서구?format=j1&lang=ko"
```

### 다른 도시
```bash
curl -s "wttr.in/부산?format=3&lang=ko"
curl -s "wttr.in/New+York?format=3"
```

## 포맷 코드
- `%c` — 날씨 이모지
- `%t` — 기온
- `%f` — 체감 온도
- `%w` — 바람
- `%h` — 습도
- `%p` — 강수량

## 주의
- 너무 자주 호출하지 말 것 (rate limit)
- 한국 도시는 한글로 검색 가능
