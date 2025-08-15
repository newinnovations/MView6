// MView6 -- High-performance PDF and photo viewer built with Rust and GTK4
//
// Copyright (c) 2024-2025 Martin van der Werff <github (at) newinnovations.nl>
//
// This file is part of MView6.
//
// MView6 is free software: you can redistribute it and/or modify it under the terms of
// the GNU Affero General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
// IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR
// BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use cairo::Matrix;

use crate::rect::{RectD, SizeD};

/// Maximum allowed zoom factor
pub const MAX_ZOOM_FACTOR: f64 = 30.0;
/// Minimum allowed zoom factor
pub const MIN_ZOOM_FACTOR: f64 = 0.02;
/// Standard zoom increment/decrement multiplier for smooth zoom operations
pub const ZOOM_MULTIPLIER: f64 = 1.05;

/// Floating point comparison epsilon for zoom state detection
/// Used to handle floating-point precision issues when comparing zoom factors
const ZOOM_EPSILON: f64 = 1.0e-6;

/// Defines how an image should be (initially) positioned and scaled within the viewport.
///
/// This enum represents the user's intent for how the image should be displayed,
/// which is then translated into specific zoom and positioning calculations.
///
/// User intent may be overridden by specifying a `ZoomMode` different than
/// `NotSpecified` at image level.
#[derive(Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub enum ZoomMode {
    /// No specific zoom mode specified - typically uses default behavior
    #[default]
    NotSpecified,
    /// Display image at its original size (1:1 pixel ratio)
    NoZoom,
    /// Scale image down to fit entirely within viewport, maintaining aspect ratio
    /// Will not scale up if image is smaller than viewport
    Fit,
    /// Scale image to fill viewport completely, maintaining aspect ratio, will
    /// not crop image
    Fill,
    /// Scale image to fill viewport completely, using the larger scaling factor
    /// Similar to Fill but always uses the maximum possible zoom, will crop
    /// parts of the image if aspect ratios don't match
    Max,
}

impl From<&str> for ZoomMode {
    /// Converts string literals to ZoomMode enum values
    ///
    /// # Arguments
    /// * `value` - String slice containing the zoom mode name
    ///
    /// # Returns
    /// * Corresponding ZoomMode enum value, or NotSpecified for unknown strings
    fn from(value: &str) -> Self {
        match value {
            "nozoom" => ZoomMode::NoZoom,
            "fit" => ZoomMode::Fit,
            "fill" => ZoomMode::Fill,
            "max" => ZoomMode::Max,
            _ => ZoomMode::NotSpecified,
        }
    }
}

impl From<ZoomMode> for &str {
    /// Converts ZoomMode enum values to their string representations
    ///
    /// # Arguments
    /// * `value` - ZoomMode enum value to convert
    ///
    /// # Returns
    /// * String slice representing the zoom mode, or empty string for NotSpecified
    fn from(value: ZoomMode) -> Self {
        match value {
            ZoomMode::NotSpecified => "",
            ZoomMode::NoZoom => "nozoom",
            ZoomMode::Fit => "fit",
            ZoomMode::Fill => "fill",
            ZoomMode::Max => "max",
        }
    }
}

/// Represents the current zoom state of the image relative to its original size.
///
/// This is determined by comparing the current zoom factor to 1.0 (original size)
/// with floating-point tolerance for comparison.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub enum ZoomState {
    /// Image is displayed at original size (zoom = 1.0)
    NoZoom,
    /// Image is enlarged (zoom > 1.0)
    ZoomedIn,
    /// Image is reduced (zoom < 1.0)
    ZoomedOut,
}

/// Manages the zoom, rotation, and positioning state of an image within a viewport.
///
/// This struct handles the complex coordinate system transformations needed for:
/// - Zooming in/out while maintaining a visual anchor point
/// - Rotating images in 90-degree increments
/// - Centering images within the viewport
/// - Handling the coordinate system changes that occur with rotation
///
/// The coordinate system uses two types of offsets:
/// - `screen_off_x/y`: Positions the image within the viewport
/// - `image_off_x/y`: Corrects for coordinate system origin changes during rotation
#[derive(Debug, Clone)]
pub struct Zoom {
    /// Current zoom factor (1.0 = original size)
    zoom: f64,
    /// Rotation angle in degrees (0, 90, 180, 270)
    rotation: i32,
    /// Horizontal offset to center image in viewport
    offset_x: f64,
    /// Vertical offset to center image in viewport
    offset_y: f64,
    /// Original image dimensions (width, height) before any transformations
    image_size: SizeD,
}

impl Default for Zoom {
    /// Creates a Zoom instance with default values (no zoom, no rotation, no offset)
    fn default() -> Self {
        Self {
            zoom: 1.0,
            rotation: Default::default(),
            offset_x: Default::default(),
            offset_y: Default::default(),
            image_size: Default::default(),
        }
    }
}

