# Installation

[Documentation index](README.md) · [Project README](../README.md)

Detailed installation steps, system requirements, and troubleshooting for all platforms.

## Pre-built Binaries

### Windows

1. Download the latest `.msi` installer from the [releases page](https://github.com/newinnovations/MView6/releases).
2. Double-click the downloaded file and follow the installer.

> [!WARNING]
> The Windows installer is currently unsigned, so you may see a security warning when running it.
> This is expected: the code is compiled and the installer is automatically created by GitHub using GitHub Actions.
> The warning appears because we haven't purchased a code signing certificate.
>
> - If you see "Windows protected your PC", click **More info**, then **Run anyway**.
> - Some antivirus software may flag unsigned executables. This can happen with unsigned apps.

### Ubuntu/Debian

Currently only tested with Ubuntu 24.04.

1. Download the latest `.deb` package from the [releases page](https://github.com/newinnovations/MView6/releases).
2. Install using your package manager:

   ```bash
   sudo dpkg -i mview6_*.deb
   sudo apt-get install -f  # Install any missing dependencies
   ```

You can also double-click the `.deb` file and install it with your desktop's package installer.

## Building from Source

### Prerequisites

- Rust, latest stable version
- GTK4 development libraries
- Additional system dependencies may be required

#### Ubuntu

```bash
sudo apt-get install libglib2.0-dev libgdk-pixbuf-2.0-dev libgraphene-1.0-dev libgtk-4-dev librsvg2-dev
```

#### macOS

```bash
brew install pkg-config cairo gtk4
```

### Build and Install

```bash
git clone https://github.com/newinnovations/MView6.git
cd MView6
cargo build --release
cargo install --path .
```

The binary will be installed to `~/.cargo/bin/mview6`. Make sure that directory is in your `PATH`.

## System Requirements

| Requirement | Minimum                                                |
| ----------- | ------------------------------------------------------ |
| Windows     | Windows 10 or later                                    |
| Linux       | GTK4 support (most modern distributions)               |
| Memory      | 512MB RAM minimum, 2GB recommended for large documents |
| Storage     | 50MB for installation                                  |

## Troubleshooting

### Windows

> [!TIP]
> - If the installer does not run, try running it as administrator.
> - If antivirus software quarantines the installer, add an exception if you trust the download.

### Linux

> [!TIP]
> - If dependencies are missing after installing the `.deb` package, run `sudo apt-get install -f`.
> - For other Linux distributions, you may need to build from source.

### General

> [!TIP]
> - For the best performance, keep your graphics drivers up to date.
> - If a specific file format does not open correctly, check that you are using the latest MView6 release.
