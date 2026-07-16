# Saving, Clipboard & File Management

[Documentation index](README.md) · [Project README](../README.md)

This guide explains how to copy images, paste images, save PNG files, create faster previews, and delete files safely.

## Copy and paste

MView6 can use your system clipboard, so you can copy from MView6 into another app or paste an image into MView6 for a quick look.

| Shortcut           | Action            | Description                                                                    |
| ------------------ | ----------------- | ------------------------------------------------------------------------------ |
| `Ctrl + C`         | Copy whole image  | Copy the full loaded image or document page.                                   |
| `Ctrl + Shift + C` | Copy visible area | Copy only the part currently visible on screen, including zoom or rotation.    |
| `Ctrl + V`         | Paste image       | Paste an image from the clipboard into MView6 for temporary viewing.           |

---

## Save to a file

You can save exported images through the normal file save dialog. Exports are currently saved as **PNG**.

* **`Ctrl + S`** saves the complete original raster image.
* **`Ctrl + Shift + S`** saves what you currently see on screen, including zoom, cropping, or rotation.

---

## Create previews for faster loading

Very large images, PDFs, and videos can take a moment to render the first time. You can create a cached preview so they open faster later.

Select a file in the file list and press `c` without Ctrl. MView6 creates the preview in the background and shows progress while it works.

---

## Delete files

MView6 supports both safe trashing and permanent deletion.

### Move to trash

Select a file and press `Delete`. MView6 marks the file with a trash icon and shows a message such as `Move '[filename]' to trash`.

While the message is visible, you can undo the action by clicking **Undo** or by pressing `Escape`, `Enter`, or `Space`. If you press `Delete` on more files while the message is still visible, they are added to the same pending trash action.

When the message disappears or is dismissed, MView6 moves the files to your system trash.

### Delete permanently

Select a file and press `Shift + Delete`. MView6 asks for confirmation before permanently removing the file or directory from disk.
