#!/usr/bin/env bash
pushd api || exit
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi --zip-file fileb://target/lambda/api/bootstrap.zip
popd || exit

pushd deals || exit
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-deals --zip-file fileb://target/lambda/deals/bootstrap.zip
popd || exit

cat accounts-sydney.yml > accounts.yml
pushd refresh || exit
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-refresh --zip-file fileb://target/lambda/refresh/bootstrap.zip
popd || exit

cat accounts-singapore.yml > accounts.yml
pushd refresh || exit
cargo lambda build --release --target x86_64-unknown-linux-musl --output-format zip && aws lambda update-function-code --function-name MaccasApi-refresh --zip-file fileb://target/lambda/refresh/bootstrap.zip --region ap-southeast-1
popd || exit
