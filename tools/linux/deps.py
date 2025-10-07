#!/usr/bin/env python3

import subprocess
import sys

error = False


def get_shared_libraries(executable):
    """Get shared libraries required by the executable."""
    try:
        output = subprocess.check_output(
            ["ldd", executable], text=True, stderr=subprocess.DEVNULL
        )
        libs = [line.split()[0] for line in output.split("\n") if line and "=>" in line]
        return libs
    except subprocess.CalledProcessError:
        # print("Error: Could not retrieve shared libraries.")
        return []


IGNORE = ["i386", "lib32", "google-chrome", "microsoft-edge", "codium", "windsurf"]


def get_package_for_library(lib):
    """Find the package that provides a given shared library."""
    global error
    try:
        output = subprocess.check_output(
            ["dpkg", "-S", lib], text=True, stderr=subprocess.DEVNULL
        )

        packages = set()

        for line in output.splitlines():
            if any(word in line for word in IGNORE):
                continue
            if line.endswith(f"/{lib}"):
                package = line.split(":")[0]
                version = get_package_version(package)
                packages.add(f"{package} (>= {version})")

        if len(packages) != 1:
            print("error", lib, packages)
            error = True

        return packages
    except subprocess.CalledProcessError:
        print(f"{lib} -> Package not found")
        error = True
        return set()


def get_package_version(package):
    """Find the version of an installed package."""
    global error
    try:
        result = subprocess.run(
            f"dpkg -s {package} | grep -oP 'Version:\\s*\\K[^+-]+'",
            shell=True,
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        print(f"{package} -> Package not found")
        error = True
        return "0.0.0"


def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <path_to_executable>")
        sys.exit(1)

    executable = sys.argv[1]
    shared_libs = get_shared_libraries(executable)

    if shared_libs:
        all = set()
        for lib in shared_libs:
            all.update(get_package_for_library(lib))

        if error:
            sys.exit(2)

        all = list(all)
        all.sort()
        print(", ".join(all))
    else:
        # Cross compilation (for now)
        FALLBACK = [
            "libcairo2 (>= 1.18.0)",
            "libdav1d7 (>= 1.4.1)",
            "libgdk-pixbuf-2.0-0 (>= 2.42.10)",
            "libgtk-4-1 (>= 4.14.5)",
            "libpango-1.0-0 (>= 1.52.1)",
        ]
        print(", ".join(FALLBACK))
        # print("No dependencies found or executable is statically compiled.")
        # sys.exit(3)


if __name__ == "__main__":
    main()
