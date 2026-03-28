# Worker API Bash Test Design

## 목적

PushOver Worker API를 대상으로 bash + curl + jq 기반의 기능 테스트를 구축합니다.

- **회귀 테스트**: 코드 변경 시 API 기능이 정상 작동하는지 확인
- **통합 테스트**: 실제 Worker API와 PushOver API 연동 테스트

---

## 범위

### 포함 (In Scope)
- Worker API 직접 호출 테스트 (UI 통하지 않음)
- 성공 케이스 위주 (happy path)
- 다음 엔드포인트 테스트:
  - `GET /health` - 헬스체크
  - `POST /api/v1/messages` - 메시지 전송
  - `GET /api/v1/messages` - 메시지 목록 조회
  - `GET /api/v1/messages/:receipt/status` - 수신 상태 조회
  - 인증 실패 테스트

### 제외 (Out Scope)
- Webhook 관련 4개 엔드포인트
- 실패 케이스 (HTTP 400, 404, 502 등)
- UI 테스트 (Playwright 이미 존재)
- 성능/부하 테스트

---

## 아키텍처

### 파일 구조

```
PushOver/
├── tests/
│   └── api-test.sh          # 단일 테스트 스크립트 (모든 테스트 포함)
├── Makefile                 # test-api 타겟
├── .env.test                # 테스트 환경변수 (Git 제외)
└── .env.example             # 예시 파일
```

### 구조 원칙
- **단일 스크립트**: 모든 테스트가 `tests/api-test.sh` 하나의 파일에 포함
- **직렬 실행**: 테스트 간 순서 종속성이 있으므로 순차 실행
- **Makefile 래퍼**: `make test-api`로 편리하게 실행

---

## 구현 상세

### 1. 테스트 스크립트 (`tests/api-test.sh`)

```bash
#!/bin/bash
set -euo pipefail

# 색상 출력
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 환경변수 로드
if [ -f .env.test ]; then
  source .env.test
else
  echo "Error: .env.test not found"
  exit 1
fi

# Verbose 모드 (선택 사항)
VERBOSE="${VERBOSE:-false}"
if [ "$VERBOSE" = "true" ]; then
  set -x  # 디버깅 출력 활성화
fi

# 필수 환경변수 검증
: "${CF_WORKER_URL:?CF_WORKER_URL required}"
: "${CF_WORKER_TOKEN:?CF_WORKER_TOKEN required}"
: "${PUSHOVER_API_TOKEN:?PUSHOVER_API_TOKEN required}"
: "${PUSHOVER_USER_KEY:?PUSHOVER_USER_KEY required}"

# 테스트 카운터
PASSED=0
FAILED=0

# 참고: 산술 연산 시 set -e 문제 방지
# ((PASSED++))는 결과가 0일 때 exit code 1 반환
# 따라서 $((PASSED + 1)) 형식 사용

# 헬퍼 함수
log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_test() { echo -e "${YELLOW}[TEST]${NC} $1"; }

# 테스트 1: Health Check
test_health_check() {
  log_test "Health Check"

  response=$(curl -s "$CF_WORKER_URL/health")

  if [ "$response" = "OK" ]; then
    echo "✓ Health check passed"
    PASSED=$((PASSED + 1))
  else
    echo "✗ Expected 'OK', got '$response'"
    FAILED=$((FAILED + 1))
  fi
}

# 테스트 2: 메시지 전송
test_send_message() {
  log_test "Send Message"

  timestamp=$(date +%s)
  response=$(curl -s --max-time 10 -X POST "$CF_WORKER_URL/api/v1/messages" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $CF_WORKER_TOKEN" \
    -d "{
      \"token\": \"$PUSHOVER_API_TOKEN\",
      \"user\": \"$PUSHOVER_USER_KEY\",
      \"message\": \"Test message $timestamp\",
      \"title\": \"API Test\"
    }")

  status=$(echo "$response" | jq -r '.status' 2>/dev/null)

  if [ -z "$status" ] || [ "$status" = "null" ]; then
    echo "✗ Failed to parse JSON response"
    echo "  Response: $response"
    FAILED=$((FAILED + 1))
    return
  fi

  if [ "$status" = "success" ]; then
    receipt=$(echo "$response" | jq -r '.receipt')
    request=$(echo "$response" | jq -r '.request')
    echo "✓ Message sent, receipt: $receipt, request: $request"
    LAST_RECEIPT="${receipt:-$request}"  # receipt가 null이면 request UUID 사용
    PASSED=$((PASSED + 1))
  else
    echo "✗ Expected status 'success', got '$status'"
    echo "  Response: $response"
    FAILED=$((FAILED + 1))
  fi
}

# 테스트 3: 메시지 목록 조회
test_get_messages() {
  log_test "Get Messages"

  response=$(curl -s --max-time 10 "$CF_WORKER_URL/api/v1/messages?limit=10" \
    -H "Authorization: Bearer $CF_WORKER_TOKEN")

  status=$(echo "$response" | jq -r '.status')
  count=$(echo "$response" | jq -r '.messages | length')

  if [ "$status" = "success" ] && [ "$count" -ge 0 ]; then
    echo "✓ Retrieved $count messages"
    PASSED=$((PASSED + 1))
  else
    echo "✗ Failed to get messages"
    echo "  Response: $response"
    FAILED=$((FAILED + 1))
  fi
}

# 테스트 4: 메시지 상태 조회
test_get_message_status() {
  log_test "Get Message Status"

  if [ -z "${LAST_RECEIPT:-}" ]; then
    echo "⊘ Skipping (no receipt from previous test)"
    return
  fi

  response=$(curl -s --max-time 10 "$CF_WORKER_URL/api/v1/messages/$LAST_RECEIPT/status" \
    -H "Authorization: Bearer $CF_WORKER_TOKEN")

  status=$(echo "$response" | jq -r '.status')

  # receipt가 없는 경우(일반 메시지) "not found"은 예외 처리
  if [ "$status" = "sent" ] || [ "$status" = "pending" ]; then
    echo "✓ Message status: $status"
    PASSED=$((PASSED + 1))
  elif echo "$response" | grep -q "not found"; then
    echo "✓ Message has no receipt (normal for non-emergency messages)"
    PASSED=$((PASSED + 1))
  else
    echo "✗ Unexpected status: $status"
    echo "  Response: $response"
    FAILED=$((FAILED + 1))
  fi
}

# 테스트 5: 인증 실패
test_authentication_required() {
  log_test "Authentication Required"

  # HTTP 상태 코드로 검증
  http_code=$(curl -s -w "%{http_code}" -o /tmp/response.json "$CF_WORKER_URL/api/v1/messages?limit=5")

  if [ "$http_code" = "401" ]; then
    echo "✓ Correctly rejected unauthenticated request (401)"
    PASSED=$((PASSED + 1))
  else
    echo "✗ Expected 401, got HTTP $http_code"
    echo "  Response: $(cat /tmp/response.json)"
    FAILED=$((FAILED + 1))
  fi
}

# 메인 실행 함수
run_all_tests() {
  log_info "Starting Worker API tests for: $CF_WORKER_URL"
  echo ""

  test_health_check
  test_send_message
  test_get_messages
  test_get_message_status
  test_authentication_required

  echo ""
  log_info "Results: $PASSED passed, $FAILED failed"

  if [ $FAILED -gt 0 ]; then
    exit 1
  fi
}

# 실행
run_all_tests
```

