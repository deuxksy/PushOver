.PHONY: test-api test-api-verbose

test-api:
	@echo "Running Worker API tests..."
	@bash tests/api-test.sh

test-api-verbose:
	@echo "Running Worker API tests (verbose)..."
	@VERBOSE=true bash tests/api-test.sh
