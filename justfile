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