impl Zoom {
    /// Creates a new Zoom with default values
    ///
    /// Equivalent to calling `Zoom::default()`
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets all zoom, rotation, and positioning to default values
    ///
    /// This effectively returns the image to its original state:
    /// - Zoom factor: 1.0 (original size)
    /// - Rotation: 0 degrees
    /// - Offsets: 0.0 (no translation)
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Determines the current zoom state by comparing zoom factor to 1.0
    ///
    /// Uses floating-point epsilon comparison to handle precision issues.
    ///
    /// # Returns
    /// * `ZoomState` indicating whether image is zoomed in, out, or at original size
    pub fn state(&self) -> ZoomState {
        if self.zoom > 1.0 + ZOOM_EPSILON {
            ZoomState::ZoomedIn
        } else if self.zoom < 1.0 - ZOOM_EPSILON {
            ZoomState::ZoomedOut
        } else {
            ZoomState::NoZoom
        }
    }

    /// Returns the horizontal offset used for positioning the image in the viewport
    pub fn offset_x(&self) -> f64 {
        self.offset_x
    }

    /// Returns the vertical offset used for positioning the image in the viewport
    pub fn offset_y(&self) -> f64 {
        self.offset_y
    }

    /// Sets both horizontal and vertical offsets for image positioning
    ///
    /// # Arguments
    /// * `offset_x` - New horizontal offset in screen coordinates
    /// * `offset_y` - New vertical offset in screen coordinates
    pub fn set_offset(&mut self, offset_x: f64, offset_y: f64) {
        self.offset_x = offset_x;
        self.offset_y = offset_y;
    }

    /// Sets the rotation angle, constraining it to 90-degree increments (0, 90, 180, 270)
    ///
    /// Input values are automatically normalized to the nearest valid rotation angle.
    ///
    /// # Arguments
    /// * `rotation` - Rotation angle in degrees (will be rounded to nearest 90-degree increment)
    pub fn set_rotation(&mut self, rotation: i32) {
        self.rotation = Self::normalize_rotation(rotation);
    }

    /// Adds to the current rotation angle, constraining the result to 90-degree increments
    ///
    /// This is useful for relative rotation operations like "rotate 90 degrees clockwise"
    /// or "rotate 180 degrees" from the current orientation.
    ///
    /// # Arguments
    /// * `delta` - Rotation angle to add in degrees (will be rounded to nearest 90-degree increment)
    ///
    /// # Examples
    /// ```
    /// let mut zoom = Zoom::new();
    /// zoom.add_rotation(90);   // Now at 90 degrees
    /// zoom.add_rotation(90);   // Now at 180 degrees
    /// zoom.add_rotation(-90);  // Now at 90 degrees
    /// ```
    pub fn add_rotation(&mut self, delta: i32) {
        self.rotation = Self::normalize_rotation(self.rotation + delta);
    }

    /// Normalizes rotation to the nearest 90-degree increment (0, 90, 180, 270)
    ///
    /// This ensures that rotation values are always compatible with the matrix
    /// calculations, which only handle these specific angles.
    ///
    /// # Arguments
    /// * `rotation` - Input rotation angle in degrees
    ///
    /// # Returns
    /// * Normalized rotation angle as one of: 0, 90, 180, or 270 degrees
    fn normalize_rotation(rotation: i32) -> i32 {
        // Round to nearest 90-degree increment, then normalize to 0-359 range
        let rounded = ((rotation as f64 / 90.0).round() as i32) * 90;
        rounded.rem_euclid(360)
    }

    /// Creates a Cairo transformation matrix for rendering the image
    ///
    /// This matrix combines:
    /// - Scaling (zoom factor)
    /// - Rotation (in 90-degree increments)
    /// - Translation (positioning offsets)
    ///
    /// The matrix transforms from image coordinates to screen coordinates.
    ///
    /// # Returns
    /// * `Matrix` - Cairo transformation matrix ready for rendering operations
    pub fn transform_matrix(&self) -> Matrix {
        match self.rotation % 360 {
            90 => Matrix::new(
                0.0,
                self.zoom,
                -self.zoom,
                0.0,
                self.offset_x(),
                self.offset_y(),
            ),
            180 => Matrix::new(
                -self.zoom,
                0.0,
                0.0,
                -self.zoom,
                self.offset_x(),
                self.offset_y(),
            ),
            270 => Matrix::new(
                0.0,
                -self.zoom,
                self.zoom,
                0.0,
                self.offset_x(),
                self.offset_y(),
            ),
            _ => Matrix::new(
                self.zoom,
                0.0,
                0.0,
                self.zoom,
                self.offset_x(),
                self.offset_y(),
            ),
        }
    }

