#!/bin/bash
LAMBDA_ARCH="linux/arm64"
RUST_TARGET="aarch64-unknown-linux-gnu"
RUST_VERSION="latest"

docker run --platform ${LAMBDA_ARCH} --rm --user "$(id -u)":"$(id -g)" \
  -v "${PWD}":/usr/src/lambda-app -w /usr/src/lambda-app rust:${RUST_VERSION} \
  cargo build --release --target ${RUST_TARGET}

zip -j lambda.zip ./target/${RUST_TARGET}/release/bootstrap sync-config.yaml
aws lambda update-function-code --function-name $1 --zip-file fileb://lambda.zip --publish --region $2 $3
rm lambda.zip
