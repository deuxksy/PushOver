.PONY: test-api test-api-verbose dashboard-test-loc dashboard-test-dev dashboard-test-all

test-api:
	@echo "Running Worker API tests..."
	@bash tests/api-test.sh

test-api-verbose:
	@echo "Running Worker API tests (verbose)..."
	@VERBOSE=true bash tests/api-test.sh

dashboard-test-loc:
	@echo "Running Dashboard LOC (Local) tests..."
	@cd dashboard && pnpm test:loc

dashboard-test-dev:
	@echo "Running Dashboard dev tests..."
	@cd dashboard && pnpm test:dev

dashboard-test-all:
	@echo "Running all Dashboard tests..."
	@cd dashboard && pnpm test:all
