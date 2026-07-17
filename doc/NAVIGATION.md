# Navigation

[Documentation index](README.md) · [Project README](../README.md)

This guide covers the everyday keys and mouse actions for moving around in MView6. If you want to start MView6 from a terminal with a specific file, folder, sort order, or filter, see [Command Line Usage](CLI_USAGE.md).

## Help

Press `h` to show help inside the app. Press `h` again to switch to the second help page.

## Move through files

| Key(s)                                                | Function                                                          |
| ----------------------------------------------------- | ----------------------------------------------------------------- |
| `home`                                                | first image                                                       |
| `end`                                                 | last image                                                        |
| `z` _or_ `←` _or_ `↑` _or_ `numpad 4` _or_ `numpad 8` | previous image                                                    |
| `x` _or_ `→` _or_ `↓` _or_ `numpad 6` _or_ `numpad 2` | next image                                                        |
| `page up`                                             | previous 20 images                                                |
| `page down`                                           | next 20 images                                                    |
| `a`                                                   | previous __liked__ image                                          |
| `s`                                                   | next __liked__ image                                              |
| `numpad 7` _or_ `numpad home`                         | hop to previous folder/archive/document (explanation below)       |
| `numpad 9` _or_ `numpad page up`                      | hop to next folder/archive/document                               |
| `enter` _or_ `numpad enter`                           | enter (open) folder/archive/document                              |
| `backspace` _or_ `numpad decimal/del`                 | leave (close) folder/archive/document and return to parent folder |
| `w`                                                   | previous page (inside multi-page text files only)                 |
| `e`                                                   | next page (inside multi-page text files only)                     |

The Filter determines which files you can select using keyboard navigation. To temporarily bypass the filter, hold `Shift` while navigating. For advanced filter control, press `Shift + F`.

### Modifier keys

The move keys above (except `a`, `s`, `w`, `e`, `enter`, and `backspace`) can be combined with modifiers:

| Modifier | Effect                                                                                                       |
| -------- | ------------------------------------------------------------------------------------------------------------ |
| `Ctrl`   | move further: 5× the normal step for `z`/`x`/arrow keys; 50 images (instead of 20) for `page up`/`page down` |
| `Shift`  | ignore the active filter while moving                                                                        |
| `Alt`    | hop to the previous/next folder/archive/document instead of moving within it (explanation below)             |
|          |                                                                                                              |

---

### Hopping: what does "hop" mean?

If a folder contains several subfolders, archives or documents, hopping lets you jump from the one you are viewing to the previous or next one. You do not have to close the current folder or archive, move in the parent folder, and open the next one manually.

## Sort the file list

Click a table header, or use these keys:

| Key(s) | Function                                                      |
| ------ | ------------------------------------------------------------- |
| `1`    | sort on file type (folder/archive/image/document/video/other) |
| `2`    | sort on name                                                  |
| `3`    | sort on size                                                  |
| `4`    | sort on date                                                  |

## Use thumbnails

| Key(s) | Function                      |
| ------ | ----------------------------- |
| `t`    | open thumbnail view           |
| `m`    | cycle through thumbnail sizes |

## Bookmarks, image information, full screen, and exit

| Key(s)                  | Function                                    |
| ----------------------- | ------------------------------------------- |
| `q`                     | quit application                            |
| `d`                     | show bookmarks (edit in configuration file) |
| `f` _or_ `numpad *`     | toggle full screen                          |
| `esc`                   | exit full screen                            |
| `i`                     | toggle image information                    |
| `space` _or_ `numpad /` | toggle folder/archive view                  |

Bookmarks are stored in `$HOME/.config/mview/mview6.json`. Example:

```json
{
  "bookmarks": [
    {
      "name": "Home folder",
      "folder": "/home/martin"
    },
    {
      "name": "Pictures folder",
      "folder": "/home/martin/Pictures"
    },
    {
      "name": "Holiday 2024",
      "folder": "/home/martin/holiday_2024.zip"
    }
  ]
}
```

