#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p build/widget-tests
xcrun swiftc -swift-version 6 -module-cache-path build/widget-tests/module-cache \
    platform/ios/WeatherWidget/WidgetStrings.swift \
    platform/ios/WeatherWidget/WeatherData.swift scripts/widget_data_tests.swift \
    -o build/widget-tests/data-tests
build/widget-tests/data-tests
