#!/bin/env bash
# Deletes all PNGs in the diff_output directory.

DIFF_OUTPUT_DIRECTORY="tests/diff_output"

if [ ! -d "$DIFF_OUTPUT_DIRECTORY" ]; then 
    echo "This script must be invoked from the project root."
    exit 1
fi

find "$DIFF_OUTPUT_DIRECTORY" -type f -name "*.png" -delete