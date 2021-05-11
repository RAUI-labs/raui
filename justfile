# Bake the README.md from the template
readme:
    cargo readme > README.md
    cargo readme > site/README.md

#
# MDBook Jobs
#

# Run Rustfmt on the MDBook rust snippets
book-rustfmt:
    rustfmt book/src/rust/*.rs

