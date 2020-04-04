#!/bin/bash -e
function main {

    RUSTFLAGS='-D warnings' cargo clippy --all-features

    for feature in $(cargo read-manifest | jq -r '.features|keys|join("\n")'); do
        echo Linting with feature "$feature"

        RUSTFLAGS='-D warnings' cargo clippy --no-default-features --features "$feature"
    done
}

main "$@"
