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

경로: `scripts/`

### 장소 검색 (`search.sh`)
```bash
shell search.sh "강남역 맛집"
shell search.sh "홍대 카페" 3
```

### 지도 이미지 (`map.sh`)
```bash
shell map.sh 37.495,127.028
shell map.sh 37.495,127.028 37.500,127.024
shell map.sh 37.495,127.028 12
```
Returns `{"url": "https://..."}` — use in A2UI Image component.

### Output (search.sh)
```json
[{
  "name": "맛있는집",
  "type": "korean_restaurant",
  "address": "서울 강남구...",
  "phone": "+82 2-555-1234",
  "rating": 4.5,
  "reviews": 230,
  "url": "https://maps.google.com/?cid=...",
  "lat": 37.495,
  "lng": 127.028
}]
```

## Output Format

### Telegram
Text list with name, rating, address, phone.

### WS
A2UI Card with list + map image. Use map.sh with coordinates from search results.
Do NOT use a2web for place display.

## Notes
- GOOGLE_MAPS_API_KEY 환경변수 필요
- 월 $200 무료 크레딧
