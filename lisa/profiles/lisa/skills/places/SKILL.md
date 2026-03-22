---
name: places
description: "장소/맛집/카페/병원 검색 + 지도 이미지 + 리뷰. Used when user asks about nearby places, restaurants, cafes, or searches for a specific place. Source: Google Maps API."
version: "4.0.0"
---

# Places — 장소검색 + 지도

## When to Use

- "강남역 근처 맛집", "여기 주변 카페"
- "서울역 약국", "스타벅스 강남점"
- "지도로 보여줘", "리뷰 보여줘"

## When NOT to Use

- 길찾기/경로 탐색

## Commands

### 장소 검색
```sh
cd skills/places && sh scripts/search.sh "강남역 맛집"
cd skills/places && sh scripts/search.sh "홍대 카페" 3
cd skills/places && sh scripts/search.sh "서울역" 0 maponly
```

### 리뷰 조회
```sh
cd skills/places && sh scripts/review.sh <place_id> [count]
```
search.sh 결과의 `id` 필드를 사용. 기본 5개.
