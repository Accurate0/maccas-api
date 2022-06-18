#!/usr/bin/env bash
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip --bin api && aws lambda update-function-code --function-name MaccasApi-v2 --zip-file fileb://target/lambda/api/bootstrap.zip

cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip --bin service && aws lambda update-function-code --function-name MaccasApi-refresh-v2 --zip-file fileb://target/lambda/service/bootstrap.zip
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip --bin service && aws lambda update-function-code --function-name MaccasApi-refresh-v2 --zip-file fileb://target/lambda/service/bootstrap.zip --region ap-southeast-1
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip --bin service && aws lambda update-function-code --function-name MaccasApi-refresh-v2 --zip-file fileb://target/lambda/service/bootstrap.zip --region ap-northeast-1
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip --bin service && aws lambda update-function-code --function-name MaccasApi-refresh-v2 --zip-file fileb://target/lambda/service/bootstrap.zip --region ap-northeast-2
