# Test each feature in windows
function main {
    for feature in $(cargo read-manifest | jq -r '.features|keys|join("\n")'); do
        echo building with feature "$feature"
        cross test --target x86_64-pc-windows-gnu  --no-default-features --features "$feature"
    done
}

main "$@"
