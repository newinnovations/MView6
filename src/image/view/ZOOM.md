# Zoom, Rotation & Mirroring — Developer Guide

This document explains how MView6 positions, scales, rotates and mirrors images
on screen. It is aimed at developers working on `src/image/view/zoom.rs` and
the code around it (`src/image/model.rs`, `src/image/view/view_obj.rs`,
`src/image/view/data/*`). It intentionally goes deeper than the rustdoc
comments in the source, focusing on *why* things are built this way and how
the pieces fit together.

If you only remember one thing: **`Zoom` is a small, cheaply-cloned value type
that fully describes "how the image maps onto the screen right now"**. Nearly
everything in the image view — drawing, hit-testing, cropped re-rendering,
measurement tools — goes through it.

## 1. The big picture

```text
             ┌─────────────────────────────────────────────┐
             │                  Zoom                       │
             │                                             │
             │ scale, rotation, mirror, offset, image_size │
             └─────────────────────────────────────────────┘
                  │                                  │
  image_to_screen / screen_to_image          transform_matrix()
                  │                                  │
                  ▼                                  ▼
        hit-testing, measurement,           cairo::Matrix used by
        overlays, drag anchoring            RenderedImage/SingleImage/
                                            DualImage draw() calls
```

`Zoom` lives in `src/image/view/zoom.rs` and is owned by `ImageViewData`
(`src/image/view/data/model.rs`) as the field `zoom`. `ImageView`
(`src/image/view/view_obj.rs`) exposes it to the rest of the application
through small wrapper methods (`rotate`, `mirror`, `zoom_in`, `zoom_out`,
`is_mirrored`, ...), which the window layer (`src/window/window_imp/*.rs`)
wires up to menu items, keyboard shortcuts, and the command palette.

## 2. Coordinate systems

Two coordinate systems matter:

- **Image coordinates**: the original, untransformed pixel space of the
  content, `(0, 0)` to `(image_size.width(), image_size.height())`. Y grows
  downward, same as image/pixel conventions.
- **Screen coordinates**: the `ImageView` drawing area, also Y-down. This is
  what Cairo draws into and what mouse events report.

`Zoom` converts between the two. The full pipeline, applied to a point in
image space, is:

```text
  scale → rotate → mirror → translate (offset)
```

i.e. `image_to_screen(p) == rotate(scale(p)).mirror().translate(offset)`,
and `screen_to_image` walks it backwards:

```text
  un-translate → un-mirror → un-rotate → un-scale
```

This order matters and is discussed in detail in §4 (mirroring).

### Helper types (`src/rect.rs`)

- `VectorPoint<T>` (aliased as `PointD`/`VectorD` for `T = f64`) — a 2D value
  with `translate`, `scale`, `unscale`, `neg`, `rotate(±90/180/270)`.
  `PointD` and `VectorD` are literally the same type; the two names exist
  purely for readability at call sites (a "point" is a location, a "vector"
  is an offset/direction).
- `Rect<T>` (aliased `RectD`) — an axis-aligned rectangle defined by two
  corners (`x0, y0`)–(`x1, y1`). Has `translate`, `scale`, `rotate`,
  `intersect`, `center`, `width`/`height`.
- `Rect::rotate(rotation)` only supports the four 90°-multiples used by
  `Zoom`; it rotates both corners as vectors and re-derives a normalized
  (`x0 ≤ x1`, `y0 ≤ y1`) rectangle from the min/max of the results. This is
  *not* a general-purpose rectangle rotation (a rotated rectangle is not
  axis-aligned in general) — it only works because we restrict ourselves to
  multiples of 90°.

## 3. Rotation

### 3.1 State & normalization

`Zoom.rotation: i32` is always one of `0, 90, 180, 270` (never negative,
never anything else). Two setters exist:

