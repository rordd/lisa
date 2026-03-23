---
name: news
description: "뉴스 검색. Used when user asks for news, latest news, breaking news about a topic. Source: Naver News API."
version: "1.0.0"
channels: ws, telegram
---

# News — 뉴스 검색

## When to Use

- "AI 뉴스", "삼성전자 뉴스"
- "오늘 뉴스", "최신 뉴스 알려줘"
- "부동산 뉴스", "코스피 뉴스"

## When NOT to Use

- 종목별 주식 뉴스 (→ stock news.sh)

## Commands

```sh
cd skills/news && sh scripts/search.sh "AI"
cd skills/news && sh scripts/search.sh "삼성전자" 5 date
```
sort: sim(관련도), date(최신). 기본 date.
