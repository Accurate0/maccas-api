#!/usr/bin/env bash
pushd api || exit
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-v2 --zip-file fileb://target/lambda/api/bootstrap.zip
popd || exit

pushd refresh || exit
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-refresh-v2 --zip-file fileb://target/lambda/refresh/bootstrap.zip
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-refresh-v2 --zip-file fileb://target/lambda/refresh/bootstrap.zip --region ap-southeast-1
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-refresh-v2 --zip-file fileb://target/lambda/refresh/bootstrap.zip --region ap-northeast-1
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-refresh-v2 --zip-file fileb://target/lambda/refresh/bootstrap.zip --region ap-northeast-2
popd || exit
