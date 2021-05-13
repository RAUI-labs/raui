# List the just recipe list
list:
    just --list

# Bake the README.md from the template
readme:
    cargo readme > README.md
    cargo readme > site/README.md

# Mandatory checks to run before pushing changes to repository
checks:
    cargo fmt --all
    cargo build --all
    cargo clippy --all
    cargo test --all --features all

# Print the documentation coverage for a crate in the workspace
doc-coverage crate="raui-core":
    cargo +nightly rustdoc -p {{crate}} -- -Z unstable-options --show-coverage

# Run the Rust doctests in the website docs
website-doc-tests:
    cargo build --features all -p raui --target-dir target/doctests
    @set -e; \
    for file in $(find site/content/ -name '*.md'); do \
        echo "Testing: $file"; \
        rustdoc \
            --edition 2018 \
            --extern raui \
            --crate-name docs-test \
            $file \
            --test \
            -L target/doctests/debug/deps; \
    done