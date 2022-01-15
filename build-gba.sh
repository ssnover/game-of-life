#!/bin/sh

cargo build --release || exit 1
arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/game-of-life target/game-of-life.gba || exit 1
gbafix target/game-of-life.gba || exit 1
echo "Successfully built ROM!"