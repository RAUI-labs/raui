# Bake the README.md from the template
readme:
    cargo readme > README.md
    cargo readme > site/README.md

# Print the documentation coverage for a crate in the workspace
doc-coverage crate="raui-core":
    cargo +nightly rustdoc -p {{crate}} -- -Z unstable-options --show-coverage

#
# MDBook Jobs
#

# Run Rustfmt on the MDBook rust snippets
book-rustfmt:
    rustfmt book/src/rust/*.rs

