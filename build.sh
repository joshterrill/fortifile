#!/bin/bash

cargo bundle --release

if [ $? -eq 0 ]; then
    echo "cargo bundle completed successfully."

    CUSTOM_PLIST="./MacOS/Info.plist"

    TARGET_PLIST="./target/release/bundle/osx/FortiFile.app/Contents/Info.plist"

    if [ -f "$TARGET_PLIST" ]; then
        cp "$CUSTOM_PLIST" "$TARGET_PLIST"
        echo "Custom Info.plist copied to the app bundle."
    else
        echo "Error: Target Info.plist does not exist. The bundle might have failed."
    fi
else
    echo "cargo bundle failed. Exiting."
    exit 1
fi
