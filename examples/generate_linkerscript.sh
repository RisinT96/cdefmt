#!/bin/bash

# Get default linker, trim comments, cleanup empty lines at end of file, insert custom section
$1 --verbose |
    awk -- '/^===/ { show = 1 - show ; next } { if (show) print }' |
    sed -e :a -e '/^\n*$/{$d;N;ba' -e '}' |
    sed "\$i .cdefmt 0 (INFO) : { KEEP(*(.cdefmt .cdefmt.*)); }" \
        >$2
