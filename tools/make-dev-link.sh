#!/bin/bash

# For development purposes, this script creates a symbolic link for the MView6 binary in /usr/bin/
# First install MView6 using the .deb package, then run this script to create the symbolic link.

SCRIPT_DIR=$(realpath "$(dirname "$0")/..")
MVIEW6_PATH="$SCRIPT_DIR/target/release/MView6"

echo "Creating symbolic link for MView6 in /usr/bin/"
echo "Linking $MVIEW6_PATH to /usr/bin/mview6"

sudo rm -f /usr/bin/mview6
sudo ln -s "$SCRIPT_DIR/target/release/MView6" /usr/bin/mview6
