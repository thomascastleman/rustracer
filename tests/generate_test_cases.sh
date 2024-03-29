#!/bin/env bash
# This script generates the macro invocations found in test_against_benchmark.rs,
# by generating a call for every scenefile found in tests/scenefiles.

SCENEFILES_DIRECTORY="tests/scenefiles/"

if [ ! -d "$SCENEFILES_DIRECTORY" ]; then
    echo "This script must be invoked from the project root."
    exit 1
fi

PRELUDE="//! This file was generated by $0.

mod common;
"

echo "$PRELUDE"

# For each found scenefile
find "$SCENEFILES_DIRECTORY" -type f -name '*.xml' | while read scenefile; do
    # Remove $SCENEFILES_DIRECTORY prefix from paths
    PREFIX_REMOVED="${scenefile#"$SCENEFILES_DIRECTORY"}"

    # Remove .xml from path
    SUFFIX_REMOVED="${PREFIX_REMOVED%".xml"}"

    IFS='/' read -r -a PATH_COMPONENTS <<< "$SUFFIX_REMOVED"
    DIRECTORY=${PATH_COMPONENTS[0]}
    FILE=${PATH_COMPONENTS[1]}

    echo "test_against_benchmark!($DIRECTORY, $FILE);"
done