## Rotate the view

Rotation only changes what you see in MView6. It does not change the file on disk.

| Key(s)    | Function             |
| --------- | -------------------- |
| `r`       | rotate clockwise     |
| `shift r` | rotate anticlockwise |

## Zoom

You can zoom with the mouse wheel and pan by dragging. You can also use these keys:

| Key(s)                                   | Function                                                    |
| ---------------------------------------- | ----------------------------------------------------------- |
| `n`                                      | toggle between `no-zoom` and `zoom-fit`                     |
| `m` _or_ `numpad 0` _or_ `numpad insert` | cycle `zoom-fill` ➔ `zoom-max` ➔ `no-zoom` ➔ `zoom-fill`    |

In thumbnail view, `m` instead cycles through the available thumbnail sizes (see [Use thumbnails](#use-thumbnails)).

### Zoom modes

| Mode        | Behavior                                                                                                                |
| ----------- | ----------------------------------------------------------------------------------------------------------------------- |
| `no_zoom`   | Shows the image at its real size. Parts may fall outside the window.                                                    |
| `zoom_fit`  | Shrinks the image if needed so the whole image fits in the window. It does not enlarge small images.                    |
| `zoom_fill` | Shrinks or enlarges the image so the whole image is visible. There may be black margins.                                |
| `zoom_max`  | Shrinks or enlarges the image to fill the window. There are no black margins, but parts of the image may be off-screen. |

## Marking images as "liked" or "disliked"

| Key(s)              | Function                                        |
| ------------------- | ----------------------------------------------- |
| `=` _or_ `numpad +` | mark image as `liked` (or unmark as `disliked`) |
| `-` _or_ `numpad -` | mark image as `disliked` (or unmark as `liked`) |

MView6 marks files by renaming them. For example, `image_123.jpg` becomes `image_123.hi.jpg`. This currently does not work inside ZIP or RAR archives.

## Change page layout

For documents such as PDFs and EPUBs, you can switch between one-page and two-page layouts.

| Key(s) | Function                                                         |
| ------ | ---------------------------------------------------------------- |
| `p`    | Cycle layout: `Single` ➔ `Dual (Odd-Even)` ➔ `Dual (Even-Odd)`   |

## Command Palette

Use the Command Palette to search for and run actions without finding them in a menu.

| Key(s)             | Function                 |
| ------------------ | ------------------------ |
| `Ctrl + Shift + P` | Open the Command Palette |

For details on configuration and slideshow settings, see [Command Palette & Slideshow Mode](COMMAND_PALETTE_AND_SLIDESHOW.md).

## Saving & Clipboard

Copy images, paste images, save PNG files, and generate previews.

| Key(s)             | Function                                          |
| ------------------ | ------------------------------------------------- |
| `Ctrl + C`         | Copy whole loaded image to clipboard              |
| `Ctrl + Shift + C` | Copy current visible canvas area to clipboard     |
| `Ctrl + V`         | Paste image from clipboard                        |
| `Ctrl + S`         | Save original raster image data to a file         |
| `Ctrl + Shift + S` | Save visible canvas area to a file                |
| `c` (without Ctrl) | Generate/cache preview for archives, PDF or video |

For more details on deletion, trashing, and clipboard options, see [Saving, Clipboard & File Management](SAVING_AND_CLIPBOARD.md).

## Measurement Tool

Measure distances directly on images and documents.

| Key(s) | Function                                                       |
| ------ | -------------------------------------------------------------- |
| `F2`   | Toggle measurement tool                                        |
| `Tab`  | Alternate active endpoint tracking (Start vs Finish crosshair) |

For details on readouts and cm calculation, see [Measurement Tool Guide](MEASUREMENT_TOOL.md).

For building from source and installing system dependencies, see the [Installation guide](INSTALLATION.md).
