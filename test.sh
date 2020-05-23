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

test() {
    testPath . || return 1
    testPath ~ || return 1
    testPath / || return 1
    testPath .. || return 1
    testPath ./target/debug/wslpath || return 1

    # test illegal Windows filename chars like ':'
    testPath ~/perl5/man/man3/local::lib.3pm || return 1
#    testPath ~/perl5/man/man3/locallib.3pm || return 1

    testAllPaths || return 1
}

if test "${@}"; then
    echo
    >&2 echo PASS
else
    echo
    >&2 echo FAIL
fi
