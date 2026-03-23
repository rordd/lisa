---
name: stock
description: "한국 주식 시세/지수/뉴스 조회. Used when user asks about stock prices, KOSPI/KOSDAQ index, or stock news. Source: Naver Finance API."
version: "2.0.0"
channels: ws, telegram
---

# Stock — 한국 주식

## When to Use

- "삼성전자 주가", "SK하이닉스 얼마야"
- "코스피 지수", "오늘 시장 어때"
- "삼성전자 뉴스", "관심종목 보여줘"

## When NOT to Use

- 해외 주식, 매매/주문, 과거 차트

## Commands

### 종목 시세
```sh
cd skills/stock && sh scripts/quote.sh 삼성전자
cd skills/stock && sh scripts/quote.sh 삼성전자 SK하이닉스
```

### 시장 지수
```sh
cd skills/stock && sh scripts/market.sh
cd skills/stock && sh scripts/market.sh KOSPI
```

### 관심종목
```sh
cd skills/stock && sh scripts/watchlist.sh
cd skills/stock && sh scripts/watchlist.sh add 삼성전자
cd skills/stock && sh scripts/watchlist.sh remove SK하이닉스
```

### 종목 뉴스
```sh
cd skills/stock && sh scripts/news.sh 삼성전자
cd skills/stock && sh scripts/news.sh 005930 3
```

## Notes
- 종목코드(005930) 또는 한국어 종목명 모두 지원
- quote.sh 결과에 charts.day, charts.candleDay 등 차트 이미지 URL 포함
