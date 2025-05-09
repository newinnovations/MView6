#!/usr/bin/env python3

import subprocess
import sys


def get_shared_libraries(executable):
    """Get shared libraries required by the executable."""
    try:
        output = subprocess.check_output(["ldd", executable], text=True)
        libs = [line.split()[0] for line in output.split("\n") if line and "=>" in line]
        return libs
    except subprocess.CalledProcessError:
        print("Error: Could not retrieve shared libraries.")
        return []


IGNORE = ["i386", "lib32", "google-chrome", "microsoft-edge", "codium", "windsurf"]


def get_package_for_library(lib):
    """Find the package that provides a given shared library."""
    try:
        output = subprocess.check_output(
            ["dpkg", "-S", lib], text=True, stderr=subprocess.DEVNULL
        )

        packages = set()

        for line in output.splitlines():
            if any(word in line for word in IGNORE):
                continue
            if line.endswith("/" + lib):
                packages.add(line.split(":")[0])

        if len(packages) != 1:
            print("warning", lib, packages)

        return packages
    except subprocess.CalledProcessError:
        return f"{lib} -> Package not found"


def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <path_to_executable>")
        sys.exit(1)

    executable = sys.argv[1]
    # print(f"Checking dependencies for: {executable}")
    # print("=" * 40)

    shared_libs = get_shared_libraries(executable)

    if shared_libs:
        # print("\nShared libraries required:")
        # for lib in shared_libs:
        #     print(lib)

        # print("\nSearching for corresponding packages:")
        all = set()
        for lib in shared_libs:
            all.update(get_package_for_library(lib))

        all = list(all)
        all.sort()
        print(", ".join(all))
    else:
        print("No dependencies found or executable is statically compiled.")


if __name__ == "__main__":
    main()
