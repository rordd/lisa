# gogcli용 Google Cloud Console OAuth 설정 가이드

gogcli를 사용하려면 Google Cloud Console에서 OAuth 2.0 클라이언트 JSON 파일을 생성해야 합니다.

---

## 1단계: Google Cloud Console에서 프로젝트 생성

1. 브라우저에서 [Google Cloud Console](https://console.cloud.google.com/)을 엽니다.
2. Google 계정으로 로그인합니다.
3. 상단 바의 프로젝트 선택 드롭다운을 클릭합니다.
4. 팝업 우측 상단의 **"새 프로젝트"**를 클릭합니다.
5. 프로젝트 이름을 입력합니다 (예: `gogcli-project`).
6. 조직은 기본값으로 두고 **"만들기"**를 클릭합니다.
7. 프로젝트가 생성되면 상단 알림에서 선택합니다.

---

## 2단계: API 활성화

1. 왼쪽 메뉴에서 **"API 및 서비스"** → **"라이브러리"**를 클릭합니다.
2. 필요한 API를 검색하고 **"사용 설정"**을 클릭합니다.

| 서비스 | API 이름 |
|--------|----------|
| Gmail | Gmail API |
| Google 캘린더 | Google Calendar API |
| Google 드라이브 | Google Drive API |
| Google 스프레드시트 | Google Sheets API |
| Google 문서 | Google Docs API |
| Google 연락처 | People API |
| Google Tasks | Tasks API |
| Google 프레젠테이션 | Google Slides API |

> **팁:** 모든 API를 활성화할 필요 없습니다. 사용할 서비스만 활성화하세요.

---

## 3단계: OAuth 동의 화면 구성

1. **"API 및 서비스"** → **"OAuth 동의 화면"**을 클릭합니다.
2. **"시작하기"**를 클릭합니다.

### 3-1. 앱 정보

| 항목 | 값 |
|------|-----|
| 앱 이름 | `gogcli` (또는 원하는 이름) |
| 사용자 지원 이메일 | 본인 Gmail 선택 |

### 3-2. 사용자 유형

- 개인 Gmail: **"외부"** 선택
- Google Workspace: **"내부"** 선택 가능

### 3-3. 연락처 정보

- 개발자 연락처 이메일을 입력합니다.

### 3-4. 완료

- Google API 서비스 사용자 데이터 정책에 동의하고 **"만들기"**를 클릭합니다.

---

## 4단계: 테스트 사용자 등록

"외부"를 선택한 경우 **"테스트"** 상태로 생성됩니다. 등록된 테스트 사용자만 인증할 수 있습니다.

1. **"Google Auth Platform"** → **"대상 사용자"**로 이동합니다.
2. **"사용자 추가"**를 클릭합니다.
3. gogcli에서 사용할 Gmail 주소를 입력합니다.
4. **"저장"**을 클릭합니다.

> **중요:** 테스트 사용자로 등록하지 않으면 `403: access_denied` 오류가 발생합니다.

> **참고:** 테스트 모드에서는 최대 100명, 인증 토큰은 7일 후 만료됩니다. 개인 사용에는 충분합니다.

---

## 5단계: OAuth 클라이언트 ID 생성 및 JSON 다운로드

1. **"API 및 서비스"** → **"사용자 인증 정보"**를 클릭합니다.
2. 상단의 **"+ 사용자 인증 정보 만들기"**를 클릭합니다.
3. **"OAuth 클라이언트 ID"**를 선택합니다.
4. 애플리케이션 유형으로 **"데스크톱 앱"**을 선택합니다.
   - **반드시 "데스크톱 앱"**을 선택하세요. "웹 애플리케이션"이 아닙니다.
5. 이름을 입력합니다 (예: `gogcli-desktop`).
6. **"만들기"**를 클릭합니다.
7. **"JSON 다운로드"** 버튼을 클릭합니다.
   - 파일명: `client_secret_XXXXXXXXXXXX.json`

> **보안 주의:** 다운로드한 JSON 파일에 클라이언트 시크릿이 포함되어 있습니다. Git에 커밋하지 마세요.

---

## 6단계: gogcli에 OAuth 인증 등록

```bash
# 1. JSON 파일 등록
gog auth credentials ~/Downloads/client_secret_XXXXXXXXXXXX.json

# 2. Google 계정 인증 추가
gog auth add you@gmail.com

# 3. 브라우저가 열리면 Google에 로그인하고 권한을 부여합니다
#    "이 앱은 확인되지 않았습니다" 경고가 나오면:
#    "고급" → "gogcli(안전하지 않음)로 이동" 클릭

# 4. 기본 계정 설정 (매번 --account 입력 생략)
export GOG_ACCOUNT=you@gmail.com

# 5. 동작 확인
gog gmail labels list

# 6. 캘린더 ID 목록 확인 (USER.md 설정용)
gog calendar calendars -a you@gmail.com
```

---

## 임베디드 리눅스(BusyBox) 추가 설정

브라우저가 없는 환경에서는 다음 방법을 사용합니다.

### 방법 A: 개발 호스트에서 인증 후 설정 파일 복사 (권장)

1. 데스크톱/노트북에서 6단계를 완료합니다.
2. 설정 디렉토리를 보드에 복사합니다:

```bash
# 설정 디렉토리 위치:
# Linux: ~/.config/gogcli/
# macOS: ~/Library/Application Support/gogcli/

scp -r ~/.config/gogcli/ root@<보드IP>:~/.config/gogcli/
```

3. 보드에서 키링 백엔드를 파일 모드로 설정합니다:

```bash
export GOG_KEYRING_BACKEND=file
export GOG_KEYRING_PASSWORD='비밀번호'
export GOG_ACCOUNT=you@gmail.com
```

### 방법 B: 수동/헤드리스 인증

```bash
# 보드에서 실행
gog auth add you@gmail.com --manual

# 터미널에 표시된 URL을 복사
# 데스크톱 브라우저에서 해당 URL을 열고 인증 완료
# 리다이렉트 URL을 복사하여 보드 터미널에 붙여넣기
```

---

## 자주 발생하는 문제

### `403: access_denied` 오류
- **원인:** 테스트 사용자로 등록되지 않은 이메일로 인증 시도
- **해결:** 4단계에서 이메일을 테스트 사용자로 추가

### "이 앱은 확인되지 않았습니다" 경고
- **정상입니다.** 개인 프로젝트이므로 Google 인증이 필요 없습니다.
- **"고급"** → **"gogcli(안전하지 않음)로 이동"** 클릭

### 토큰 만료 (7일 후)
- 테스트 모드에서는 7일마다 토큰이 만료됩니다.
- `gog auth add you@gmail.com --force-consent`로 재인증

### 새 서비스 추가 후 권한 오류
- `--force-consent` 플래그로 재인증:
```bash
gog auth add you@gmail.com --services sheets --force-consent
```

### 임베디드 환경에서 키링 오류
- OS 키링이 없는 환경에서는 파일 백엔드를 사용:
```bash
export GOG_KEYRING_BACKEND=file
export GOG_KEYRING_PASSWORD='아무_비밀번호'
```

---

## 전체 흐름 요약

```
Google Cloud Console 접속
        |
   프로젝트 생성
        |
   필요한 API 활성화 (Gmail, Calendar, Drive 등)
        |
   OAuth 동의 화면 구성 (외부, 앱 이름 입력)
        |
   테스트 사용자 추가 (본인 이메일)
        |
   사용자 인증 정보 → OAuth 클라이언트 ID 생성 (데스크톱 앱)
        |
   JSON 파일 다운로드 (client_secret_xxx.json)
        |
   gog auth credentials <json파일>
        |
   gog auth add <이메일>
        |
   사용 준비 완료!
```
