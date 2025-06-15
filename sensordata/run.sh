#!/usr/bin/env bash
set -ex

trap 'pkill -P $$' SIGINT SIGTERM EXIT

timeout 120 adb wait-for-device shell 'while [[ -z $(getprop sys.boot_completed) ]]; do sleep 1; done;'

adb install app.apk
adb push frida-server /data/local/tmp/frida-server

adb shell chmod +x /data/local/tmp/frida-server
adb shell "su -c 'pkill -f -9 frida-server && /data/local/tmp/frida-server &'" &
adb shell monkey -p com.mcdonalds.au.gma 1
adb shell svc power stayon true

sleep 10

bun run server.js

pkill -P $$

wait
