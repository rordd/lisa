---
name: movie
description: Korean box office rankings and movie search. Use for "박스오피스", "흥행순위", "영화 검색", "영화 정보", movie rankings, film search.
channels: telegram,ws,lisa
---

## Tools

### boxoffice.sh
Daily or weekly Korean box office rankings from KOBIS.
- Usage: `cd skills/movie && sh scripts/boxoffice.sh [daily|weekly] [YYYYMMDD]`
- Default: daily, yesterday
- Returns: rank, title, audience count, release date

### search.sh
Search movies by title from KOBIS.
- Usage: `cd skills/movie && sh scripts/search.sh "영화제목"`
- Returns: title, director, genre, release date, movie code

### detail.sh
Movie detail info by KOBIS movie code.
- Usage: `cd skills/movie && sh scripts/detail.sh "영화코드"`
- Returns: title, director, actors, genre, runtime, rating, audit info
