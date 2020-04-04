#!/bin/bash -e
function main {
    cargo test --all-features

    for feature in $(cargo read-manifest | jq -r '.features|keys|join("\n")'); do
        echo Testing with feature "$feature"

        cargo test --no-default-features --features "$feature"
    done
}

main "$@"
