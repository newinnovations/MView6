#!/bin/bash

executable=$1
deps_dir="target/release/deps"
output_dir="mview6-windows/bin"

copy_dependencies() {
    local file=$1
    local dependencies=$(objdump -p "$file" | grep 'DLL Name:' | sed 's/.*DLL Name: //')

    for dll in $dependencies; do
        if [ -f "$deps_dir/$dll" ]; then
            if [ ! -f "$output_dir/$dll" ]; then
                cp "$deps_dir/$dll" "$output_dir/"
                echo "   COPY: copied $dll to $output_dir"
                copy_dependencies "$deps_dir/$dll"
            else
                echo "   SKIP: $dll already in $output_dir"
            fi
        else
            echo "UNAVAIL: $dll not found in $deps_dir"
        fi
    done
}

copy_dependencies "$executable"
