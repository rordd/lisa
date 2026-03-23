---
name: youtube
description: "유튜브 영상 검색. Used when user asks for YouTube videos, video recommendations, tutorials, or video reviews. Source: YouTube Data API v3."
version: "1.0.0"
channels: ws, telegram
---

# YouTube — 영상 검색

## When to Use

- "레고 리뷰 영상", "AI 뉴스 영상"
- "유튜브에서 찾아줘", "영상 추천해줘"
- "요리 레시피 영상", "운동 영상"

## When NOT to Use

- 음악 재생 (재생 불가, 링크만 제공)

## Commands

```sh
cd skills/youtube && sh scripts/search.sh "레고 테크닉 리뷰"
cd skills/youtube && sh scripts/search.sh "AI news" 5 date
cd skills/youtube && sh scripts/search.sh "맥북 리뷰" 3 viewCount
```
order: relevance(관련도), date(최신), viewCount(조회수). 기본 relevance.
