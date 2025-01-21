#!/usr/bin/env bash
set -ex

trap 'pkill -P $$' SIGINT SIGTERM EXIT

command -v emulator > /dev/null && emulator=emulator || emulator=/android-sdk/emulator/emulator

rm -f "$HOME"/.android/avd/test.avd/*.lock

export QEMU_AUDIO_DRV=none
$emulator -avd test -no-window -read-only -no-metrics -memory 2048 &

timeout 120 adb wait-for-device shell 'while [[ -z $(getprop sys.boot_completed) ]]; do sleep 1; done;'

adb root

adb install app.apk
adb push frida-server /data/local/tmp/frida-server

adb shell chmod +x /data/local/tmp/frida-server
adb shell /data/local/tmp/frida-server &
adb shell monkey -p com.mcdonalds.au.gma 1
adb shell svc power stayon true

sleep 10

bun run server.js

pkill -P $$

wait
