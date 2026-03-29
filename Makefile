.PHONY: init plan apply output \
      migrate migrate-create migrate-local db-console \
      setup dashboard-install setup-crates \
      clean \
      build build-dashboard build-worker \
      check lint \
      deploy deploy-dashboard deploy-worker \
      test test-sdk test-cli test-worker test-worker-verbose test-dashboard-loc test-dashboard-dev test-dashboard-all \
      dev dev-dashboard dev-worker

# ── Infrastructure: 인프라 (OpenTofu) ──
# init    백엔드 초기화
# plan    변경사항 미리보기 (dry-run)
# apply   인프라 변경 적용
# output  리소스 ID/값 추출

init:
	@echo "Initializing OpenTofu backend..."
	@cd infrastructure && tofu init
plan:
	@echo "Planning infrastructure changes..."
	@cd infrastructure && tofu plan
apply:
	@echo "Applying infrastructure changes..."
	@cd infrastructure && tofu apply
output:
	@echo "Showing infrastructure outputs..."
	@cd infrastructure && tofu output

# ── Migration: DB 마이그레이션 (Wrangler D1) ──
# migrations/ 자동 탐색, d1_migrations 테이블로 이력 관리
# migrate          원격 D1 마이그레이션 적용
# migrate-local    로컬 D1 마이그레이션 적용
# migrate-create   새 마이그레이션 생성 (name=필수)
# db-console       D1 대화형 SQL (sql=필수)

DB_NAME = pushover-db

migrate:
	@echo "Applying D1 migrations (remote)..."
	@cd crates/worker && wrangler d1 migrations apply $(DB_NAME) --remote
migrate-local:
	@echo "Applying D1 migrations (local)..."
	@cd crates/worker && wrangler d1 migrations apply $(DB_NAME) --local
migrate-create:
	@echo "Creating migration..."
	@test -n "$(name)" || (echo "Usage: make migrate-create name=description" && exit 1)
	@cd crates/worker && wrangler d1 migrations create $(DB_NAME) $(name)
db-console:
	@echo "Opening D1 console..."
	@test -n "$(sql)" || (echo "Usage: make db-console sql='SELECT * FROM messages'" && exit 1)
	@cd crates/worker && wrangler d1 execute $(DB_NAME) --remote --command="$(sql)"

# ── Setup: 로컬 개발 환경 구성 ─────
# setup            전체 의존성 설치 + Rust 컴파일 + 로컬 개발 준비

setup: dashboard-install setup-crates
dashboard-install:
	@echo "Installing Dashboard dependencies..."
	@cd dashboard && pnpm install
setup-crates:
	@echo "Installing worker-build..."
	@cargo install worker-build
	@echo "Checking Rust workspace..."
	@cd crates && cargo check --workspace

# ── Clean: 정리 ─────────────────────────────
# clean          빌드 산출물 전체 삭제

clean:
	rm -rf \
		target node_modules .wrangler \
		crates/target crates/worker/node_modules crates/worker/.wrangler \
		dashboard/.next dashboard/.vercel/output dashboard/node_modules dashboard/.wrangler dashboard/test-results dashboard/playwright-report

# ── Build: 빌드 ──────────────────────────────
# build          전체 빌드 (dashboard + worker)
# build-dashboard Next.js → Cloudflare Pages 빌드 산출물
# build-worker  Rust → WASM 컴파일

build: build-dashboard build-worker
build-dashboard:
	@echo "Building Dashboard (Next.js)..."
	@cd dashboard && pnpm build
build-worker:
	@echo "Building Worker (WASM)..."
	@cd crates/worker && worker-build --release

# ── Check & Lint: 정적 분석 ────────────────
# check          빌드 없이 타입/문법 검사 (빠름)
# lint           Rust 린터 검사 (clippy)

check:
	@echo "Checking Rust types..."
	@cd crates && cargo check --workspace
lint:
	@echo "Linting Rust code..."
	@cd crates && cargo clippy --workspace -- -D warnings

# ── Deploy: 배포 ──────────────────────────
# deploy          전체 배포 (dashboard + worker)
# deploy-dashboard Cloudflare Pages 배포
# deploy-worker   Cloudflare Workers 배포

deploy: deploy-dashboard deploy-worker
deploy-dashboard:
	@echo "Deploying Dashboard to Cloudflare Pages..."
	@cd dashboard && pnpm run deploy
deploy-worker:
	@echo "Deploying Worker to Cloudflare Workers..."
	@cd crates/worker && wrangler deploy

# ── Test: 테스트 ──────────────────────────
# test               전체 테스트 (sdk + cli + worker + dashboard)
# test-sdk            Rust SDK 단위 테스트
# test-cli            CLI 실제 바이너리 통합 테스트
# test-worker         Worker API 엔드포인트 테스트
# test-worker-verbose Worker API 상세 로그 테스트
# test-dashboard-loc  Dashboard 로컬 환경 테스트
# test-dashboard-dev  Dashboard 개발 서버 대상 테스트
# test-dashboard-all  Dashboard 전체 테스트

test: test-sdk test-cli test-worker test-dashboard-loc
test-sdk:
	@echo "Running Rust SDK tests..."
	@cd crates && cargo test -p pushover-sdk
test-cli:
	@echo "Running CLI integration tests..."
	@bash tests/cli-test.sh
test-worker:
	@echo "Running Worker API tests..."
	@bash tests/api-test.sh
test-worker-verbose:
	@echo "Running Worker API tests (verbose)..."
	@VERBOSE=true bash tests/api-test.sh
test-dashboard-loc:
	@echo "Running Dashboard local tests..."
	@cd dashboard && pnpm test:loc
test-dashboard-dev:
	@echo "Running Dashboard dev tests..."
	@cd dashboard && pnpm test:dev
test-dashboard-all:
	@echo "Running all Dashboard tests..."
	@cd dashboard && pnpm test:all

# ── Dev: 로컬 개발 서버 (Cloudflare 에뮬레이션) ──
# dev            전체 개발 서버 실행
# dev-dashboard  Next.js 개발 서버 (http://localhost:3000)
# dev-worker     Worker 로컬 개발 서버 (wrangler dev)

dev: dev-dashboard dev-worker
dev-dashboard:
	@echo "Starting Dashboard dev server..."
	@cd dashboard && pnpm dev
dev-worker:
	@echo "Starting Worker dev server..."
	@cd crates/worker && wrangler dev
