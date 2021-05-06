# Bake the README.md from the template
readme:
    mdbakery --input README.template.md --output README.md

#
# MDBook Jobs
#

# Run Rustfmt on the MDBook rust snippets
book-rustfmt:
    rustfmt book/src/rust/*.rs

