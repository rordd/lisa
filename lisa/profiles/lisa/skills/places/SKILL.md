---
name: places
description: "장소/맛집/카페/병원 검색 + 지도 이미지. Used when user asks about nearby places, restaurants, cafes, or searches for a specific place. Source: Google Maps API."
version: "3.0.0"
---

# Places — 장소검색 + 지도

Google Places API로 장소 검색, Static Maps API로 지도 이미지. shell 도구로 실행.

## When to Use

- "강남역 근처 맛집", "여기 주변 카페"
- "서울역 약국", "홍대 주차장"
- "을지로 골뱅이집", "스타벅스 강남점"
- "지도로 보여줘", "위치 알려줘"

## When NOT to Use

- 길찾기/경로 탐색

## Commands

### 장소 검색 (`search.sh`)
```bash
cd skills/places && sh scripts/search.sh "강남역 맛집"
cd skills/places && sh scripts/search.sh "홍대 카페" 3
cd skills/places && sh scripts/search.sh "서울역 약국" 5 nomap
cd skills/places && sh scripts/search.sh "강남역" 0 maponly
```
Returns `{"url": "https://..."}` — use in A2UI Image component.

### Output
```json
{
  "places": [{"name": "맛있는집", "rating": 4.5, "address": "서울 강남구...", "url": "https://maps.google.com/?cid=..."}],
  "map_url": "https://maps.googleapis.com/maps/api/staticmap?..."
}
```

## Output Format

### Telegram
Text list with name, rating, address, phone.

### WS
A2UI Card with list + map image. Use map.sh with coordinates from search results.
Do NOT use a2web for place display.

### 리뷰 조회
```sh
cd skills/places && sh scripts/review.sh <place_id> [count]
```
search.sh 결과의 `id` 필드를 사용. 기본 5개.

## Notes
- GOOGLE_MAPS_API_KEY 환경변수 필요
- 월 $200 무료 크레딧
