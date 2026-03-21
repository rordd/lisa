---
name: directions
description: "Global directions/routes. Used when user asks for directions outside Korea — driving, transit, walking, bicycling. For Korean routes use directions-kr."
version: "1.0.0"
channels: ws, telegram
---

# Directions — 글로벌 길찾기

Google Directions API. 해외 자동차/대중교통/도보/자전거 전부 지원.

## When to Use

- "Times Square에서 Central Park 어떻게 가"
- "도쿄역에서 시부야", "파리 에펠탑에서 루브르"
- 해외 여행 교통편

## When NOT to Use

- 한국 내 길찾기 → directions-kr 스킬
- 장소 검색 → places 스킬

## Commands

### 경로 검색 (`route.sh`)
```bash
cd skills/directions && bash scripts/route.sh "Times Square NYC" "Central Park NYC"           # 자동차 (기본)
cd skills/directions && bash scripts/route.sh "Tokyo Station" "Shibuya Station" transit        # 대중교통
cd skills/directions && bash scripts/route.sh "Eiffel Tower" "Louvre Museum" walk              # 도보
cd skills/directions && bash scripts/route.sh "Times Square NYC" "Central Park NYC" all        # 자동차+대중교통
```

## Links

결과에 항상 Google Maps 링크 포함:
```
https://www.google.com/maps/dir/origin/destination
```

## Notes
- GOOGLE_MAPS_API_KEY 환경변수 필요
- 한국 자동차/도보는 ZERO_RESULTS 반환 → directions-kr 사용
- shell 호출 시 `2>&1` 리다이렉션 사용 금지
