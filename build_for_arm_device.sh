#!/bin/bash

set -e

source './.env';


if [ -z "$AURORA_SDK_PATH" ]; then
    echo 'No `AURORA_SDK_PATH` variable is set.';
    exit 1;
fi

if [ -z "$BUILD_TARGET" ]; then
    echo 'No `BUILD_TARGET` variable is set.';
    exit 1;
fi

if [ -z "$DEPLOY_DEVICE" ]; then
    echo 'No `DEPLOY_DEVICE` variable is set.';
    exit 1;
fi


SFDK_PATH="$AURORA_SDK_PATH/bin/sfdk"
CURRENT_DIR="$(pwd)";

# cargo update
cargo build --release --target=armv7-unknown-linux-gnueabihf

cp ./target/armv7-unknown-linux-gnueabihf/release/lets_habr ./com.lmaxyz.LetsHabr

$SFDK_PATH config target="AuroraOS-5.1.1.60-base-armv7hl"
# $SFDK_PATH config device="$DEPLOY_DEVICE"

$SFDK_PATH build
$SFDK_PATH engine exec -tt sb2 -t AuroraOS-5.1.1.60-base-armv7hl rpmsign-external sign -k $CURRENT_DIR/../.auroraos-regular-keys/regular_key.pem -c $CURRENT_DIR/../.auroraos-regular-keys/regular_cert.pem $CURRENT_DIR/RPMS/com.lmaxyz.LetsHabr-0.1-1.armv7hl.rpm

set +e

rm ./com.lmaxyz.LetsHabr
rm ./documentation.list
