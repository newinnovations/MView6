#!/bin/bash

SCRIPT_DIR=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")

cd ${SCRIPT_DIR}/../..

# docker run --rm -ti -v .:/opt/mview6 rust:1-trixie /bin/bash
docker run --rm -v .:/opt/mview6 rust:1-trixie /bin/bash -c /opt/mview6/tools/rpi/build-cross-rpi.sh
