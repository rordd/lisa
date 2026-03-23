---
name: directions-kr
description: "한국 길찾기 (자동차/대중교통/실시간 교통). Used when user asks for directions within Korea — driving routes, transit routes, commute time, taxi fare, real-time traffic conditions (막힘/원활). Korean locations only."
version: "1.0.0"
channels: ws, telegram
---

# Directions-KR — 한국 길찾기

자동차: 카카오 모빌리티 API (실시간 교통, 택시비, 톨비)
대중교통: Google Routes API (지하철/버스 환승)

## When to Use

- "강남역에서 서울역 어떻게 가", "길 알려줘"
- "차로 몇 분?", "택시비 얼마야"
- "지하철로 어떻게 가", "대중교통 알려줘"
- "실시간 교통 어때?", "길 막혀?" → drive.sh 결과에 traffic 포함

## When NOT to Use

- 해외 길찾기 → directions 스킬
- 장소 검색 → places 스킬

## Commands

### 길찾기 (`route.sh`)
```sh
cd skills/directions-kr && sh scripts/route.sh "강남역" "서울역"           # 자동차 (기본)
cd skills/directions-kr && sh scripts/route.sh "강남역" "서울역" transit   # 대중교통
cd skills/directions-kr && sh scripts/route.sh "강남역" "서울역" all       # 자동차+대중교통
```

Output: 배열로 반환
```json
[
  {"mode": "drive", "duration_min": 19, "taxi_fare": 17900, "toll": 0, "traffic_summary": [...]},
  {"mode": "transit", "duration": "2000s", "steps": [{"line": "2호선", "from": "강남", "to": "사당", "stops": 5}]}
]
```

## Output Format

### Telegram
```
🚗 강남역 → 서울역 (자동차)
⏱️ 19분 | 11.4km
💰 택시 17,900원 | 톨비 0원
🚦 교통: 원활 4구간 / 서행 3구간 / 막힘 1구간

🚇 강남역 → 서울역 (대중교통)
⏱️ 33분 | 15.4km
1. 🟢 2호선 강남→사당 (9분, 5정거장)
2. 🔵 4호선 사당→서울역 (16분, 8정거장)
```

## Links & Map

결과에 항상 Google Maps 링크 포함:
```
https://www.google.com/maps/dir/출발지/도착지
```

지도 이미지가 필요하면 places 스킬의 map.sh 사용 (geocode 좌표 활용).

## Notes
- 자동차: KAKAO_REST_API_KEY 필요, 좌표(x,y) 입력
- 대중교통: GOOGLE_MAPS_API_KEY 필요, 주소명 입력
- 사용자가 모드 안 정하면 자동차+대중교통 둘 다 보여주기
