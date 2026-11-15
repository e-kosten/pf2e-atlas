#!/bin/sh
set -eu

contents="$1/Contents"
plist="$contents/Info.plist"
resources="$contents/Resources"
icon_name=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIconName' "$plist")
fallback=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIconFile' "$plist")

test "$fallback" = Atlas.icns
test -s "$resources/$fallback"
test -s "$resources/Assets.car"
assetutil --info "$resources/Assets.car" | jq -e --arg name "$icon_name" \
    'any(.[]; .AssetType == "Icon Image" and .Name == $name)' >/dev/null
