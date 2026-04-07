ifneq (,$(wildcard .env))
    include .env
    export
endif

RUN = @bash scripts/task-runner.sh

.PHONY: init plan apply output destroy \
      _apply-tf _apply-sync \
      migrate migrate-create migrate-local db-console \
      db-backup db-backup-local db-restore db-restore-local \
      setup dashboard-install setup-crates \
      clean clean-r2 log-rotate \
      build build-dashboard build-worker \
      check lint \
      deploy deploy-dashboard deploy-worker \
      destroy-all destroy-cloud destroy-dashboard destroy-worker \
      test test-sdk test-cli test-worker test-worker-verbose test-dashboard-loc test-dashboard-dev test-dashboard-all \
      loc loc-dashboard loc-worker

# ── Infrastructure: 인프라 (OpenTofu) ──
# init    백엔드 초기화
# plan    변경사항 미리보기 (dry-run)
# apply   인프라 변경 적용 + wrangler.toml 동기화
# output  리소스 ID/값 추출
# destroy 인프라 전체 삭제 (D1, KV, R2, Queues, Cron)

init:
	$(RUN) init bash -c 'cd infrastructure && tofu init'
plan:
	$(RUN) plan bash -c 'cd infrastructure && tofu plan'
apply: _apply-tf _apply-sync
_apply-tf:
	$(RUN) tofu-apply bash -c 'cd infrastructure && tofu apply'
_apply-sync:
	$(RUN) sync-wrangler bash scripts/sync-wrangler.sh
output:
	$(RUN) output bash -c 'cd infrastructure && tofu output'
destroy:
	$(RUN) destroy bash -c 'cd infrastructure && tofu destroy'

# ── Migration: DB 마이그레이션 (Wrangler D1) ──
# migrations/ 자동 탐색, d1_migrations 테이블로 이력 관리
# migrate          원격 D1 마이그레이션 적용
# migrate-local    로컬 D1 마이그레이션 적용
# migrate-create   새 마이그레이션 생성 (name=필수)
# db-console       D1 대화형 SQL (sql=필수)

DB_NAME = pushover-db

migrate:
	$(RUN) migrate bash -c 'cd crates/worker && wrangler d1 migrations apply $(DB_NAME) --remote'
migrate-local:
	$(RUN) migrate-local bash -c 'cd crates/worker && wrangler d1 migrations apply $(DB_NAME) --local'
migrate-create:
	$(RUN) migrate-create bash -c 'test -n "$(name)" || (echo "Usage: make migrate-create name=description" && exit 1) && cd crates/worker && wrangler d1 migrations create $(DB_NAME) $(name)'
db-console:
	$(RUN) db-console bash -c 'test -n "$(sql)" || (echo "Usage: make db-console sql=QUERY" && exit 1) && cd crates/worker && wrangler d1 execute $(DB_NAME) --remote --command="$(sql)"'

# ── Backup & Restore: D1 백업/복구 ──────────────
# db-backup          원격 D1 전체 export → SQL dump (backups/)
# db-backup-local    로컬 D1 전체 export → SQL dump (backups/)
# db-restore         원격 D1 복구 (file=필수)
# db-restore-local   로컬 D1 복구 (file=필수)

BACKUP_DIR  := backups
BACKUP_FILE := $(BACKUP_DIR)/d1-$(shell date -u +%Y%m%d_%H%M%S).sql

db-backup:
	$(RUN) db-backup bash -c 'mkdir -p $(BACKUP_DIR) && cd crates/worker && wrangler d1 export $(DB_NAME) --remote --output=../../$(BACKUP_FILE)'
db-backup-local:
	$(RUN) db-backup-local bash -c 'mkdir -p $(BACKUP_DIR) && FILE=$(BACKUP_DIR)/d1-local-$$(date -u +%Y%m%d_%H%M%S).sql && cd crates/worker && wrangler d1 export $(DB_NAME) --local --output=../../$$FILE && echo "Done: $$FILE"'
db-restore:
	$(RUN) db-restore bash -c 'test -n "$(file)" || (echo "Usage: make db-restore file=backups/d1-20260329_180000.sql" && exit 1) && test -f "$(file)" || (echo "File not found: $(file)" && exit 1) && cd crates/worker && wrangler d1 execute $(DB_NAME) --remote --file=../../$(file)'
db-restore-local:
	$(RUN) db-restore-local bash -c 'test -n "$(file)" || (echo "Usage: make db-restore-local file=backups/d1-20260329_180000.sql" && exit 1) && test -f "$(file)" || (echo "File not found: $(file)" && exit 1) && cd crates/worker && wrangler d1 execute $(DB_NAME) --local --file=../../$(file)'

# ── Setup: 로컬 개발 환경 구성 ─────
# setup            전체 의존성 설치 + Rust 컴파일 + 로컬 개발 준비

setup: dashboard-install setup-crates
dashboard-install:
	$(RUN) dashboard-install bash -c 'cd dashboard && pnpm install'
setup-crates:
	$(RUN) setup-crates bash -c 'cargo install worker-build && cd crates && cargo check --workspace'

# ── Clean: 정리 ─────────────────────────────
# clean          빌드 산출물 전체 삭제
# clean-r2       R2 버킷 오브젝트 삭제
# log-rotate     MAKE.log 수동 로테이션

clean:
	$(RUN) clean bash -c 'rm -rf target node_modules .wrangler crates/target crates/worker/node_modules crates/worker/.wrangler dashboard/.next dashboard/.vercel/output dashboard/node_modules dashboard/.wrangler dashboard/test-results dashboard/playwright-report'

