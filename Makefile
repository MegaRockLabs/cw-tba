
TEST_ADDRS ?= $(shell jq -r '.[].address' ./e2e/configs/test_accounts.json | tr '\n' ' ')
GAS_LIMIT ?= "75000000"


lint:
	cargo clippy --all-targets -- -D warnings

optimize:
	# NOTE: On a cache miss, the dockerized workspace-optimizer container
	# is creating these dirs with permissions we cannot use in CI.
	# So, we need to ensure these dirs are created before calling optimize.sh:
	mkdir -p artifacts target
	sh scripts/optimize.sh

publish-packages:
	sh scripts/publish-packages.sh

publish-contracts:
	sh scripts/publish-contracts.sh

schema:
	sh scripts/schema.sh $(VERSION)

release:
	sh scripts/release.sh $(VERSION)

upload:
	sh scripts/upload.sh