    /// Creates an unscaled transformation matrix for high-resolution overlay rendering
    ///
    /// This matrix is used when displaying hi-res rendered overlays (such as PDF and SVG content)
    /// where the overlay is pre-rendered at the target resolution rather than using Cairo's
    /// built-in scaling. The matrix handles rotation and translation but omits the zoom scaling
    /// since the overlay content is already rendered at the correct scale.
    ///
    /// The screen offsets are clamped to positive values to ensure proper positioning. Negative
    /// values will be handled during rendering of the high-resolution overlay by transposing
    /// the source content in the target overlay by that amount. Which needs the overlay to be
    /// positioned at the origin.
    ///
    /// # Arguments
    /// * `size` - overlay pixmap dimensions as SizeD (width, height)
    ///
    /// # Returns
    /// * `Matrix` - Unscaled transformation matrix for overlay positioning
    pub fn unscaled_transform_matrix(&self, size: SizeD) -> Matrix {
        self.new_unscaled(size).transform_matrix()
    }

    /// Creates an unscaled [`Zoom`] instance for high-resolution rendering
    ///
    /// This matrix is used when displaying hi-res rendered overlays (such as PDF and SVG content)
    /// where the overlay is pre-rendered at the target resolution rather than using Cairo's
    /// built-in scaling. The matrix handles rotation and translation but omits the zoom scaling
    /// since the overlay content is already rendered at the correct scale.
    ///
    /// The screen offsets are clamped to positive values to ensure proper positioning. Negative
    /// values will be handled during rendering of the high-resolution overlay by transposing
    /// the source content in the target overlay by that amount. Which needs the overlay to be
    /// positioned at the origin.
    ///
    /// # Arguments
    /// * `size` - overlay pixmap dimensions as SizeD (width, height)
    ///
    /// # Returns
    /// * `Zoom` - Unscaled instance for overlay positioning
    pub fn new_unscaled(&self, size: SizeD) -> Self {
        let i = self.image_rect_scaled_offset();
        let x0 = i.x0.max(0.0);
        let y0 = i.y0.max(0.0);
        let (offset_x, offset_y) = match self.rotation % 360 {
            90 => (x0 + size.height(), y0),
            180 => (x0 + size.width(), y0 + size.height()),
            270 => (x0, y0 + size.width()),
            _ => (x0, y0),
        };
        Self {
            zoom: 1.0,
            rotation: self.rotation,
            offset_x,
            offset_y,
            image_size: size,
        }
    }

    /// Returns the image rectangle in its current rotation, but without scaling or offset
    ///
    /// This accounts for coordinate system changes due to rotation:
    /// - 0°: Normal coordinates (0,0 to width,height)
    /// - 90°: X becomes -Y, Y becomes X,
    /// - 180°: Both X and Y are negated
    /// - 270°: X becomes Y, Y becomes -X,
    ///
    /// # Returns
    /// * `RectD` - Image bounds in rotated coordinate system
    pub fn image_rect(&self) -> RectD {
        match self.rotation % 360 {
            90 => RectD::new(-self.image_size.height(), 0.0, 0.0, self.image_size.width()),
            180 => RectD::new(
                -self.image_size.width(),
                -self.image_size.height(),
                0.0,
                0.0,
            ),
            270 => RectD::new(0.0, -self.image_size.width(), self.image_size.height(), 0.0),
            _ => RectD::new(0.0, 0.0, self.image_size.width(), self.image_size.height()),
        }
    }

    /// Returns the image rectangle after applying zoom scaling but before offset positioning
    ///
    /// # Returns
    /// * `RectD` - Image bounds with rotation and zoom applied
    pub fn image_rect_scaled(&self) -> RectD {
        self.image_rect().scale(self.zoom)
    }

    /// Returns the final image rectangle after all transformations (rotation, zoom, offset)
    ///
    /// This represents the actual screen position and size of the image.
    ///
    /// # Returns
    /// * `RectD` - Final image bounds in screen coordinates
    pub fn image_rect_scaled_offset(&self) -> RectD {
        self.image_rect_scaled()
            .translate(self.offset_x(), self.offset_y())
    }

    /// Calculates the intersection between the transformed image and the viewport
    ///
    /// This determines which portion of the image is actually visible on screen.
    ///
    /// # Arguments
    /// * `viewport` - The visible area rectangle
    ///
    /// # Returns
    /// * `RectD` - Intersection area between image and viewport
    pub fn intersection(&self, viewport: &RectD) -> RectD {
        self.image_rect_scaled_offset().intersect(viewport)
    }

