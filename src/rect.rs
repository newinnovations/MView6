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

#![allow(dead_code)]

use std::fmt::Debug;

/// A rectangle defined by two corner points (x0, y0) and (x1, y1).
/// The rectangle is valid when x0 <= x1 and y0 <= y1.
/// Empty rectangles have x0 >= x1 or y0 >= y1.
///
/// Generic over numeric types T that support basic arithmetic and comparison operations.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Rect<T> {
    pub x0: T,
    pub y0: T,
    pub x1: T,
    pub y1: T,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Size<T> {
    width: T,
    height: T,
}

impl<T> Size<T>
where
    T: Copy,
{
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> T {
        self.width
    }

    pub fn height(&self) -> T {
        self.height
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct VectorPoint<T> {
    x: T,
    y: T,
}

impl<T> VectorPoint<T>
where
    T: Default
        + Copy
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>
        + std::ops::Div<Output = T>,
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }

    /// Returns a new vector translated by the given offsets.
    pub fn translate(&self, offset: VectorPoint<T>) -> Self {
        Self::new(self.x + offset.x(), self.y + offset.y())
    }

    /// Returns a new vector scaled by the given scale.
    pub fn scale(&self, scale: T) -> Self {
        Self::new(self.x * scale, self.y * scale)
    }

    /// Returns a new vector unscaled by the given scale.
    pub fn unscale(&self, scale: T) -> Self {
        Self::new(self.x / scale, self.y / scale)
    }

    /// Returns the vector rotated by 180 degrees
    pub fn neg(&self) -> Self {
        Self::new(T::default() - self.x, T::default() - self.y)
    }

    pub fn rotate(&self, rotation: i32) -> Self {
        match rotation {
            -90 | 270 => Self::new(self.y, T::default() - self.x),
            -180 | 180 => Self::new(T::default() - self.x, T::default() - self.y),
            -270 | 90 => Self::new(T::default() - self.y, self.x),
            _ => Self::new(self.x, self.y),
        }
    }
}

