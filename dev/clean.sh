#!/bin/bash

set -e

cd $(dirname "${BASH_SOURCE[0]}")
OD="$(pwd)"

rm -rf APT RUST nestopia snes9x