- `set_rotation(angle)` — sets an absolute angle.
- `add_rotation(delta)` — adds a relative delta (used for "rotate 90°
  clockwise/counterclockwise" actions).

Both funnel through `normalize_rotation`, which rounds to the nearest
multiple of 90 and reduces modulo 360 via `rem_euclid` (so negative deltas,
e.g. `-90`, normalize to `270`, never `-90`).

### 3.2 Direction convention

Angles increase **counterclockwise** as displayed on screen. This falls out
of `VectorPoint::rotate`:

```rust
pub fn rotate(&self, rotation: i32) -> Self {
    match rotation {
        -90 | 270 => Self::new(self.y, T::default() - self.x),
        -180 | 180 => Self::new(T::default() - self.x, T::default() - self.y),
        -270 | 90  => Self::new(T::default() - self.y, self.x),
        _ => Self::new(self.x, self.y),
    }
}
```

Concretely, in the window layer (`src/window/window_imp/menu.rs`):

```rust
rotate_submenu.append(Some("90° Clockwise"),        Some("win.rotate::270")); // add_rotation(270) == add_rotation(-90)
rotate_submenu.append(Some("90° Counterclockwise"), Some("win.rotate::90"));  // add_rotation(90)
```

So `add_rotation(90)` is counterclockwise, `add_rotation(270)` (equivalently
`-90`) is clockwise. Keyboard shortcuts (`src/window/window_imp/keyboard.rs`)
follow the same convention: `r` → `rotate_image(270)` (clockwise), `R`
(Shift+r) → `rotate_image(90)` (counterclockwise), `Ctrl+R` → mirror (see §4).

### 3.3 What rotation does to the image rectangle

`image_rect_rotated()` returns `RectD::new_from_size(image_size).rotate(rotation)`
— the image's own `(0,0)-(w,h)` box, rotated about the origin and
renormalized. At 90°/270° this swaps width and height (as you'd expect: a
portrait image rotated 90° becomes landscape-shaped on screen).

This is the basis for:

- `image_rect_rotated_scaled()` — same, times `scale`.
- `image_rect_transformed()` — same, translated by `offset`; this is the
  image's on-screen bounding box, used by `intersection_screen_coord`.
- `apply_zoom()` (§5) — uses the *rotated* size (not the raw image size) to
  compute a fit/fill/max zoom factor and to center the image, so that
  rotated portrait/landscape images are still fitted correctly.

### 3.4 The rendering matrix for pure rotation

`transform_matrix()` builds a `cairo::Matrix` directly from a small lookup
table (no trigonometry, since we only ever need 0/90/180/270°):

| rotation | `xx`    | `yx`    | `xy`    | `yy`    |
|---------:|--------:|--------:|--------:|--------:|
| 0        | `scale` | 0       | 0       | `scale` |
| 90       | 0       | `scale` | `-scale`| 0       |
| 180      | `-scale`| 0       | 0       | `-scale`|
| 270      | 0       | `-scale`| `scale` | 0       |

Translation (`x0, y0`) is simply `offset.x(), offset.y()` — **no extra
correction is needed for rotation alone**, because `offset` is always
computed (by `apply_zoom`/`update_zoom`) to already describe where the
rotated bounding box's origin belongs on screen. This is unlike mirroring,
which *does* need a correction — see §4.4.

### 3.5 `top_left()` — mapping a screen rect back to a "logical" corner

When the render thread finishes rendering a crop of the image, the code
needs to know: "which corner of this on-screen rectangle corresponds to the
*origin* (pixel `(0,0)`) of the rendered surface, given the current
rotation?" That's `top_left(rect)`. As the image rotates, a different
corner of the screen-space bounding rect becomes the "start" corner:

```text
             ┌─────┐
             │180° │   ┌────────┐
             │     │   │    270°│
             │   TL│   │TL      │
             └─────┘   └────────┘
                       ────→ x
          ┌────────┐ │ ┌─────┐
          │      TL│ │ │TL   │
          │90°     │ ↓ │     │
          └────────┘ y │   0°│
                       └─────┘
```

(`TL` marks, for each rotation, which corner of the screen rect is where the
surface's own `(0,0)` pixel ends up.) This value becomes the `origin` stored
in `RenderedImage` (see §6).

## 4. Mirroring

### 4.1 What "mirror" means here

`Zoom.mirror: bool` represents a horizontal (left/right) flip, toggled with
`toggle_mirror()` / set explicitly with `set_mirror(bool)`, and queried with
`is_mirrored()`. There is currently no vertical-flip flag — see §8 if you
need to add one.

**Contract: mirroring always flips left/right *as currently displayed on
screen*, regardless of the current rotation.** This is a deliberate design
decision (see `test_mirror_always_flips_screen_left_right` in `zoom.rs`) and
is why the implementation applies the flip in *screen-space, after
rotation* rather than in the image's own coordinate space before rotation:
if you flipped the image before rotating it, a 90°-rotated image would
appear flipped top/bottom instead of left/right, which is not what a "flip
left/right" menu command should do at any rotation.

### 4.2 Why a naive reflection isn't enough

The mathematically simplest mirror is `x → -x`. But naively negating `x`
reflects about the coordinate origin, which is *not* where the image's
bounding box lives after scaling/rotation/translation — it would fly the
image off to one side instead of flipping it in place. Every mirror-aware
computation in `zoom.rs` therefore needs to reflect about the correct axis
*and* compensate so the on-screen bounding box stays anchored exactly where
it was (see `test_mirror_bounding_box_unchanged`).

### 4.3 The shared reflection helpers

Three private helpers on `Zoom` centralize this axis/anchor math so it is
derived and tested in one place:

```rust
/// (x0 + x1) of the image's rotated (unscaled) bounding box.
fn mirror_axis_sum(&self) -> f64 { ... }

/// mirror_axis_sum(), scaled -- the axis to reflect about in
/// rotated/scaled (pre-translation) screen space.
fn mirror_shift(&self) -> f64 { self.scale * self.mirror_axis_sum() }

/// Reflects a single on-screen x-coordinate about mirror_shift().
fn reflect_x(&self, x: f64) -> f64 { self.mirror_shift() - x }
```

Every coordinate-space method that needs to mirror a point/rect calls
`reflect_x` (or, for matrices, `reflect_matrix_x` — see §4.4) instead of
re-deriving `shift - x` inline. **If you need to touch the mirror math,
change it here — don't reintroduce a local copy of this formula.**

Call sites and what they do with it:

| Method                       | What gets reflected                                             |
| ---------------------------- | --------------------------------------------------------------- |
| `transform_matrix()`         | the matrix's screen-x row, plus a translation correction (§4.4) |
| `top_left(rect)`             | which of `rect.x0`/`rect.x1` is treated as the "left" edge      |
| `intersection_image_coord()` | the viewport rect's x-extent, before un-rotating/un-scaling     |
| `screen_to_image(point)`     | the point's x, before un-rotating/un-scaling                    |
| `image_to_screen(point)`     | the point's x, after scaling/rotating, before translating       |

`top_left` is a special case: it doesn't use `reflect_x`/`mirror_shift` at
all, because it isn't reflecting a *value* — it's choosing which of the two
existing corners (`rect.x0` vs `rect.x1`) plays the role of "left", so a
simple swap suffices:

```rust
let (x0, x1) = if self.mirror { (rect.x1, rect.x0) } else { (rect.x0, rect.x1) };
```

### 4.4 The rendering matrix for mirroring

`transform_matrix()` builds the rotation/scale matrix as before (§3.4), then
if `mirror` is set, reflects it:

```rust
let matrix = Matrix::new(a, b, c, d, e, f); // rotation + scale + translate
if self.mirror {
    let shift = self.mirror_shift();
    let reflected = Self::reflect_matrix_x(matrix);
    Matrix::new(reflected.xx(), reflected.yx(), reflected.xy(), reflected.yy(),
                reflected.x0() + shift, reflected.y0())
} else {
    matrix
}
```

`reflect_matrix_x` is a small `pub` associated function (used outside
`zoom.rs` too — see §6) that negates the components producing the on-screen
x-coordinate, leaving translation untouched:

```rust
pub fn reflect_matrix_x(matrix: Matrix) -> Matrix {
    Matrix::new(-matrix.xx(), matrix.yx(), -matrix.xy(), matrix.yy(),
                matrix.x0(), matrix.y0())
}
```

Because a pure reflection about `x = 0` would otherwise move the image, the
caller adds `shift = mirror_shift()` to the translation (`x0`) to re-anchor
the bounding box exactly where the unmirrored image would have been. This
`+ shift` correction is *only* valid when the matrix's untranslated part
covers the full image content at the current `image_size` (i.e. for the
"whole image" case in `Zoom::transform_matrix`); see §6 for why the cropped
`RenderedImage` case must *not* add it.

### 4.5 Where mirror state resets

- `set_content_pre` (`view_obj.rs`) resets `mirror` to `false` whenever a new
  piece of content is loaded, alongside setting rotation from EXIF/content
  metadata. Mirroring is a per-view/session choice, not a persisted content
  property (unlike rotation, which can come from EXIF).
- `Zoom::reset()` also clears it (via `Default`).

## 5. Zoom (scale) & positioning

Independent of rotation/mirror, `Zoom` also owns:

- `scale: f64` — current zoom factor (`1.0` = original size), clamped to
  `[MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR]` (`0.001`–`300.0`).
- `offset: VectorD` — screen-space position of the image's transformed
  origin (see `origin()`/`set_origin()`/`set_offset()`).

Two entry points mutate both together:

- **`apply_zoom(zoom_mode, image_size, viewport)`** — used whenever content,
  viewport size, rotation, or the zoom *mode* changes. Computes `scale` from
  `ZoomMode` (`NoZoom`/`Fit`/`Fill`/`Max`, see doc comments on `ZoomMode`)
  against the *rotated* image size, then centers the rotated+scaled image
  in the viewport by setting `offset` to `viewport.center() -
  image_rect_rotated_scaled().center()`. Mirroring does not affect this
  centering calculation (a mirrored image has the same bounding box as the
  unmirrored one — see `test_mirror_bounding_box_unchanged`).
- **`update_zoom(new_zoom, anchor)`** — "zoom to point": rescales while
  keeping the image content under `anchor` (typically the cursor, or the
  viewport center for keyboard zoom) visually fixed. Used for scroll-wheel /
  `+`/`-` zoom.

`ZoomMode` (`NotSpecified`/`NoZoom`/`Fit`/`Fill`/`Max`) is the user-facing
"how should this image be sized" intent; it's resolved to a concrete `scale`
by `apply_zoom`. See `ImageViewData::apply_zoom` (`data/model.rs`) for how a
per-content `zoom_mode` (e.g. forced `NoZoom` for tiny images) interacts
with the view's global `zoom_mode` setting.

## 6. Rendering pipeline & cropped re-renders

Most content types (`SingleImage`, `DualImage`, animations) are drawn
directly with `current_image_zoom.transform_matrix()` — see
`Image::transform_matrix` in `src/image/model.rs`. Nothing special is needed
there: those surfaces are always the *whole* image content, so the general
formula in §3.4/§4.4 applies as-is.

Vector/paginated content (SVG, PDF via mupdf/pdfium) is different: rendering
happens asynchronously on a render thread (`self.content.render(zoom, viewport)`
in `ImageViewData::redraw_quality`), which only rasterizes the *visible
crop* of the content (via `Zoom::intersection_image_coord`/`intersection`)
at the current zoom — not the whole page. The result comes back as a
`RenderedImage` (`src/image/model.rs`) carrying:

- `surface` — the rasterized crop (unrotated, unmirrored pixels — rotation
  and mirroring are never baked into the pixels themselves, only into how
  the surface is subsequently drawn).
- `origin` — where this crop's `(0,0)` pixel belongs on screen, computed via
  `zoom.top_left(&rect)` at render time (`data/redraw.rs::event_render_done`).
- `orig_image_zoom` — a full clone of the `Zoom` that was active when this
  crop was rendered.

Two problems make `RenderedImage::transform_matrix` more involved than just
calling `zoom.transform_matrix()`:

1. **The current zoom may have moved on** since the render was kicked off
   (the render thread is async; the user may have panned/zoomed further
   while waiting). The matrix must interpolate: scale by
   `current_image_zoom.scale() / orig_image_zoom.scale()` and re-anchor the
   origin accordingly. This is unrelated to rotation/mirror and isn't
   covered further here — see the method body.
2. **The full-image mirror formula doesn't apply to a crop.** `mirror_shift()`
   is derived from the *full* `image_size`'s rotated bounding box, but the
   surface here represents an arbitrarily-sized, arbitrarily-positioned
   crop. Reusing the full-image formula would reflect about the wrong axis
   and misplace the crop.

   The fix: build a *mirror-free* rotation+scale+translate matrix (temporarily
   clearing `mirror` on the cloned `Zoom`), then — only if the original zoom
   was mirrored — reflect it with the *same* `Zoom::reflect_matrix_x` helper
   used internally by `transform_matrix()`, but **without** the `+ shift`
   translation correction. No correction is needed here because `origin`
   (computed via `top_left`, which *is* mirror-aware) already places the
   crop at the correct on-screen position; reflecting the matrix about that
   already-correct local origin is enough.

```rust
let mirrored = zoom.is_mirrored();
zoom.set_mirror(false);
let matrix = zoom.transform_matrix(); // mirror-free
if mirrored {
    Zoom::reflect_matrix_x(matrix)     // no extra translation needed here
} else {
    matrix
}
```

If you add a new content type that renders cropped surfaces, follow this
same pattern rather than calling `zoom.transform_matrix()` directly with
mirroring left on.

### 6.1 Dual-page PDF spreads: resolve mirroring once, at the full-spread level

`Pages::Dual` mode (`src/backends/document/{pdfium,mupdf}.rs::render_dual`)
renders two PDF pages into a single crop, side by side, backed by *two*
independent page objects (and two independent native rasterizers) rather
than one. This is a variation on §6's "cropped re-render" pattern, and it
has its own mirror pitfall worth calling out.

`render_dual` is handed one `Zoom` (`zoom`) whose `image_size` is the size
of the **whole spread** (left + scaled right page, computed in
`page_size_dual`), and must decide which pixels of *each individual page*
to rasterize for the current `viewport`. An earlier version of this code
cloned `zoom` once per page (`zoom_left`, `zoom_right`), gave each clone its
own page-sized `image_size`, and let each clone independently compute its
own crop via `Zoom::intersection`/`intersection_image_coord`. This works
fine while mirroring is off (the composite mapping is a plain linear
scale+translate, so restricting it to a sub-range and computing it directly
on that sub-range give the same answer either way), but breaks once
`mirror` is on: `mirror_shift()` (§4.3) is derived from the *clone's own*
(page-sized) `image_size`, not the full spread's, so each page's clone
reflects about the wrong axis. As long as the whole spread was fully inside
the viewport, both wrong axes still happened to yield the same "everything
visible" crop, hiding the bug — but as soon as the user zoomed in or panned
so only part of the spread was visible, the two clones' independently
(and differently) miscomputed axes produced inconsistent, shifted crops:
pages appeared to slide/overlap instead of clipping cleanly at the viewport
edge.

The fix follows the same principle as problem 2 above: **resolve the mirror
reflection exactly once, at the level that knows the true full-width axis**,
and keep everything derived from it mirror-free:

1. Call `zoom.intersection_image_coord(viewport)` **once**, on the original,
   full-spread `zoom`. This yields `crop`, the visible region in natural
   (unmirrored, unsplit) spread-image coordinates — the one and only place
   the mirror axis (based on the true total spread width) is used.
2. Split `crop` between the two pages with plain rectangle intersection
   against each page's known sub-range (`[0, width_left]` and
   `[width_left, width_left + scale_right * width_right]`), converting the
   right page's slice back to its own raw coordinates by undoing the
   `width_left` offset and the `scale_right` height-matching factor.
3. Rasterize each page directly from its own (now mirror-free) crop
   rectangle and scale — see `page_render_crop` — instead of routing back
   through a per-page `Zoom` clone that could re-derive (and get wrong) its
   own mirror axis.

As in §6, the two rasterized page buffers are stitched together (in natural
left-then-right order, never internally flipped — see `SurfaceData::from_dual_bgra8`/
`from_dual_rgb`) into one `RenderedImage`, whose *single* `orig_image_zoom`
(the full-spread `zoom`, unmodified) is what `RenderedImage::transform_matrix`
later uses to apply the one, correct, whole-crop mirror flip at draw time.

## 7. UI wiring

```text
window_imp/menu.rs         "win.mirror" action, "Mirror (flip left/right)" menu item
window_imp/keyboard.rs     Ctrl+R  → mirror_image()   (r / Shift+R → rotate)
window_imp/commands.rs     Command palette entry, shortcut "ctrl+shift+r"
window_imp/actions.rs      mirror_image(): calls ImageView::mirror(), syncs
                           the "mirror" GAction's boolean state for menu checkmarks
image/view/view_obj.rs     ImageView::mirror() / is_mirrored(): toggles
                           Zoom::mirror, re-applies zoom, drops any stale
                           zoom_overlay, and redraws
```

`ImageView::mirror()` (and `rotate()`) both clear `p.zoom_overlay` after
changing the transform. `zoom_overlay` holds the last `RenderedImage` (§6);
since it was rendered for the *previous* rotation/mirror state, it must be
discarded so a fresh render is requested rather than displaying stale pixels
under a wrong/hacky matrix.

`RedrawReason::MirrorChanged` (`data/redraw.rs`) mirrors (pun intended) the
existing `RotationChanged` reason: both are treated as "high quality, but
nothing to show yet if a re-render was just kicked off" in
`redraw_quality()`. If you add more transform-affecting state in the future,
follow the same pattern: add a `RedrawReason` variant and include it
alongside `RotationChanged`/`MirrorChanged` wherever they're matched
together, rather than only alongside one of them.

## 8. Extending this (e.g. adding vertical flip)

If you need a vertical mirror (flip top/bottom) in the future:

- Add a second bool (e.g. `flip_v`) to `Zoom`, mirroring the structure of
  `mirror`/`set_mirror`/`toggle_mirror`/`is_mirrored`.
- You will need a `mirror_axis_sum_y` / `reflect_y` counterpart to §4.3's
  helpers (analogous, but operating on `rect.y0 + rect.y1` and the matrix's
  `yy`/`yx` components).
- Update `reflect_matrix_x` (or add a sibling `reflect_matrix_y`) — resist
  the temptation to hand-negate matrix components at additional call sites;
  route everything through one shared helper as described in §4.3/§4.4, or
  future maintainers will end up with the same "reflection logic duplicated
  N times" problem this document was written to avoid.
- Remember `top_left()` and `RenderedImage::transform_matrix()` both need to
  learn about the new flip — they're the two places outside the "obvious"
  math helpers that silently assume there's only one mirror axis.

## 9. Testing

`zoom.rs`'s `#[cfg(test)] mod tests` is the source of truth for expected
behavior; when changing any of the above, run:

```sh
cargo test zoom
```

Tests worth knowing about if you're touching mirror/rotation:

- `test_mirror_always_flips_screen_left_right` — mirroring must swap
  on-screen left/right at *every* rotation (0/90/180/270), not just at 0°.
- `test_mirror_bounding_box_unchanged` — mirroring must not move or resize
  the image's on-screen bounding box.
- `test_mirror_coordinate_round_trip` — `image_to_screen` /
  `screen_to_image` must remain exact inverses of each other with mirroring
  enabled, at every rotation.
- `test_mirror_transformation_matrix` — pins down the exact matrix
  coefficients and translation for a known scale/offset/size.
- `test_add_rotation` / `test_rotation_normalization` — the 90°-snapping and
  wraparound behavior of rotation deltas.