impl<T> std::ops::Add for VectorPoint<T>
where
    T: Copy + std::ops::Add<Output = T>,
{
    type Output = VectorPoint<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> std::ops::AddAssign for VectorPoint<T>
where
    T: Copy + std::ops::Add<Output = T>,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

impl<T> std::ops::Sub for VectorPoint<T>
where
    T: Copy + std::ops::Sub<Output = T>,
{
    type Output = VectorPoint<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl VectorPoint<f32> {
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl VectorPoint<f64> {
    pub fn distance(&self, other: &Self) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl<T> Rect<T>
where
    T: Copy
        + PartialOrd
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>
        + std::ops::Div<Output = T>
        + Debug
        + Default,
{
    /// Creates a new rectangle with the given coordinates.
    /// No validation is performed - the rectangle may be invalid or empty.
    pub const fn new(x0: T, y0: T, x1: T, y1: T) -> Self {
        Self { x0, y0, x1, y1 }
    }

    pub fn new_from_size(size: Size<T>) -> Self {
        Self::new(T::default(), T::default(), size.width, size.height)
    }

    /// Returns true if the rectangle is empty (has zero or negative area).
    /// An empty rectangle has x0 >= x1 or y0 >= y1.
    pub fn is_empty(&self) -> bool {
        self.x0 >= self.x1 || self.y0 >= self.y1
    }

    /// Returns true if the rectangle is valid (x0 <= x1 and y0 <= y1).
    /// A valid rectangle may still be empty if x0 == x1 or y0 == y1.
    pub fn is_valid(&self) -> bool {
        self.x0 <= self.x1 && self.y0 <= self.y1
    }

    /// Returns true if the point (x, y) is contained within the rectangle.
    /// Uses half-open intervals: [x0, x1) and [y0, y1).
    /// Returns false for empty rectangles.
    pub fn contains(&self, p: VectorPoint<T>) -> bool {
        if self.is_empty() {
            false
        } else {
            p.x >= self.x0 && p.x < self.x1 && p.y >= self.y0 && p.y < self.y1
        }
    }

    /// Returns the width of the rectangle.
    /// Returns zero for empty rectangles.
    pub fn width(&self) -> T {
        if self.is_empty() {
            T::default()
        } else {
            self.x1 - self.x0
        }
    }

    /// Returns the height of the rectangle.
    /// Returns zero for empty rectangles.
    pub fn height(&self) -> T {
        if self.is_empty() {
            T::default()
        } else {
            self.y1 - self.y0
        }
    }

    /// Returns the size of the rectangle.
    /// Returns zero for empty rectangles.
    pub fn size(&self) -> Size<T> {
        if self.is_empty() {
            Size::default()
        } else {
            Size {
                width: self.x1 - self.x0,
                height: self.y1 - self.y0,
            }
        }
    }

    /// Returns the union of this rectangle with another rectangle.
    /// The union is the smallest rectangle that contains both rectangles.
    /// If both rectangles are empty, returns an empty rectangle.
    /// If one rectangle is empty, returns the other rectangle.
    pub fn union(&self, other: &Self) -> Self {
        if self.is_empty() && other.is_empty() {
            // Both empty - return an empty rectangle
            Self::new(T::default(), T::default(), T::default(), T::default())
        } else if self.is_empty() {
            // Self is empty, return other
            *other
        } else if other.is_empty() {
            // Other is empty, return self
            *self
        } else {
            // Both non-empty, compute union
            Self::new(
                if self.x0 <= other.x0 {
                    self.x0
                } else {
                    other.x0
                },
                if self.y0 <= other.y0 {
                    self.y0
                } else {
                    other.y0
                },
                if self.x1 >= other.x1 {
                    self.x1
                } else {
                    other.x1
                },
                if self.y1 >= other.y1 {
                    self.y1
                } else {
                    other.y1
                },
            )
        }
    }

    /// Returns the intersection of this rectangle with another rectangle.
    /// The intersection is the largest rectangle contained in both rectangles.
    /// Returns an empty rectangle if there is no intersection.
    pub fn intersect(&self, other: &Self) -> Self {
        let x0 = if self.x0 >= other.x0 {
            self.x0
        } else {
            other.x0
        };
        let y0 = if self.y0 >= other.y0 {
            self.y0
        } else {
            other.y0
        };
        let x1 = if self.x1 <= other.x1 {
            self.x1
        } else {
            other.x1
        };
        let y1 = if self.y1 <= other.y1 {
            self.y1
        } else {
            other.y1
        };

        Self::new(x0, y0, x1, y1)
    }

    /// Returns a new rectangle scaled by the given scale.
    pub fn scale(&self, scale: T) -> Self {
        Self::new(
            self.x0 * scale,
            self.y0 * scale,
            self.x1 * scale,
            self.y1 * scale,
        )
    }

    /// Returns a new rectangle translated by the given offsets.
    /// Both corner points are moved by (xoff, yoff).
    pub fn translate(&self, offset: VectorPoint<T>) -> Self {
        Self::new(
            self.x0 + offset.x(),
            self.y0 + offset.y(),
            self.x1 + offset.x(),
            self.y1 + offset.y(),
        )
    }

    pub fn rotate(&self, rotation: i32) -> Self {
        if self.is_valid() {
            let tl = VectorPoint::new(self.x0, self.y0).rotate(rotation);
            let br = VectorPoint::new(self.x1, self.y1).rotate(rotation);
            Self::new(
                if tl.x < br.x { tl.x } else { br.x },
                if tl.y < br.y { tl.y } else { br.y },
                if tl.x > br.x { tl.x } else { br.x },
                if tl.y > br.y { tl.y } else { br.y },
            )
        } else {
            Self::default()
        }
    }

    pub fn point0(&self) -> VectorPoint<T> {
        VectorPoint {
            x: self.x0,
            y: self.y0,
        }
    }

    pub fn point1(&self) -> VectorPoint<T> {
        VectorPoint {
            x: self.x1,
            y: self.y1,
        }
    }
}

// Floating-point specific implementations
impl Rect<f32> {
    /// Rounds the rectangle coordinates to the nearest integers.
    /// Returns (x0, y0, x1, y1) as i32 values.
    /// Uses floor for top-left corner and ceil for bottom-right to ensure coverage.
    pub fn round(&self) -> (i32, i32, i32, i32) {
        (
            self.x0.floor() as i32,
            self.y0.floor() as i32,
            self.x1.ceil() as i32,
            self.y1.ceil() as i32,
        )
    }

    /// Converts this f32 rectangle to an i32 rectangle using rounding
    pub fn to_i32_rect(self) -> Rect<i32> {
        let (x0, y0, x1, y1) = self.round();
        Rect::new(x0, y0, x1, y1)
    }

    /// Creates an f32 rectangle from an i32 rectangle
    pub fn from_i32_rect(rect: &Rect<i32>) -> Self {
        Rect::new(
            rect.x0 as f32,
            rect.y0 as f32,
            rect.x1 as f32,
            rect.y1 as f32,
        )
    }
}

impl Rect<f64> {
    /// Rounds the rectangle coordinates to the nearest integers.
    /// Returns (x0, y0, x1, y1) as i32 values.
    /// Uses floor for top-left corner and ceil for bottom-right to ensure coverage.
    pub fn round(&self) -> (i32, i32, i32, i32) {
        (
            self.x0.floor() as i32,
            self.y0.floor() as i32,
            self.x1.ceil() as i32,
            self.y1.ceil() as i32,
        )
    }

    /// Converts this f64 rectangle to an i32 rectangle using rounding
    pub fn to_i32_rect(self) -> Rect<i32> {
        let (x0, y0, x1, y1) = self.round();
        Rect::new(x0, y0, x1, y1)
    }

    /// Creates an f64 rectangle from an i32 rectangle
    pub fn from_i32_rect(rect: &Rect<i32>) -> Self {
        Rect::new(
            rect.x0 as f64,
            rect.y0 as f64,
            rect.x1 as f64,
            rect.y1 as f64,
        )
    }

    /// Converts f64 rectangle to f32 rectangle (with potential precision loss)
    pub fn to_f32_rect(self) -> Rect<f32> {
        Rect::new(
            self.x0 as f32,
            self.y0 as f32,
            self.x1 as f32,
            self.y1 as f32,
        )
    }

    pub fn center(self) -> (f64, f64) {
        ((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
    }
}

// Integer specific implementations
impl Rect<i32> {
    /// For integer rectangles, round returns the coordinates as-is
    pub fn round(&self) -> (i32, i32, i32, i32) {
        (self.x0, self.y0, self.x1, self.y1)
    }

    /// Converts this i32 rectangle to an f32 rectangle
    pub fn to_f32_rect(self) -> Rect<f32> {
        Rect::new(
            self.x0 as f32,
            self.y0 as f32,
            self.x1 as f32,
            self.y1 as f32,
        )
    }

    /// Converts this i32 rectangle to an f64 rectangle
    pub fn to_f64_rect(self) -> Rect<f64> {
        Rect::new(
            self.x0 as f64,
            self.y0 as f64,
            self.x1 as f64,
            self.y1 as f64,
        )
    }
}

// Type aliases for convenience
pub type RectI = Rect<i32>;
pub type RectF = Rect<f32>;
pub type RectD = Rect<f64>;
pub type SizeI = Size<i32>;
pub type SizeF = Size<f32>;
pub type SizeD = Size<f64>;
pub type PointI = VectorPoint<i32>;
pub type PointF = VectorPoint<f32>;
pub type PointD = VectorPoint<f64>;
pub type VectorI = VectorPoint<i32>;
pub type VectorF = VectorPoint<f32>;
pub type VectorD = VectorPoint<f64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_new() {
        let rect_i32 = Rect::<i32>::new(1, 2, 5, 6);
        let rect_f32 = Rect::<f32>::new(1.0, 2.0, 5.0, 6.0);
        let rect_f64 = Rect::<f64>::new(1.0, 2.0, 5.0, 6.0);

        assert_eq!(rect_i32.x0, 1);
        assert_eq!(rect_f32.x0, 1.0);
        assert_eq!(rect_f64.x0, 1.0);
    }

    #[test]
    fn test_i32_rect() {
        let rect = RectI::new(0, 0, 10, 20);

        assert!(!rect.is_empty());
        assert!(rect.is_valid());
        assert!(rect.contains(PointI::new(5, 5)));
        assert!(!rect.contains(PointI::new(10, 5))); // Exclusive upper bound
        assert_eq!(rect.width(), 10);
        assert_eq!(rect.height(), 20);
        assert_eq!(rect.size(), SizeI::new(10, 20));

        let translated = rect.translate(VectorI::new(5, 5));
        assert_eq!(translated.x0, 5);
        assert_eq!(translated.y0, 5);
        assert_eq!(translated.x1, 15);
        assert_eq!(translated.y1, 25);
    }

    #[test]
    fn test_f32_rect() {
        let rect = RectF::new(0.0, 0.5, 10.5, 11.5);

        assert!(!rect.is_empty());
        assert!(rect.is_valid());
        assert!(rect.contains(PointF::new(5.25, 5.25)));
        assert!(!rect.contains(PointF::new(10.5, 5.0))); // Exclusive upper bound
        assert_eq!(rect.width(), 10.5);
        assert_eq!(rect.height(), 11.0);
        assert_eq!(rect.size(), SizeF::new(10.5, 11.0));

        let (x0, y0, x1, y1) = rect.round();
        assert_eq!((x0, y0, x1, y1), (0, 0, 11, 12));
    }

    #[test]
    fn test_f64_rect() {
        let rect = RectD::new(0.0, 0.0, 10.7, 10.3);

        assert!(!rect.is_empty());
        assert!(rect.is_valid());
        assert!(rect.contains(PointD::new(5.35, 5.15)));
        assert_eq!(rect.width(), 10.7);
        assert_eq!(rect.height(), 10.3);
        assert_eq!(rect.size(), SizeD::new(10.7, 10.3));

        let (x0, y0, x1, y1) = rect.round();
        assert_eq!((x0, y0, x1, y1), (0, 0, 11, 11));
    }

    #[test]
    fn test_union_generic() {
        let rect1 = RectI::new(0, 0, 5, 5);
        let rect2 = RectI::new(3, 3, 8, 8);
        let union = rect1.union(&rect2);

        assert_eq!(union, RectI::new(0, 0, 8, 8));

        // Test with f32
        let rect1_f = RectF::new(0.0, 0.0, 5.5, 5.5);
        let rect2_f = RectF::new(3.0, 3.0, 8.2, 8.2);
        let union_f = rect1_f.union(&rect2_f);

        assert_eq!(union_f, RectF::new(0.0, 0.0, 8.2, 8.2));
    }

    #[test]
    fn test_intersect_generic() {
        let rect1 = RectI::new(0, 0, 10, 10);
        let rect2 = RectI::new(5, 5, 15, 15);
        let intersection = rect1.intersect(&rect2);

        assert_eq!(intersection, RectI::new(5, 5, 10, 10));

        // Non-intersecting rectangles
        let rect3 = RectI::new(20, 20, 30, 30);
        let no_intersection = rect1.intersect(&rect3);
        assert!(no_intersection.is_empty());
    }

    #[test]
    fn test_empty_rectangles_generic() {
        let empty_i32 = RectI::new(5, 5, 5, 5);
        let empty_f32 = RectF::new(5.0, 5.0, 5.0, 5.0);

        assert!(empty_i32.is_empty());
        assert!(empty_f32.is_empty());
        assert_eq!(empty_i32.width(), 0);
        assert_eq!(empty_f32.width(), 0.0);
        assert!(!empty_i32.contains(PointI::new(5, 5)));
        assert!(!empty_f32.contains(PointF::new(5.0, 5.0)));
    }

    #[test]
    fn test_type_conversions() {
        let rect_i32 = RectI::new(1, 2, 5, 6);
        let rect_f32 = rect_i32.to_f32_rect();
        let rect_f64 = rect_i32.to_f64_rect();

        assert_eq!(rect_f32, RectF::new(1.0, 2.0, 5.0, 6.0));
        assert_eq!(rect_f64, RectD::new(1.0, 2.0, 5.0, 6.0));

        // Test f64 to f32 conversion
        let rect_f64_precise = RectD::new(1.1, 2.2, 5.5, 6.6);
        let rect_f32_converted = rect_f64_precise.to_f32_rect();
        assert_eq!(rect_f32_converted, RectF::new(1.1, 2.2, 5.5, 6.6));

        // Test rounding conversion
        let rect_f32_fractional = RectF::new(1.2, 2.7, 5.1, 6.9);
        let rect_i32_rounded = rect_f32_fractional.to_i32_rect();
        assert_eq!(rect_i32_rounded, RectI::new(1, 2, 6, 7));
    }

    #[test]
    fn test_type_aliases() {
        let rect_i = RectI::new(0, 0, 10, 10);
        let rect_f = RectF::new(0.0, 0.0, 10.0, 10.0);
        let rect_d = RectD::new(0.0, 0.0, 10.0, 10.0);

        assert_eq!(rect_i.width(), 10);
        assert_eq!(rect_f.width(), 10.0);
        assert_eq!(rect_d.width(), 10.0);
    }

    #[test]
    fn test_precision_edge_cases() {
        // Test with very small f32 values
        let tiny_f32 = RectF::new(0.0, 0.0, f32::EPSILON, f32::EPSILON);
        assert!(!tiny_f32.is_empty());
        assert!(tiny_f32.width() > 0.0);

        // Test with very small f64 values
        let tiny_f64 = RectD::new(0.0, 0.0, f64::EPSILON, f64::EPSILON);
        assert!(!tiny_f64.is_empty());
        assert!(tiny_f64.width() > 0.0);

        // Test negative coordinates
        let negative = RectI::new(-10, -10, -5, -5);
        assert!(!negative.is_empty());
        assert_eq!(negative.width(), 5);
        assert_eq!(negative.height(), 5);
        assert!(negative.contains(PointI::new(-7, -7)));
    }

    #[test]
    fn test_floating_point_rounding() {
        // Test f32 rounding
        let rect_f32 = RectF::new(1.2, 2.7, 5.1, 6.9);
        let (x0, y0, x1, y1) = rect_f32.round();
        assert_eq!((x0, y0, x1, y1), (1, 2, 6, 7));

        // Test f64 rounding
        let rect_f64 = RectD::new(1.2, 2.7, 5.1, 6.9);
        let (x0, y0, x1, y1) = rect_f64.round();
        assert_eq!((x0, y0, x1, y1), (1, 2, 6, 7));

        // Test exact values
        let exact_f32 = RectF::new(2.0, 3.0, 4.0, 5.0);
        let (x0, y0, x1, y1) = exact_f32.round();
        assert_eq!((x0, y0, x1, y1), (2, 3, 4, 5));
    }
}
