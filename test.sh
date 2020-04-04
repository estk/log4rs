#!/bin/bash -e
# Test each feature in windows
function main {
    if [ "$1" == "win" ]; then
        target_arg='--target x86_64-pc-windows-gnu'
    else
        target_arg=''
    fi
    for feature in $(cargo read-manifest | jq -r '.features|keys|join("\n")'); do
        echo building with feature "$feature"
        echo cross test $target_arg  --no-default-features --features "$feature"
        cross test $target_arg  --no-default-features --features "$feature"
    done
}

main "$@"
