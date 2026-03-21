---
name: stock
description: "한국 주식 시세/지수/뉴스 조회. Used when user asks about stock prices, KOSPI/KOSDAQ index, or stock news. Source: Naver Finance API."
version: "1.0.0"
channels: ws, telegram
---

# Stock — 한국 주식

한국 주식 시세, 시장 지수, 종목 뉴스를 조회한다. shell 도구로 스크립트를 실행한다.

## When to Use

✅ **USE this skill when:**
- "삼성전자 주가", "SK하이닉스 얼마야"
- "코스피 지수", "오늘 시장 어때"
- "삼성전자 뉴스", "네이버 관련 소식"
- "주식 시세 알려줘"
- "관심종목 보여줘", "삼성전자 관심종목에 추가해"

## When NOT to Use

❌ **DON'T use this skill when:**
- 해외 주식 (미국, 일본 등) → 다른 도구 필요
- 매매/주문 → 이 스킬은 조회 전용
- 과거 차트 데이터 → 별도 도구 필요

## Scripts

경로: `scripts/`

### 종목 시세 (`quote.sh`)
```bash
# 종목코드로 조회
shell quote.sh 005930

# 종목명으로 조회 (자동 코드 변환)
shell quote.sh 삼성전자

# 여러 종목 동시 조회
shell quote.sh 삼성전자 SK하이닉스 네이버
```

Output:
```json
{
  "code": "005930",
  "name": "삼성전자",
  "market": "코스피",
  "price": "72,300",
  "change": "+900",
  "changePercent": "1.26",
  "direction": "상승",
  "status": "CLOSE",
  "tradedAt": "2026-03-20T16:10:20+09:00"
}
```

### 시장 지수 (`market.sh`)
```bash
# 코스피 + 코스닥
shell market.sh

# 코스피만
shell market.sh KOSPI

# 코스닥만
shell market.sh KOSDAQ
```

Output:
```json
{
  "index": "코스피",
  "price": "2,650.12",
  "change": "+15.30",
  "changePercent": "0.58",
  "direction": "상승",
  "status": "CLOSE"
}
```

### 관심종목 (`watchlist.sh`)
```bash
# 관심종목 전체 시세 조회
shell watchlist.sh

# 종목 추가
shell watchlist.sh add 삼성전자

# 종목 삭제
shell watchlist.sh remove SK하이닉스

# 목록만 (시세 없이)
shell watchlist.sh list
```

### 종목 뉴스 (`news.sh`)
```bash
# 최근 뉴스 5건 (기본)
shell news.sh 005930

# 3건만
shell news.sh 005930 3
```

Output:
```json
[
  {
    "title": "삼성전자, 1분기 실적 전망...",
    "source": "한국경제",
    "date": "202603211500",
    "url": "https://n.news.naver.com/mnews/article/..."
  }
]
```

## Output Format

### 텔레그램
```
📈 삼성전자 (005930) — 코스피
💰 72,300원 (+900, +1.26%)
📊 장 마감 | 2026-03-20 16:10
```

### WS
시세 데이터를 시각적으로 표시.

## Charts

quote.sh 결과에 네이버 차트 이미지 URL 포함:
- `charts.day` — 일봉 라인
- `charts.month3` — 3개월
- `charts.year` — 1년
- `charts.candleDay` — 일봉 캔들
- `charts.candleWeek` — 주봉 캔들

차트 이미지를 시각적으로 표시하거나 URL로 전송.

## Notes
- 장 운영시간: 09:00-15:30 KST
- status가 CLOSE면 전일 종가, OPEN이면 실시간
- 네이버 금융 API 기반 (비공식, rate limit 주의)
- 종목코드 6자리 또는 한국어 종목명 모두 지원
