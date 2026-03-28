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
  response=$(curl -s --max-time 10 -X POST "$CF_WORKER_URL/api/v1/messages" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $CF_WORKER_TOKEN" \
    -d "{
      \"user\": \"$PUSHOVER_USER_KEY\",
      \"message\": \"Test message $timestamp\",
      \"title\": \"API Test\"
    }")

  status=$(echo "$response" | jq -r '.status' 2>/dev/null)

  if [ -z "$status" ] || [ "$status" = "null" ]; then
    echo "✗ Failed to parse JSON response"
    echo "  Response: $response"
    ((FAILED++))
    return
  fi

  if [ "$status" = "success" ]; then
    receipt=$(echo "$response" | jq -r '.receipt')
    echo "✓ Message sent, receipt: $receipt"
    LAST_RECEIPT="$receipt"
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

  response=$(curl -s --max-time 10 "$CF_WORKER_URL/api/v1/messages?limit=10" \
    -H "Authorization: Bearer $CF_WORKER_TOKEN")

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

  response=$(curl -s --max-time 10 "$CF_WORKER_URL/api/v1/messages/$LAST_RECEIPT/status" \
    -H "Authorization: Bearer $CF_WORKER_TOKEN")

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

  # HTTP 상태 코드로 검증
  http_code=$(curl -s -w "%{http_code}" -o /tmp/response.json "$CF_WORKER_URL/api/v1/messages?limit=5")

  if [ "$http_code" = "401" ]; then
    echo "✓ Correctly rejected unauthenticated request (401)"
    ((PASSED++))
  else
    echo "✗ Expected 401, got HTTP $http_code"
    echo "  Response: $(cat /tmp/response.json)"
    ((FAILED++))
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
