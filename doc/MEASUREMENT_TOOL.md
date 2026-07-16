# Measurement Tool

[Documentation index](README.md) · [Project README](../README.md)

Use the measurement tool when you want to measure the distance between two points on an image or document.

![MView6 measurement tool](./images/mview6-measure.png)

## What you see

When the tool is on, MView6 shows two crosshairs:

* **Green crosshair**: the start point.
* **Cyan crosshair**: the finish point.
* **Arrow and values**: an arrow connects both points. The readout shows horizontal change, vertical change, and the straight-line distance.

> [!NOTE]
> Measurements in centimeters assume **600 DPI**. Pixel-based placement is taken from the original image or page, not from the current zoom level. You can zoom (deep) to precisely place the crosshairs.

---

## Controls

| Key / Control   | Action                  | Description                                                                                              |
| --------------- | ----------------------- | -------------------------------------------------------------------------------------------------------- |
| `F2`            | Toggle measurement      | Show or hide the measurement tool.                                                                       |
| `Tab`           | Switch active point     | Choose whether the mouse controls the start point or finish point.                                       |
| **Mouse click** | Place active point      | Click the image to place the currently active point.                                                     |
| **Mouse move**  | Preview measurement     | Move the mouse to update the arrow and values before placing the point.                                  |

---

## How the measurement works

MView6 uses the positions of the two points on the original image or page. Zooming in or out does not change the measurement.

* **Horizontal change** shows how far the finish point is left or right from the start point.
* **Vertical change** shows how far the finish point is above or below the start point.
* **Distance** shows the straight-line distance between both points.

For centimeter measurements, MView6 converts pixels using the document resolution. The default is 600 DPI.

> [!TIP]
> Use a flat-bed scanner to create an image of things you want to measure, precisely and without distortion
