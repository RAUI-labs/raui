# List the just recipe list
list:
    just --list

# Bake the README.md from the template
readme:
    cargo readme > README.md

format:
    cargo fmt --all

build:
    cargo build --all

clippy:
    cargo clippy --all

test:
    cargo test --all --features all

# Mandatory checks to run before pushing changes to repository
checks:
    just format
    just build
    just clippy
    just test

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

publish:
    cargo publish --no-verify --manifest-path ./raui-derive/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-core/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-material/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-binary-renderer/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-json-renderer/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-yaml-renderer/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-ron-renderer/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-html-renderer/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-tesselate-renderer/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-tetra-renderer/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-ggez-renderer/Cargo.toml
    cargo publish --no-verify --manifest-path ./raui-quick-start/Cargo.toml
