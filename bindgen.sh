#!/usr/bin/env bash

set -e

if [[ $# -ne 1 ]] || [[ $(basename "$1") != 'trace-cmd.h' ]]; then
    echo "Usage: $(basename "$0") /path/to/trace-cmd.h"
    exit 1
fi

echo "
#![allow(clippy::upper_case_acronyms)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
" > ./src/bindings.rs

bindgen \
    --no-layout-tests \
    --no-doc-comments \
    --with-derive-default \
    --size_t-is-usize \
    "$1" \
    -- \
    -I /usr/include/traceevent/ \
    -I /usr/include/tracefs/ \
    >> ./src/bindings.rs
