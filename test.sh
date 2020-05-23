#!/usr/bin/env bash

# shellcheck disable=SC2155
testPath() {
    local path="${1}"
    local winPath=$(wslpath -w -a "${path}")
    local wslPath=$(wslpath "${winPath}")
    local myWslPath=$(./target/debug/wslpath "${winPath}")
    if [[ "${myWslPath}" != "${wslPath}" ]]; then
        echo "${myWslPath} != ${wslPath}"
        return 1
    fi
}

testAllPaths() {
    locate | map testPath
}

testAllPaths "${@}"