    /// Calculates the required pixmap size for rendering the visible intersection
    ///
    /// For rotated images (90° and 270°), width and height are swapped to account
    /// for the coordinate system transformation.
    ///
    /// # Arguments
    /// * `intersection` - Visible area to be rendered
    ///
    /// # Returns
    /// * `SizeD` - Required dimensions for the rendering pixmap
    pub fn pixmap_size(&self, intersection: &RectD) -> SizeD {
        match self.rotation_degrees() {
            90 | 270 => SizeD::new(intersection.height(), intersection.width()),
            _ => SizeD::new(intersection.width(), intersection.height()),
        }
    }

    /// Calculates the offset within the source image for rendering the visible intersection
    ///
    /// This determines which part of the original image corresponds to the visible area,
    /// accounting for rotation transformations.
    ///
    /// # Arguments
    /// * `intersection` - Visible area being rendered
    ///
    /// # Returns
    /// * `(f64, f64)` - Source image offset coordinates (x, y)
    pub fn pixmap_offset(&self, intersection: &RectD) -> (f64, f64) {
        let rect = self.image_rect_scaled_offset();
        match self.rotation_degrees() {
            270 => (intersection.y1 - rect.y1, rect.x0 - intersection.x0),
            180 => (intersection.x1 - rect.x1, intersection.y1 - rect.y1),
            90 => (rect.y0 - intersection.y0, intersection.x1 - rect.x1),
            _ => (rect.x0 - intersection.x0, rect.y0 - intersection.y0),
        }
    }

    /// Applies the specified zoom mode to fit the image within the given viewport
    ///
    /// This method calculates the appropriate zoom factor and positioning based on:
    /// - The desired zoom mode (fit, fill, max, etc.)
    /// - Current rotation angle (affects effective image dimensions)
    /// - Viewport size (allocation rectangle)
    /// - Image dimensions
    ///
    /// # Arguments
    /// * `zoom_mode` - How the image should be scaled and positioned
    /// * `image_size` - Original image dimensions (width, height)
    /// * `viewport` - Viewport rectangle where image will be displayed
    ///
    /// # Requirements
    /// This method requires valid (positive) image dimensions. Zero or negative
    /// dimensions may cause division by zero or unexpected behavior and will
    /// not be accepted.
    pub fn apply_zoom(&mut self, zoom_mode: ZoomMode, image_size: SizeD, viewport: RectD) {
        self.image_size = image_size;

        // Account for rotation when calculating effective image size
        // Rotations of 90° and 270° swap width and height
        let image_rect = self.image_rect();

        // Validate effective image dimensions
        if image_rect.width() <= 0.0 || image_rect.height() <= 0.0 {
            eprintln!(
                "Warning: Invalid effective image dimensions ({}, {})",
                image_rect.width(),
                image_rect.height()
            );
            return;
        }

        // Calculate zoom factor based on the specified mode
        let zoom = if zoom_mode == ZoomMode::NoZoom {
            1.0
        } else {
            // Calculate zoom factors for both dimensions
            let zoom_x = viewport.width() / image_rect.width();
            let zoom_y = viewport.height() / image_rect.height();

            match zoom_mode {
                ZoomMode::Max => {
                    // Max: Use the larger zoom factor (may crop image)
                    zoom_x.max(zoom_y)
                }
                ZoomMode::Fit => {
                    // Fit: Use smaller zoom factor, but don't scale up small images
                    if viewport.width() > image_rect.width()
                        && viewport.height() > image_rect.height()
                    {
                        1.0
                    } else {
                        zoom_x.min(zoom_y)
                    }
                }
                _ => {
                    // Fill: Use smaller zoom factor to fit entirely within viewport
                    zoom_x.min(zoom_y)
                }
            }
        };

        // Apply zoom constraints
        self.zoom = zoom.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);

