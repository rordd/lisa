---
name: blog
description: "블로그 검색/리뷰 조회. Used when user asks for blog reviews, product reviews, travel reviews, restaurant reviews from blogs. Source: Naver Blog API."
version: "1.0.0"
channels: ws, telegram
---

# Blog — 블로그 리뷰 검색

## When to Use

- "에어팟 프로 리뷰", "맥북 에어 후기"
- "제주도 여행 블로그", "강남 맛집 리뷰"
- "블로그에서 찾아줘"

## When NOT to Use

- 장소 검색 (→ places), 상품 가격 (→ shopping)

## Commands

```sh
cd skills/blog && sh scripts/search.sh "에어팟 프로 리뷰"
cd skills/blog && sh scripts/search.sh "제주도 여행" 5 date
```
sort: sim(관련도), date(최신). 기본 sim.
