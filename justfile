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
    cargo build --examples

clippy:
    cargo clippy --all

test:
    cargo test --all --features all

example NAME="setup":
    cargo run --example {{NAME}}

demo NAME="todo-app":
    cd ./demos/{{NAME}} && cargo run

guide NAME:
    cd ./site/rust/guide_{{NAME}} && cargo run

# Mandatory checks to run before pushing changes to repository
checks:
    just format
    just build
    just clippy
    just test
    just readme

# Print the documentation coverage for a crate in the workspace
doc-coverage crate="raui-core":
    cargo +nightly rustdoc -p {{crate}} -- -Z unstable-options --show-coverage

clean:
  find . -name target -type d -exec rm -r {} +
  just remove-lockfiles

remove-lockfiles:
  find . -name Cargo.lock -type f -exec rm {} +

list-outdated:
  cargo outdated -R -w

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

website-live-dev:
    cd site && zola serve

update:
    cargo update --manifest-path ./raui-derive/Cargo.toml --aggressive
    cargo update --manifest-path ./raui-core/Cargo.toml --aggressive
    cargo update --manifest-path ./raui-material/Cargo.toml --aggressive
    cargo update --manifest-path ./raui-retained/Cargo.toml --aggressive
    cargo update --manifest-path ./raui-immediate/Cargo.toml --aggressive
    cargo update --manifest-path ./raui-immediate-widgets/Cargo.toml --aggressive
    cargo update --manifest-path ./raui-json-renderer/Cargo.toml --aggressive
    cargo update --manifest-path ./raui-tesselate-renderer/Cargo.toml --aggressive
    cargo update --manifest-path ./raui-app/Cargo.toml --aggressive
    cargo update --manifest-path ./Cargo.toml --aggressive

publish:
    cargo publish --no-verify --manifest-path ./raui-derive/Cargo.toml
    sleep 1
    cargo publish --no-verify --manifest-path ./raui-core/Cargo.toml
    sleep 1
    cargo publish --no-verify --manifest-path ./raui-material/Cargo.toml
    sleep 1
    cargo publish --no-verify --manifest-path ./raui-retained/Cargo.toml
    sleep 1
    cargo publish --no-verify --manifest-path ./raui-immediate/Cargo.toml
    sleep 1
    cargo publish --no-verify --manifest-path ./raui-immediate-widgets/Cargo.toml
    sleep 1
    cargo publish --no-verify --manifest-path ./raui-json-renderer/Cargo.toml
    sleep 1
    cargo publish --no-verify --manifest-path ./raui-tesselate-renderer/Cargo.toml
    sleep 1
    cargo publish --no-verify --manifest-path ./raui-app/Cargo.toml
    sleep 1
    cargo publish --no-verify --manifest-path ./Cargo.toml