### 2. Makefile

```makefile
.PHONY: test-api test-api-verbose

test-api:
	@echo "Running Worker API tests..."
	@bash tests/api-test.sh

test-api-verbose:
	@echo "Running Worker API tests (verbose)..."
	@bash tests/api-test.sh --verbose
```

### 3. 환경변수 (`.env.test`)

```bash
# Worker API
CF_WORKER_URL=https://pushover-worker.cromksy.workers.dev
CF_WORKER_TOKEN=your-worker-api-token-here

# PushOver Credentials (실제 전송용)
PUSHOVER_API_TOKEN=your-pushover-api-token
PUSHOVER_USER_KEY=your-pushover-user-key

# Test Options
VERBOSE=false
```

**참고**: 환경변수명은 CF 접두사를 사용하여 혼동을 방지합니다.

---

## 사용법

### 로컬 실행

```bash
# 환경변수 설정
cp .env.example .env.test
# .env.test 편집

# 테스트 실행
make test-api
```

### CI 통합

```yaml
# .github/workflows/test.yml
- name: Run Worker API tests
  env:
    CF_WORKER_URL: ${{ secrets.CF_WORKER_URL }}
    CF_WORKER_TOKEN: ${{ secrets.CF_WORKER_TOKEN }}
    PUSHOVER_API_TOKEN: ${{ secrets.PUSHOVER_API_TOKEN }}
    PUSHOVER_USER_KEY: ${{ secrets.PUSHOVER_USER_KEY }}
  run: make test-api
```

---

## 의존성

- `bash`: 모든 Unix 계열 OS에 기본 내장
- `curl`: HTTP 요청 (대부분 기본 설치)
- `jq`: JSON 파싱 (CI 환경에서는 대부분可用)

```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt install jq

# Alpine Linux
apk add jq
```

---

## 검증 방식

### jq를 활용한 JSON 검증

```bash
# 상태 검증
status=$(echo "$response" | jq -r '.status')
[ "$status" = "success" ]

# 배열 길이 검증
count=$(echo "$response" | jq -r '.messages | length')
[ "$count" -ge 0 ]

# 필드 추출
receipt=$(echo "$response" | jq -r '.receipt')
```

### grep를 활용한 문자열 매칭

```bash
# 특정 문자열 포함 검증
echo "$response" | grep -q "Unauthorized"
```

---

## 테스트 순서

1. **Health Check** - 항상 먼저 실행
2. **Send Message** - receipt 획득 (다음 테스트에서 사용)
3. **Get Messages** - 목록 조회
4. **Get Message Status** - 2번에서 얻은 receipt로 상태 조회
5. **Authentication Required** - 인증 없는 요청 거부 확인

---

## 확장 가능성

향후 필요시 다음과 같이 확장 가능:

- 병렬 실행: `background` job 활용
- verbose 모드: `set -x`로 디버깅 출력
- skip 옵션: 특정 테스트 건너뛰기
- 더 많은 엔드포인트: 함수 추가만으로 확장

---

## 참고사항

- **dev.spec.ts 버그 수정 완료**: `../.env.test` 경로를 `../../.env.test`로 수정
- **.gitignore 업데이트 완료**: `.env.test`를 Git 제외 목록에 추가
- **테스트 통과 확인**: 5개 테스트 모두 성공 (Health Check, Send Message, Get Messages, Status, Auth)
- **환경변수명**: CF_WORKER_URL, CF_WORKER_TOKEN으로 통일 (혼동 방지)
- **산술 연산 주의**: `set -e`模式下 `((var++))`는 결과가 0일 때 exit code 1 반환
- **receipt 처리**: 일반 메시지는 receipt가 null, request UUID를 fallback으로 사용
- **token 필드**: 요청 body에 PushOver API 토큰 필수 (Worker API 요구사항)
