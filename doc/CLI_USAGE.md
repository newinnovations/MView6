# Command Line Usage

[Documentation index](README.md) · [Project README](../README.md)

You can start MView6 from a terminal and optionally tell it what to open, how to sort files, and which file types to show.

## Basic command

```bash
mview6 [FILE OR DIRECTORY] [OPTIONS]
```

`[FILE OR DIRECTORY]` is optional. If you leave it out, MView6 opens the current folder and selects the last file you visited or the first file.

---

## Options

| Option / Flag           | Description                                                             | Default Value          |
| ----------------------- | ----------------------------------------------------------------------- | ---------------------- |
| `-h, --help`            | Show command line help and exit.                                        | N/A                    |
| `-V, --version`         | Show the installed version and exit.                                    | N/A                    |
| `-s, --sort <SORT>`     | Choose the starting sort order. See [Sort options](#sort-options).      | `0a` (type, ascending) |
| `-f, --filter <FILTER>` | Choose which file types to show. See [Filter options](#filter-options). | `all`                  |

---

### Sort options

Use these values with `--sort` or `-s`:

| Value | Sort Mode       | Description                                                |
| ----- | --------------- | ---------------------------------------------------------- |
| `0a`  | Type ascending  | Group by folders, archives, images, documents, and videos. |
| `0d`  | Type descending | Group by file type in the opposite order.                  |
| `1a`  | Name ascending  | Sort file names from A to Z.                               |
| `1d`  | Name descending | Sort file names from Z to A.                               |
| `2a`  | Size ascending  | Show the smallest files first.                             |
| `2d`  | Size descending | Show the largest files first.                              |
| `3a`  | Date ascending  | Show the oldest files first.                               |
| `3d`  | Date descending | Show the newest files first.                               |

---

### Filter options

The Filter determines which files you can select using keyboard navigation. To temporarily bypass the filter, hold `Shift` while navigating. For advanced filter control, press `Shift + F` in the application.

Use these values with `--filter` or `-f`:

| Value      | Filtered Type | Description                                                                         |
| ---------- | ------------- | ----------------------------------------------------------------------------------- |
| `all`      | Everything    | All files (default).                                                                |
| `image`    | Images        | Static and animated image files, such as JPEG, PNG, GIF, SVG, WEBP, AVIF, and HEIC. |
| `video`    | Videos        | Video files.                                                                        |
| `document` | Documents     | Document files, such as PDF and EPUB.                                               |
| `archive`  | Archives      | ZIP and RAR archives.                                                               |

---

## Examples

**Open a folder:**

  ```bash
  mview6 ~/Pictures
  ```

**Open MView6 with the newest files first:**

  ```bash
  mview6 -s 3d
  ```

**Show only documents in a folder:**

  ```bash
  mview6 -f document ~/Documents/Reference
  ```

**Start at a specific picture and sort by name:**

  ```bash
  mview6 -s 1a ~/Pictures/vacation/photo_001.jpg
  ```