R2_BUCKETS    = pushover-images pushover-backups
R2_ENDPOINT   = https://${CLOUDFLARE_ACCOUNT_ID}.r2.cloudflarestorage.com
R2_REGION     = auto

clean-r2:
	$(RUN) clean-r2 bash -c 'for bucket in $(R2_BUCKETS); do echo "  Cleaning $$bucket..."; count=$$(aws s3 ls s3://$$bucket/ --endpoint-url $(R2_ENDPOINT) --region $(R2_REGION) 2>/dev/null | wc -l | tr -d " "); if [ "$$count" -gt 0 ]; then echo "    Found $$count prefixes, deleting..."; aws s3 rm s3://$$bucket/ --recursive --endpoint-url $(R2_ENDPOINT) --region $(R2_REGION); else echo "    (empty)"; fi; done'

log-rotate:
	@bash scripts/task-runner.sh log-rotate bash -c 'test -f MAKE.log && gzip -c MAKE.log > MAKE.log.1.gz && : > MAKE.log && echo "Rotated MAKE.log" || echo "No MAKE.log to rotate"'

# ── Build: 빌드 ──────────────────────────────
# build          전체 빌드 (dashboard + worker)
# build-dashboard Next.js → Cloudflare Pages 빌드 산출물
# build-worker  Rust → WASM 컴파일

build: build-worker build-dashboard
build-dashboard:
	$(RUN) build-dashboard bash -c 'cd dashboard && pnpm build'
build-worker:
	$(RUN) build-worker bash -c 'cd crates/worker && worker-build --release'

# ── Check & Lint: 정적 분석 ────────────────
# check          빌드 없이 타입/문법 검사 (빠름)
# lint           Rust 린터 검사 (clippy)

check:
	$(RUN) check bash -c 'cd crates && cargo check --workspace'
lint:
	$(RUN) lint bash -c 'cd crates && cargo clippy --workspace -- -D warnings'

# ── Deploy: 배포 (순서: infra → worker → dashboard) ──
# deploy          전체 배포 (apply → deploy-worker → deploy-dashboard)
# deploy-dashboard Cloudflare Pages 배포
# deploy-worker   Cloudflare Workers 배포

deploy: apply deploy-worker deploy-dashboard
deploy-dashboard:
	$(RUN) deploy-dashboard bash -c 'cd dashboard && pnpm run deploy'
deploy-worker:
	$(RUN) deploy-worker bash -c 'cd crates/worker && wrangler deploy'

# ── Destroy: 삭제 (순서: Pages,Worker → R2 cleanup → infra) ──
# destroy-all    전체 삭제 (Pages,Worker → R2 cleanup → infra)
# destroy        인프라만 삭제 (D1, KV, R2, Queues)
# destroy-cloud  Pages,Worker만 삭제
# destroy-dashboard Cloudflare Pages 프로젝트 삭제
# destroy-worker Cloudflare Worker 삭제

destroy-all: destroy-cloud clean-r2 destroy
destroy-cloud: destroy-dashboard destroy-worker
destroy-dashboard:
	$(RUN) destroy-dashboard bash -c 'wrangler pages project delete pushover-dashboard'
destroy-worker:
	$(RUN) destroy-worker bash -c 'cd crates/worker && wrangler delete'

# ── Test: 테스트 ──────────────────────────
# test               전체 테스트 (sdk + cli + worker + dashboard)
# test-sdk            Rust SDK 단위 테스트
# test-cli            CLI 실제 바이너리 통합 테스트
# test-worker         Worker API 엔드포인트 테스트
# test-worker-verbose Worker API 상세 로그 테스트
# test-dashboard-loc  Dashboard 로컬 환경 테스트
# test-dashboard-dev  Dashboard 개발 서버 대상 테스트
# test-dashboard-all  Dashboard 전체 테스트

test: test-sdk test-cli test-worker test-dashboard-dev
test-sdk:
	$(RUN) test-sdk bash -c 'cd crates && TEST_NAME=test-sdk cargo test -p pushover-sdk'
test-cli:
	$(RUN) test-cli bash -c 'TEST_NAME=test-cli bash tests/cli-test.sh'
test-worker:
	$(RUN) test-worker bash -c 'TEST_NAME=test-worker bash tests/api-test.sh'
test-worker-verbose:
	$(RUN) test-worker-verbose bash -c 'TEST_NAME=test-worker-verbose VERBOSE=true bash tests/api-test.sh'
test-dashboard-loc:
	$(RUN) test-dashboard-loc bash -c 'cd dashboard && TEST_NAME=test-dashboard-loc pnpm test:loc'
test-dashboard-dev:
	$(RUN) test-dashboard-dev bash -c 'cd dashboard && TEST_NAME=test-dashboard-dev pnpm test:dev'
test-dashboard-all:
	$(RUN) test-dashboard-all bash -c 'cd dashboard && TEST_NAME=test-dashboard-all pnpm test:all'

# ── Dev: 로컬 개발 서버 ──
# loc            전체 개발 서버 실행 (loc-dashboard + loc-worker)
# loc-dashboard  Next.js 개발 서버 (http://localhost:3000)
# loc-worker     Worker 로컬 개발 서버 (wrangler dev)

loc: loc-dashboard loc-worker
loc-dashboard:
	$(RUN) loc-dashboard bash -c 'cd dashboard && pnpm loc'
loc-worker:
	$(RUN) loc-worker bash -c 'cd crates/worker && wrangler dev'
