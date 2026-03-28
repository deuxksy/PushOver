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

# 필수 환경변수 검증
: "${WORKER_URL:?WORKER_URL required}"
: "${WORKER_TOKEN:?WORKER_TOKEN required}"
: "${PUSHOVER_TOKEN:?PUSHOVER_TOKEN required}"
: "${PUSHOVER_USER_KEY:?PUSHOVER_USER_KEY required}"

# 테스트 카운터
PASSED=0
FAILED=0

# 헬퍼 함수
log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_test() { echo -e "${YELLOW}[TEST]${NC} $1"; }

# 테스트 1: Health Check
test_health_check() {
  log_test "Health Check"

  response=$(curl -s "$WORKER_URL/health")

  if [ "$response" = "OK" ]; then
    echo "✓ Health check passed"
    ((PASSED++))
  else
    echo "✗ Expected 'OK', got '$response'"
    ((FAILED++))
  fi
}

# 테스트 2: 메시지 전송
test_send_message() {
  log_test "Send Message"

  timestamp=$(date +%s)
  response=$(curl -s -X POST "$WORKER_URL/api/v1/messages" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $WORKER_TOKEN" \
    -d "{
      \"user\": \"$PUSHOVER_USER_KEY\",
      \"message\": \"Test message $timestamp\",
      \"title\": \"API Test\"
    }")

  status=$(echo "$response" | jq -r '.status')

  if [ "$status" = "success" ]; then
    receipt=$(echo "$response" | jq -r '.receipt')
    echo "✓ Message sent, receipt: $receipt"
    export LAST_RECEIPT="$receipt"
    ((PASSED++))
  else
    echo "✗ Expected status 'success', got '$status'"
    echo "  Response: $response"
    ((FAILED++))
  fi
}

# 테스트 3: 메시지 목록 조회
test_get_messages() {
  log_test "Get Messages"

  response=$(curl -s "$WORKER_URL/api/v1/messages?limit=10" \
    -H "Authorization: Bearer $WORKER_TOKEN")

  status=$(echo "$response" | jq -r '.status')
  count=$(echo "$response" | jq -r '.messages | length')

  if [ "$status" = "success" ] && [ "$count" -ge 0 ]; then
    echo "✓ Retrieved $count messages"
    ((PASSED++))
  else
    echo "✗ Failed to get messages"
    echo "  Response: $response"
    ((FAILED++))
  fi
}

# 테스트 4: 메시지 상태 조회
test_get_message_status() {
  log_test "Get Message Status"

  if [ -z "${LAST_RECEIPT:-}" ]; then
    echo "⊘ Skipping (no receipt from previous test)"
    return
  fi

  response=$(curl -s "$WORKER_URL/api/v1/messages/$LAST_RECEIPT/status" \
    -H "Authorization: Bearer $WORKER_TOKEN")

  status=$(echo "$response" | jq -r '.status')

  if [ "$status" = "sent" ] || [ "$status" = "pending" ]; then
    echo "✓ Message status: $status"
    ((PASSED++))
  else
    echo "✗ Unexpected status: $status"
    echo "  Response: $response"
    ((FAILED++))
  fi
}

# 테스트 5: 인증 실패
test_authentication_required() {
  log_test "Authentication Required"

  response=$(curl -s "$WORKER_URL/api/v1/messages?limit=5")

  if echo "$response" | grep -q "Unauthorized\|401"; then
    echo "✓ Correctly rejected unauthenticated request"
    ((PASSED++))
  else
    echo "✗ Should require authentication"
    echo "  Response: $response"
    ((FAILED++))
  fi
}

# 메인 실행 함수
run_all_tests() {
  log_info "Starting Worker API tests for: $WORKER_URL"
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
WORKER_URL=https://pushover-worker.cromksy.workers.dev
WORKER_TOKEN=your-worker-token-here

# PushOver Credentials (실제 전송용)
PUSHOVER_TOKEN=your-pushover-api-token
PUSHOVER_USER_KEY=your-pushover-user-key

# Test Options
VERBOSE=false
```

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
    WORKER_URL: ${{ secrets.WORKER_URL }}
    WORKER_TOKEN: ${{ secrets.WORKER_TOKEN }}
    PUSHOVER_TOKEN: ${{ secrets.PUSHOVER_TOKEN }}
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

- **dev.spec.ts 버그 수정**: 현재 `../.env.test` 경로를 `../../.env.test`로 수정 필요
- **README와 동기화**: README의 curl 예시와 테스트 코드를 동기화
- **테스트 격리**: 실제 PushOver API 호출이므로 테스트 전용 계정 사용 권장
