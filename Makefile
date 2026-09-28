.PHONY: doc-test

doc-test:
	docdoctor check --manifest-path ./Cargo.toml docs/fences.md docs/metadata.md
