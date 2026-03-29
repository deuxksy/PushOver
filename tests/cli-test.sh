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

# Verbose 모드
VERBOSE="${VERBOSE:-false}"
if [ "$VERBOSE" = "true" ]; then
  set -x
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

# CLI 바이너리 경로
CLI_BIN="crates/target/release/pushover"

# macOS: ~/Library/Application Support/pushover/config.toml
# Linux: ~/.config/pushover/config.toml
if [ "$(uname)" = "Darwin" ]; then
  CONFIG_DIR="$HOME/Library/Application Support/pushover"
else
  CONFIG_DIR="$HOME/.config/pushover"
fi
CONFIG_FILE="$CONFIG_DIR/config.toml"
BACKUP_FILE="${CONFIG_FILE}.bak"

# Config 백업/복원
backup_config() {
  if [ -f "$CONFIG_FILE" ]; then
    cp "$CONFIG_FILE" "$BACKUP_FILE"
  fi
}

restore_config() {
  if [ -f "$BACKUP_FILE" ]; then
    mv "$BACKUP_FILE" "$CONFIG_FILE"
  elif [ -f "$CONFIG_FILE" ]; then
    rm "$CONFIG_FILE"
  fi
}

# 테스트용 config 생성
write_test_config() {
  mkdir -p "$CONFIG_DIR"
  cat > "$CONFIG_FILE" <<EOF
default_profile = "test"

[[profiles]]
name = "test"
user_key = "$PUSHOVER_USER_KEY"
pushover_token = "$PUSHOVER_API_TOKEN"
worker_token = "$CLOUDFLARE_WORKER_TOKEN"
api_endpoint = "$CLOUDFLARE_WORKER_URL"
EOF
}

# 종료 시 config 복원
trap restore_config EXIT

# 테스트 1: CLI 빌드 확인
test_cli_binary() {
  log_test "CLI Binary Exists"

  if [ ! -f "$CLI_BIN" ]; then
    log_info "Building CLI binary..."
    (cd crates && cargo build -p pushover-cli --release 2>&1)
  fi

  if [ -f "$CLI_BIN" ]; then
    echo "✓ CLI binary found: $CLI_BIN"
    PASSED=$((PASSED + 1))
  else
    echo "✗ Failed to build CLI binary"
    FAILED=$((FAILED + 1))
    return 1
  fi
}

# 테스트 2: 메시지 전송 (config 사용)
test_send_message() {
  log_test "Send Message via CLI"

  backup_config
  write_test_config

  timestamp=$(date +%s)
  output=$("$CLI_BIN" send "CLI test $timestamp" --title "${TEST_NAME:-test-cli}" --image tests/sample.jpg 2>&1)
  exit_code=$?

  restore_config

  if [ $exit_code -eq 0 ] && echo "$output" | grep -q "successfully"; then
    echo "✓ Message sent via CLI"
    PASSED=$((PASSED + 1))
  else
    echo "✗ CLI send failed (exit: $exit_code)"
    echo "  Output: $output"
    FAILED=$((FAILED + 1))
  fi
}

# 테스트 3: 메시지 전송 (--user/--token 오버라이드)
test_send_with_override() {
  log_test "Send Message with --user/--token override"

  backup_config
  write_test_config

  timestamp=$(date +%s)
  output=$("$CLI_BIN" send "CLI override test $timestamp" \
    --title "${TEST_NAME:-test-cli} override" \
    --user "$PUSHOVER_USER_KEY" \
    --token "$PUSHOVER_API_TOKEN" \
    --image tests/sample.jpg 2>&1)
  exit_code=$?

  restore_config

  if [ $exit_code -eq 0 ] && echo "$output" | grep -q "successfully"; then
    echo "✓ Message sent with override args"
    PASSED=$((PASSED + 1))
  else
    echo "✗ CLI send with override failed (exit: $exit_code)"
    echo "  Output: $output"
    FAILED=$((FAILED + 1))
  fi
}

# 테스트 4: 이미지 첨부 메시지 전송
test_send_with_image() {
  log_test "Send Message with Image"

  if [ ! -f "tests/sample.jpg" ]; then
    echo "⊘ Skipping (tests/sample.jpg not found)"
    return
  fi

  backup_config
  write_test_config

  timestamp=$(date +%s)
  output=$("$CLI_BIN" send "CLI image test $timestamp" \
    --title "${TEST_NAME:-test-cli} image" \
    --image tests/sample.jpg 2>&1)
  exit_code=$?

  restore_config

  if [ $exit_code -eq 0 ] && echo "$output" | grep -q "successfully"; then
    echo "✓ Message sent with image"
    PASSED=$((PASSED + 1))
  else
    echo "✗ CLI send with image failed (exit: $exit_code)"
    echo "  Output: $output"
    FAILED=$((FAILED + 1))
  fi
}

# 메인 실행
run_all_tests() {
  log_info "Starting CLI tests for: $CLOUDFLARE_WORKER_URL"
  echo ""

  test_cli_binary
  test_send_message
  # test_send_with_override
  # test_send_with_image

  echo ""
  log_info "Results: $PASSED passed, $FAILED failed"

  if [ $FAILED -gt 0 ]; then
    exit 1
  fi
}

run_all_tests