        // Center the image within the viewport
        let (vp_center_x, vp_center_y) = viewport.center();
        let (image_center_x, image_center_y) = self.image_rect_scaled().center();
        self.offset_x = vp_center_x - image_center_x;
        self.offset_y = vp_center_y - image_center_y;
    }

    /// Updates the zoom factor while maintaining a visual anchor point
    ///
    /// This method implements "zoom to point" functionality, where the image
    /// is scaled around a specific point (typically the mouse cursor position).
    /// The visual content at the anchor point remains stationary while the
    /// rest of the image scales around it.
    ///
    /// # Arguments
    /// * `new_zoom` - Desired zoom factor (will be clamped to valid range)
    /// * `anchor` - Point in screen coordinates to zoom around (x, y)
    ///
    /// # Example
    /// ```
    /// // Zoom in 2x around the center of a 800x600 viewport
    /// image_zoom.update_zoom(2.0, (400.0, 300.0));
    /// ```
    pub fn update_zoom(&mut self, new_zoom: f64, anchor: (f64, f64)) {
        let new_zoom = new_zoom.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);

        // Early return if zoom hasn't actually changed
        if (new_zoom - self.zoom).abs() < ZOOM_EPSILON {
            return;
        }

        let (anchor_x, anchor_y) = anchor;

        // Calculate the point in image coordinates that corresponds to the anchor
        let view_cx = (anchor_x - self.offset_x()) / self.zoom;
        let view_cy = (anchor_y - self.offset_y()) / self.zoom;

        // Calculate new offsets so the anchor point remains visually stationary
        self.set_offset(anchor_x - view_cx * new_zoom, anchor_y - view_cy * new_zoom);

        // Apply the new zoom factor
        self.zoom = new_zoom;
    }

    /// Returns the current zoom factor
    ///
    /// # Returns
    /// * `f64` - Current zoom factor (1.0 = original size)
    pub fn zoom_factor(&self) -> f64 {
        self.zoom
    }

    /// Returns the current rotation angle in degrees
    ///
    /// # Returns
    /// * `i32` - Rotation angle (0, 90, 180, or 270 degrees)
    pub fn rotation_degrees(&self) -> i32 {
        self.rotation
    }

    /// Checks if the image is currently rotated (not at 0 degrees)
    ///
    /// # Returns
    /// * `bool` - True if image is rotated, false if at 0 degrees
    pub fn is_rotated(&self) -> bool {
        self.rotation % 360 != 0
    }

    /// Checks if the image is currently zoomed (not at 1.0 zoom factor)
    ///
    /// Uses epsilon comparison to handle floating-point precision issues.
    ///
    /// # Returns
    /// * `bool` - True if image is zoomed in or out, false if at original size
    pub fn is_zoomed(&self) -> bool {
        self.state() != ZoomState::NoZoom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test rectangle
    fn test_rect(width: i32, height: i32) -> RectD {
        RectD::new(0.0, 0.0, width as f64, height as f64)
    }

    // Helper function to compare floating point values with tolerance
    fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() < tolerance
    }

    #[test]
    fn test_zoom_mode_string_conversion() {
        assert_eq!(ZoomMode::from("fit"), ZoomMode::Fit);
        assert_eq!(ZoomMode::from("fill"), ZoomMode::Fill);
        assert_eq!(ZoomMode::from("max"), ZoomMode::Max);
        assert_eq!(ZoomMode::from("nozoom"), ZoomMode::NoZoom);
        assert_eq!(ZoomMode::from("invalid"), ZoomMode::NotSpecified);

        assert_eq!(<&str>::from(ZoomMode::Fit), "fit");
        assert_eq!(<&str>::from(ZoomMode::Fill), "fill");
        assert_eq!(<&str>::from(ZoomMode::Max), "max");
        assert_eq!(<&str>::from(ZoomMode::NoZoom), "nozoom");
        assert_eq!(<&str>::from(ZoomMode::NotSpecified), "");
    }

    #[test]
    fn test_image_zoom_default() {
        let zoom = Zoom::default();
        assert_eq!(zoom.rotation, 0);
        assert_eq!(zoom.zoom, 1.0);
        assert_eq!(zoom.offset_x(), 0.0);
        assert_eq!(zoom.offset_y(), 0.0);
        assert_eq!(zoom.state(), ZoomState::NoZoom);
    }

    #[test]
    fn test_image_zoom_new_and_reset() {
        let mut zoom = Zoom::new();
        zoom.zoom = 2.0;
        zoom.set_rotation(90);
        zoom.set_offset(10.0, 20.0);

        zoom.reset();
        assert_eq!(zoom.rotation, 0);
        assert_eq!(zoom.zoom, 1.0);
        assert_eq!(zoom.offset_x(), 0.0);
        assert_eq!(zoom.offset_y(), 0.0);
    }

    #[test]
    fn test_zoom_state() {
        let mut zoom = Zoom::new();

        // Test NoZoom state
        zoom.zoom = 1.0;
        assert_eq!(zoom.state(), ZoomState::NoZoom);

        // Test small variations around 1.0 (should still be NoZoom due to epsilon)
        zoom.zoom = 1.0 + ZOOM_EPSILON / 2.0;
        assert_eq!(zoom.state(), ZoomState::NoZoom);
        zoom.zoom = 1.0 - ZOOM_EPSILON / 2.0;
        assert_eq!(zoom.state(), ZoomState::NoZoom);

        // Test ZoomedIn state
        zoom.zoom = 1.5;
        assert_eq!(zoom.state(), ZoomState::ZoomedIn);

        // Test ZoomedOut state
        zoom.zoom = 0.5;
        assert_eq!(zoom.state(), ZoomState::ZoomedOut);
    }

    #[test]
    fn test_rotation_normalization() {
        let mut zoom = Zoom::new();

        // Test basic 90-degree increments
        zoom.set_rotation(90);
        assert_eq!(zoom.rotation, 90);

        zoom.set_rotation(180);
        assert_eq!(zoom.rotation, 180);

        zoom.set_rotation(270);
        assert_eq!(zoom.rotation, 270);

        zoom.set_rotation(360);
        assert_eq!(zoom.rotation, 0);

        // Test negative rotations
        zoom.set_rotation(-90);
        assert_eq!(zoom.rotation, 270);

        zoom.set_rotation(-180);
        assert_eq!(zoom.rotation, 180);

        // Test rounding to nearest 90-degree increment
        zoom.set_rotation(45);
        assert_eq!(zoom.rotation, 90);

        zoom.set_rotation(130);
        assert_eq!(zoom.rotation, 90);

        zoom.set_rotation(225);
        assert_eq!(zoom.rotation, 270);

        // Test values that should round to 0
        zoom.set_rotation(44);
        assert_eq!(zoom.rotation, 0);

        zoom.set_rotation(-44);
        assert_eq!(zoom.rotation, 0);
    }

    #[test]
    fn test_add_rotation() {
        let mut zoom = Zoom::new();

        // Test clockwise rotation
        zoom.add_rotation(90);
        assert_eq!(zoom.rotation, 90);

        zoom.add_rotation(90);
        assert_eq!(zoom.rotation, 180);

        zoom.add_rotation(90);
        assert_eq!(zoom.rotation, 270);

        zoom.add_rotation(90);
        assert_eq!(zoom.rotation, 0);

        // Test counter-clockwise rotation
        zoom.add_rotation(-90);
        assert_eq!(zoom.rotation, 270);

        // Test large increments
        zoom.add_rotation(450); // Should be equivalent to 90 degrees
        assert_eq!(zoom.rotation, 0);

        // Test rounding with add_rotation
        zoom.set_rotation(0);
        zoom.add_rotation(40); // Should round to 0 (40 rounds to 0 when added to 0)
        assert_eq!(zoom.rotation, 0);

        zoom.add_rotation(50); // Should round to 90 (50 rounds to 90)
        assert_eq!(zoom.rotation, 90);
    }

    #[test]
    fn test_offset_operations() {
        let mut zoom = Zoom::new();

        // Test basic offset setting
        zoom.set_offset(10.0, 20.0);
        assert_eq!(zoom.offset_x(), 10.0);
        assert_eq!(zoom.offset_y(), 20.0);
    }

    #[test]
    fn test_apply_zoom_no_zoom() {
        let mut zoom = Zoom::new();
        let image_size = SizeD::new(100.0, 200.0);
        let viewport = test_rect(400, 300);

        zoom.apply_zoom(ZoomMode::NoZoom, image_size, viewport);

        assert_eq!(zoom.zoom, 1.0);
        // Image should be centered in viewport
        assert_eq!(zoom.offset_x, 150.0); // (400 - 100) / 2
        assert_eq!(zoom.offset_y, 50.0); // (300 - 200) / 2
    }

    #[test]
    fn test_apply_zoom_fit() {
        let mut zoom = Zoom::new();
        let image_size = SizeD::new(200.0, 400.0); // 2:1 aspect ratio
        let viewport = test_rect(400, 300); // 4:3 aspect ratio

        zoom.apply_zoom(ZoomMode::Fit, image_size, viewport);

        // Should scale to fit height (limiting factor)
        assert_eq!(zoom.zoom, 0.75); // 300 / 400 = 0.75

        // Test fit mode with small image (should not scale up)
        let small_image = SizeD::new(50.0, 50.0);
        zoom.apply_zoom(ZoomMode::Fit, small_image, viewport);
        assert_eq!(zoom.zoom, 1.0); // Should not scale up
    }

    #[test]
    fn test_apply_zoom_fill() {
        let mut zoom = Zoom::new();
        let image_size = SizeD::new(200.0, 400.0); // 2:1 aspect ratio
        let viewport = test_rect(400, 300); // 4:3 aspect ratio

        zoom.apply_zoom(ZoomMode::Fill, image_size, viewport);

        // Should scale to fit width (smaller scaling factor)
        assert_eq!(zoom.zoom, 0.75); // min(400/200, 300/400) = min(2.0, 0.75) = 0.75
    }

    #[test]
    fn test_apply_zoom_max() {
        let mut zoom = Zoom::new();
        let image_size = SizeD::new(200.0, 400.0); // 2:1 aspect ratio
        let viewport = test_rect(400, 300); // 4:3 aspect ratio

        zoom.apply_zoom(ZoomMode::Max, image_size, viewport);

        // Should scale to fill completely (larger scaling factor)
        assert_eq!(zoom.zoom, 2.0); // max(400/200, 300/400) = max(2.0, 0.75) = 2.0
    }

    #[test]
    fn test_apply_zoom_with_rotation() {
        let mut zoom = Zoom::new();
        let image_size = SizeD::new(100.0, 200.0);
        let viewport = test_rect(400, 300);

        // Test with 90-degree rotation (dimensions should be swapped)
        zoom.set_rotation(90);

        // With 90° rotation, effective size is (200, 100)
        // Scaling factors: 400/200 = 2.0, 300/100 = 3.0

        // Fit mode does not need scaling
        zoom.apply_zoom(ZoomMode::Fit, image_size, viewport);
        assert_eq!(zoom.zoom, 1.0);

        // Fill mode should use smaller factor: 2.0
        zoom.apply_zoom(ZoomMode::Fill, image_size, viewport);
        assert_eq!(zoom.zoom, 2.0);

        // Max mode should use smaller factor: 2.0
        zoom.apply_zoom(ZoomMode::Max, image_size, viewport);
        assert_eq!(zoom.zoom, 3.0);

        // // Check image offsets for 90-degree rotation
        // let size_x_zoomed = zoom.zoom * 200.0; // effective width after rotation
        // assert_eq!(zoom.image_off_x, size_x_zoomed);
        // assert_eq!(zoom.image_off_y, 0.0);
    }

    #[test]
    fn test_apply_zoom_constraints() {
        let mut zoom = Zoom::new();
        let image_size = SizeD::new(100.0, 100.0);
        let viewport = test_rect(1, 1); // Very small viewport

        zoom.apply_zoom(ZoomMode::Fill, image_size, viewport);

        // Should be clamped to minimum zoom
        assert_eq!(zoom.zoom, MIN_ZOOM_FACTOR);

        // Test maximum zoom constraint
        let viewport_large = test_rect(10000, 10000); // Very large viewport
        zoom.apply_zoom(ZoomMode::Max, image_size, viewport_large);

        // Should be clamped to maximum zoom
        assert_eq!(zoom.zoom, MAX_ZOOM_FACTOR);
    }

    #[test]
    fn test_apply_zoom_invalid_dimensions() {
        let mut zoom = Zoom::new();
        let viewport = test_rect(400, 300);

        // Test zero dimensions
        zoom.apply_zoom(ZoomMode::Fit, SizeD::new(0.0, 100.0), viewport);
        assert_eq!(zoom.zoom, 1.0); // Should remain unchanged

        zoom.apply_zoom(ZoomMode::Fit, SizeD::new(100.0, 0.0), viewport);
        assert_eq!(zoom.zoom, 1.0); // Should remain unchanged

        // Test negative dimensions
        zoom.apply_zoom(ZoomMode::Fit, SizeD::new(-100.0, 100.0), viewport);
        assert_eq!(zoom.zoom, 1.0); // Should remain unchanged
    }

    #[test]
    fn test_update_zoom() {
        let mut zoom = Zoom::new();
        zoom.set_offset(100.0, 100.0);
        zoom.zoom = 1.0;

        let anchor = (150.0, 150.0); // Point 50 pixels from current offset

        // Zoom in 2x around the anchor point
        zoom.update_zoom(2.0, anchor);

        assert_eq!(zoom.zoom, 2.0);

        // The anchor point should remain visually stationary
        // Point that was 50 pixels from offset should still be at anchor
        let expected_off_x = anchor.0 - (50.0 * 2.0); // 150 - 100 = 50
        let expected_off_y = anchor.1 - (50.0 * 2.0);

        assert!(approx_eq(zoom.offset_x(), expected_off_x, 0.001));
        assert!(approx_eq(zoom.offset_y(), expected_off_y, 0.001));
    }

    #[test]
    fn test_update_zoom_constraints() {
        let mut zoom = Zoom::new();
        let anchor = (100.0, 100.0);

        // Test minimum constraint
        zoom.update_zoom(0.001, anchor);
        assert_eq!(zoom.zoom, MIN_ZOOM_FACTOR);

        // Test maximum constraint
        zoom.update_zoom(100.0, anchor);
        assert_eq!(zoom.zoom, MAX_ZOOM_FACTOR);

        // Test no-change case
        let initial_offset = zoom.offset_x();
        zoom.update_zoom(MAX_ZOOM_FACTOR, anchor); // Same zoom value
        assert_eq!(zoom.offset_x(), initial_offset); // Should not change
    }

    #[test]
    fn test_transformation_matrix() {
        let mut zoom = Zoom::new();
        zoom.zoom = 2.0;
        zoom.set_offset(10.0, 20.0);

        // Test 0-degree rotation
        zoom.set_rotation(0);
        let matrix = zoom.transform_matrix();
        assert_eq!(matrix.xx(), 2.0);
        assert_eq!(matrix.yx(), 0.0);
        assert_eq!(matrix.xy(), 0.0);
        assert_eq!(matrix.yy(), 2.0);
        assert_eq!(matrix.x0(), 10.0);
        assert_eq!(matrix.y0(), 20.0);

        // Test 90-degree rotation
        zoom.set_rotation(90);
        let matrix = zoom.transform_matrix();
        assert_eq!(matrix.xx(), 0.0);
        assert_eq!(matrix.yx(), 2.0);
        assert_eq!(matrix.xy(), -2.0);
        assert_eq!(matrix.yy(), 0.0);
        assert_eq!(matrix.x0(), 10.0);
        assert_eq!(matrix.y0(), 20.0);

        // Test 180-degree rotation
        zoom.set_rotation(180);
        let matrix = zoom.transform_matrix();
        assert_eq!(matrix.xx(), -2.0);
        assert_eq!(matrix.yx(), 0.0);
        assert_eq!(matrix.xy(), 0.0);
        assert_eq!(matrix.yy(), -2.0);
        assert_eq!(matrix.x0(), 10.0);
        assert_eq!(matrix.y0(), 20.0);

        // Test 270-degree rotation
        zoom.set_rotation(270);
        let matrix = zoom.transform_matrix();
        assert_eq!(matrix.xx(), 0.0);
        assert_eq!(matrix.yx(), -2.0);
        assert_eq!(matrix.xy(), 2.0);
        assert_eq!(matrix.yy(), 0.0);
        assert_eq!(matrix.x0(), 10.0);
        assert_eq!(matrix.y0(), 20.0);
    }

    #[test]
    fn test_unscaled_transform_matrix() {
        let mut zoom = Zoom::new();
        zoom.offset_x = 10.0;
        zoom.offset_y = 20.0;

        let size = SizeD::new(100.0, 200.0);

        // Test 0-degree rotation
        zoom.set_rotation(0);
        let matrix = zoom.unscaled_transform_matrix(size);
        assert_eq!(matrix.xx(), 1.0);
        assert_eq!(matrix.yx(), 0.0);
        assert_eq!(matrix.xy(), 0.0);
        assert_eq!(matrix.yy(), 1.0);
        assert_eq!(matrix.x0(), 10.0);
        assert_eq!(matrix.y0(), 20.0);

        // Test 90-degree rotation
        zoom.set_rotation(90);
        let matrix = zoom.unscaled_transform_matrix(size);
        assert_eq!(matrix.xx(), 0.0);
        assert_eq!(matrix.yx(), 1.0);
        assert_eq!(matrix.xy(), -1.0);
        assert_eq!(matrix.yy(), 0.0);
        assert_eq!(matrix.x0(), 210.0);
        assert_eq!(matrix.y0(), 20.0);

        // Test 180-degree rotation
        zoom.set_rotation(180);
        let matrix = zoom.unscaled_transform_matrix(size);
        assert_eq!(matrix.xx(), -1.0);
        assert_eq!(matrix.yx(), 0.0);
        assert_eq!(matrix.xy(), 0.0);
        assert_eq!(matrix.yy(), -1.0);
        assert_eq!(matrix.x0(), 110.0);
        assert_eq!(matrix.y0(), 220.0);

        // Test 270-degree rotation
        zoom.set_rotation(270);
        let matrix = zoom.unscaled_transform_matrix(size);
        assert_eq!(matrix.xx(), 0.0);
        assert_eq!(matrix.yx(), -1.0);
        assert_eq!(matrix.xy(), 1.0);
        assert_eq!(matrix.yy(), 0.0);
        assert_eq!(matrix.x0(), 10.0);
        assert_eq!(matrix.y0(), 120.0);

        // Test negative screen offsets (should be clamped to 0)
        zoom.offset_x = -5.0;
        zoom.offset_y = -10.0;
        zoom.set_rotation(0);
        let matrix = zoom.unscaled_transform_matrix(size);
        assert_eq!(matrix.x0(), 0.0);
        assert_eq!(matrix.y0(), 0.0);

        // Test clamping with rotation
        zoom.set_rotation(90);
        let matrix = zoom.unscaled_transform_matrix(size);
        assert_eq!(matrix.x0(), 200.0);
        assert_eq!(matrix.y0(), 0.0);
    }

    #[test]
    fn test_utility_methods() {
        let mut zoom = Zoom::new();
        zoom.zoom = 2.5;
        zoom.set_rotation(180);

        assert_eq!(zoom.zoom_factor(), 2.5);
        assert_eq!(zoom.rotation_degrees(), 180);
        assert!(zoom.is_rotated());

        zoom.set_rotation(0);
        assert!(!zoom.is_rotated());
    }
}
