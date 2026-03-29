#!/bin/bash
set -euo pipefail

# 색상 출력
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 환경변수 로드
if [ -f .env ]; then
  source .env
else
  echo "Error: .env not found"
  exit 1
fi

# Verbose 모드 (선택 사항)
VERBOSE="${VERBOSE:-false}"
if [ "$VERBOSE" = "true" ]; then
  set -x  # 디버깅 출력 활성화
fi

# 필수 환경변수 검증
: "${CLOUDFLARE_WORKER_URL:?CLOUDFLARE_WORKER_URL required}"
: "${CLOUDFLARE_WORKER_TOKEN:?CLOUDFLARE_WORKER_TOKEN required}"
: "${PUSHOVER_API_TOKEN:?PUSHOVER_API_TOKEN required}"
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

  response=$(curl -s "$CLOUDFLARE_WORKER_URL/health")

  if [ "$response" = "OK" ]; then
    echo "✓ Health check passed"
    PASSED=$((PASSED + 1))
  else
    echo "✗ Expected 'OK', got '$response'"
    FAILED=$((FAILED + 1))
  fi
}

# 테스트 2: 메시지 전송 (이미지 첨부 포함)
test_send_message() {
  log_test "Send Message (with image)"

  # sample.jpg base64 인코딩
  IMAGE_B64=""
  if [ -f "tests/sample.jpg" ]; then
    IMAGE_B64=$(base64 -i tests/sample.jpg)
  fi

  timestamp=$(date +%s)

  if [ -n "$IMAGE_B64" ]; then
    response=$(curl -s --max-time 15 -X POST "$CLOUDFLARE_WORKER_URL/api/v1/messages" \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer $CLOUDFLARE_WORKER_TOKEN" \
      -d "{
        \"token\": \"$PUSHOVER_API_TOKEN\",
        \"user\": \"$PUSHOVER_USER_KEY\",
        \"message\": \"Test message $timestamp\",
        \"title\": \"${TEST_NAME:-test-worker}\",
        \"image\": \"$IMAGE_B64\"
      }")
  else
    response=$(curl -s --max-time 10 -X POST "$CLOUDFLARE_WORKER_URL/api/v1/messages" \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer $CLOUDFLARE_WORKER_TOKEN" \
      -d "{
        \"token\": \"$PUSHOVER_API_TOKEN\",
        \"user\": \"$PUSHOVER_USER_KEY\",
        \"message\": \"Test message $timestamp\",
        \"title\": \"${TEST_NAME:-test-worker}\"
      }")
  fi

  status=$(echo "$response" | jq -r '.status' 2>/dev/null)

  if [ -z "$status" ] || [ "$status" = "null" ]; then
    echo "✗ Failed to parse JSON response"
    echo "  Response: $response"
    FAILED=$((FAILED + 1))
    return
  fi

  if [ "$status" = "success" ] || [ "$status" = "queued" ]; then
    message_id=$(echo "$response" | jq -r '.message_id // .receipt')
    request=$(echo "$response" | jq -r '.request')
    echo "✓ Message $status, id: $message_id, request: $request"
    LAST_RECEIPT="${message_id:-$request}"
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

  response=$(curl -s --max-time 10 "$CLOUDFLARE_WORKER_URL/api/v1/messages?limit=10" \
    -H "Authorization: Bearer $CLOUDFLARE_WORKER_TOKEN")

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

  response=$(curl -s --max-time 10 "$CLOUDFLARE_WORKER_URL/api/v1/messages/$LAST_RECEIPT/status" \
    -H "Authorization: Bearer $CLOUDFLARE_WORKER_TOKEN")

  status=$(echo "$response" | jq -r '.status')

  # receipt가 없는 경우(d일반 메시지) "not found"은 예외 처리
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
  http_code=$(curl -s -w "%{http_code}" -o /tmp/response.json "$CLOUDFLARE_WORKER_URL/api/v1/messages?limit=5")

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
  log_info "Starting Worker API tests for: $CLOUDFLARE_WORKER_URL"
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
