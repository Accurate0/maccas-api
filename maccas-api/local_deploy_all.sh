#!/usr/bin/env bash
pushd api || exit
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-v2 --zip-file fileb://target/lambda/api/bootstrap.zip
popd || exit

pushd deals || exit
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-deals-v2 --zip-file fileb://target/lambda/deals/bootstrap.zip
popd || exit
