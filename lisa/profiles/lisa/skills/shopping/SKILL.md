---
name: shopping
description: "상품 검색/가격 비교. Used when user asks about product prices, shopping, cheapest price, or product comparison. Source: Naver Shopping API."
version: "1.0.0"
channels: ws, telegram
---

# Shopping — 상품 검색/가격 비교

## When to Use

- "에어팟 최저가", "레고 가격"
- "아이패드 얼마야", "가격 비교해줘"
- "추천 키보드", "맥북 에어 가격"

## When NOT to Use

- 실제 구매/결제
- 해외 직구 가격 (국내 쇼핑몰만)

## Commands

### 상품 검색
```sh
cd skills/shopping && sh scripts/search.sh "에어팟 프로"
cd skills/shopping && sh scripts/search.sh "레고 테크닉" 5
cd skills/shopping && sh scripts/search.sh "맥북 에어" 5 asc
```
sort: sim(관련도), date(최신), asc(가격↑), dsc(가격↓). 기본 sim.
