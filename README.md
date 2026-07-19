# MView6

<img src="./resources/mview6.svg" height="120" align="right">

[![Built with Rust][built-with-badge]][built-with-url]
[![GitHub Actions Workflow Status][build-badge]][build-url]
[![GitHub Release][release-badge]][release-url]

[built-with-badge]: https://img.shields.io/badge/built_with-Rust,_GTK4_and_PDFium--rs-darkgreen?logo=rust
[built-with-url]: https://github.com/newinnovations/pdfium-rs
[build-badge]: https://img.shields.io/github/actions/workflow/status/newinnovations/MView6/release.yml?logo=github
[build-url]: https://github.com/newinnovations/MView6/actions/workflows/release.yml
[release-badge]: https://img.shields.io/github/v/release/newinnovations/MView6?logo=github
[release-url]: https://github.com/newinnovations/MView6/releases

**High-performance PDF and photo viewer built with Rust and GTK4**

MView6 helps you browse large folders of images, PDFs, e-books, videos, and archives without getting in your way. Open a folder, ZIP, RAR, PDF, or image, then move through everything quickly with the keyboard, mouse, thumbnails, or slideshow mode. Built on modern, performance-focused technologies such as Rust and GTK4.

![MView6 browsing photos inside an archive](./doc/images/mview6.png)

## Gallery

Browse the [gallery](doc/GALLERY.md) for more screenshots and see MView6 in action.

## Key Features

| Category                    | Highlights                                                                                                                                                                                                                                                                                                                                                              |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Browse folders and archives | Open ZIP and RAR files without extracting them first.<br>Move between folders, archives, images, documents, and videos from one file list.<br>Jump between nearby folders or archives while keeping your place.                                                                                                                                                         |
| View images and documents   | Supports PDF, EPUB, JPEG, PNG, SVG, HEIC, AVIF, animated GIF/WEBP, and more.<br>Choose fit-to-window, fill-window, maximum zoom, or actual-size viewing.<br>Read PDFs and e-books one page at a time or in a two-page layout.<br>Rotate images and pages without changing the original file.<br>Show photo details such as camera settings and GPS data when available. |
| Text and binary files       | View text files with syntax highlighting.<br>Inspect binary files with the hexdump facility.                                                                                                                                                                                                                                                                            |
| Move quickly                | Use keyboard shortcuts for fast browsing.<br>Zoom with the mouse wheel and drag to pan.<br>Open thumbnails to scan a whole folder visually.<br>Keep your browsing position as you move through folders and archives.                                                                                                                                                    |
| Organize and inspect        | Mark images as liked or disliked.<br>Sort files by type, name, size, or date.<br>Copy images, save PNG exports, create previews or contact sheets for video, archives and documents.<br>Delete files or move files to trash.<br>Measure distances directly on images and documents.                                                                                     |

## Documentation & User Guide

Start here when you want to learn a specific part of MView6, or browse the full [documentation index](doc/README.md):

- [Navigation](doc/NAVIGATION.md) — the most useful keys, mouse controls, zoom options, sorting, and thumbnails.
- [Saving, Clipboard & File Management](doc/SAVING_AND_CLIPBOARD.md) — copying, pasting, saving as PNG files, creating previews/contact sheets, and deleting safely.
- [Measurement Tool](doc/MEASUREMENT_TOOL.md) — measuring pixel or physical distances on images and documents.
- [Command Palette & Slideshow](doc/COMMAND_PALETTE_AND_SLIDESHOW.md) — finding commands quickly and playing a folder as a slideshow.
- [Command Line Usage](doc/CLI_USAGE.md) — opening MView6 from a terminal with a file, folder, sort order, or filter.

## Installation

### Windows

Download the latest `.msi` installer from the [releases page](https://github.com/newinnovations/MView6/releases) and run it.

> [!NOTE]
> The installer is currently unsigned, so Windows may show a security warning. See the [Installation guide](doc/INSTALLATION.md) for details on why this happens and how to proceed.

### Ubuntu/Debian

Download the latest `.deb` package from the [releases page](https://github.com/newinnovations/MView6/releases) and install it:

```bash
sudo dpkg -i mview6_*.deb
sudo apt-get install -f  # Install any missing dependencies
```

### Building from Source

```bash
git clone https://github.com/newinnovations/MView6.git
cd MView6
cargo run --release
```

For system requirements, dependency installation, and troubleshooting, see the full [Installation guide](doc/INSTALLATION.md).

## Contributing

MView6 is developed in Rust with GTK4. Contributions are welcome through pull requests and issue reports.

## License

MView6 is free software: you can redistribute it and/or modify it under the terms of
the GNU Affero General Public License as published by the Free Software Foundation, either
version 3 of the License, or (at your option) any later version.
