//! Transformation support for 2D graphics.
//!
//! This module provides transformation matrices for translation, rotation,
//! and scaling operations on 2D graphics.

/// A 2D affine transformation matrix.
///
/// This matrix represents a 2D affine transformation using a 3x3 matrix
/// in homogeneous coordinates. The matrix is stored in column-major order:
///
/// ```text
/// [ a  c  tx ]
/// [ b  d  ty ]
/// [ 0  0  1  ]
/// ```
///
/// Where:
/// - `a`, `b`, `c`, `d` represent the linear transformation (rotation, scaling, shearing)
/// - `tx`, `ty` represent the translation
///
/// # Examples
///
/// ```
/// use nih_plug_graphics::Transform;
///
/// // Create an identity transform
/// let transform = Transform::identity();
///
/// // Create a translation
/// let translate = Transform::translation(10.0, 20.0);
///
/// // Create a rotation (45 degrees)
/// let rotate = Transform::rotation(std::f32::consts::PI / 4.0);
///
/// // Create a scaling
/// let scale = Transform::scale(2.0, 2.0);
///
/// // Combine transformations
/// let combined = translate.then(&rotate).then(&scale);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Matrix element at position (0, 0) - x scaling and rotation
    pub a: f32,
    /// Matrix element at position (1, 0) - y shearing and rotation
    pub b: f32,
    /// Matrix element at position (0, 1) - x shearing and rotation
    pub c: f32,
    /// Matrix element at position (1, 1) - y scaling and rotation
    pub d: f32,
    /// Translation in x direction
    pub tx: f32,
    /// Translation in y direction
    pub ty: f32,
}

impl Transform {
    /// Creates an identity transformation (no transformation).
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// let transform = Transform::identity();
    /// let (x, y) = transform.apply(10.0, 20.0);
    /// assert_eq!(x, 10.0);
    /// assert_eq!(y, 20.0);
    /// ```
    #[inline]
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Creates a translation transformation.
    ///
    /// # Arguments
    ///
    /// * `tx` - Translation in x direction
    /// * `ty` - Translation in y direction
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// let transform = Transform::translation(10.0, 20.0);
    /// let (x, y) = transform.apply(5.0, 5.0);
    /// assert_eq!(x, 15.0);
    /// assert_eq!(y, 25.0);
    /// ```
    #[inline]
    pub fn translation(tx: f32, ty: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx,
            ty,
        }
    }

    /// Creates a rotation transformation around the origin.
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians (positive = counter-clockwise)
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// // Rotate 90 degrees counter-clockwise
    /// let transform = Transform::rotation(std::f32::consts::PI / 2.0);
    /// let (x, y) = transform.apply(1.0, 0.0);
    /// assert!((x - 0.0).abs() < 0.0001);
    /// assert!((y - 1.0).abs() < 0.0001);
    /// ```
    #[inline]
    pub fn rotation(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Creates a rotation transformation around a specific point.
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians (positive = counter-clockwise)
    /// * `cx` - X coordinate of the rotation center
    /// * `cy` - Y coordinate of the rotation center
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// // Rotate 90 degrees around point (10, 10)
    /// let transform = Transform::rotation_around(std::f32::consts::PI / 2.0, 10.0, 10.0);
    /// ```
    #[inline]
    pub fn rotation_around(angle: f32, cx: f32, cy: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            tx: cx - cx * cos + cy * sin,
            ty: cy - cx * sin - cy * cos,
        }
    }

    /// Creates a uniform scaling transformation.
    ///
    /// # Arguments
    ///
    /// * `scale` - Uniform scale factor for both x and y
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// let transform = Transform::uniform_scale(2.0);
    /// let (x, y) = transform.apply(10.0, 20.0);
    /// assert_eq!(x, 20.0);
    /// assert_eq!(y, 40.0);
    /// ```
    #[inline]
    pub fn uniform_scale(scale: f32) -> Self {
        Self::scale(scale, scale)
    }

    /// Creates a scaling transformation.
    ///
    /// # Arguments
    ///
    /// * `sx` - Scale factor in x direction
    /// * `sy` - Scale factor in y direction
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// let transform = Transform::scale(2.0, 3.0);
    /// let (x, y) = transform.apply(10.0, 20.0);
    /// assert_eq!(x, 20.0);
    /// assert_eq!(y, 60.0);
    /// ```
    #[inline]
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Creates a scaling transformation around a specific point.
    ///
    /// # Arguments
    ///
    /// * `sx` - Scale factor in x direction
    /// * `sy` - Scale factor in y direction
    /// * `cx` - X coordinate of the scaling center
    /// * `cy` - Y coordinate of the scaling center
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// let transform = Transform::scale_around(2.0, 2.0, 10.0, 10.0);
    /// ```
    #[inline]
    pub fn scale_around(sx: f32, sy: f32, cx: f32, cy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            tx: cx - sx * cx,
            ty: cy - sy * cy,
        }
    }

    /// Applies this transformation to a point.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the point
    /// * `y` - Y coordinate of the point
    ///
    /// # Returns
    ///
    /// A tuple `(x', y')` representing the transformed point.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// let transform = Transform::translation(10.0, 20.0);
    /// let (x, y) = transform.apply(5.0, 5.0);
    /// assert_eq!(x, 15.0);
    /// assert_eq!(y, 25.0);
    /// ```
    #[inline]
    pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.tx,
            self.b * x + self.d * y + self.ty,
        )
    }

    /// Applies this transformation to an integer point.
    ///
    /// The result is rounded to the nearest integer.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the point
    /// * `y` - Y coordinate of the point
    ///
    /// # Returns
    ///
    /// A tuple `(x', y')` representing the transformed point as integers.
    #[inline]
    pub fn apply_int(&self, x: i32, y: i32) -> (i32, i32) {
        let (fx, fy) = self.apply(x as f32, y as f32);
        (fx.round() as i32, fy.round() as i32)
    }

    /// Combines this transformation with another transformation.
    ///
    /// Returns a new transformation that applies `self` first, then `other`.
    /// This is equivalent to matrix multiplication: `other * self`.
    ///
    /// # Arguments
    ///
    /// * `other` - The transformation to apply after this one
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// let translate = Transform::translation(10.0, 0.0);
    /// let scale = Transform::scale(2.0, 2.0);
    /// let combined = translate.then(&scale);
    ///
    /// // First translate, then scale
    /// let (x, y) = combined.apply(5.0, 5.0);
    /// assert_eq!(x, 30.0); // (5 + 10) * 2
    /// assert_eq!(y, 10.0); // 5 * 2
    /// ```
    #[inline]
    pub fn then(&self, other: &Transform) -> Transform {
        Transform {
            a: other.a * self.a + other.c * self.b,
            b: other.b * self.a + other.d * self.b,
            c: other.a * self.c + other.c * self.d,
            d: other.b * self.c + other.d * self.d,
            tx: other.a * self.tx + other.c * self.ty + other.tx,
            ty: other.b * self.tx + other.d * self.ty + other.ty,
        }
    }

    /// Translates this transformation.
    ///
    /// Returns a new transformation with the translation applied.
    ///
    /// # Arguments
    ///
    /// * `tx` - Translation in x direction
    /// * `ty` - Translation in y direction
    #[inline]
    pub fn translate(&self, tx: f32, ty: f32) -> Transform {
        self.then(&Transform::translation(tx, ty))
    }

    /// Rotates this transformation.
    ///
    /// Returns a new transformation with the rotation applied.
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians
    #[inline]
    pub fn rotate(&self, angle: f32) -> Transform {
        self.then(&Transform::rotation(angle))
    }

    /// Scales this transformation.
    ///
    /// Returns a new transformation with the scaling applied.
    ///
    /// # Arguments
    ///
    /// * `sx` - Scale factor in x direction
    /// * `sy` - Scale factor in y direction
    #[inline]
    pub fn scale_by(&self, sx: f32, sy: f32) -> Transform {
        self.then(&Transform::scale(sx, sy))
    }

    /// Computes the inverse of this transformation.
    ///
    /// Returns `None` if the transformation is not invertible (determinant is zero).
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// let transform = Transform::translation(10.0, 20.0);
    /// let inverse = transform.inverse().unwrap();
    ///
    /// let (x, y) = transform.apply(5.0, 5.0);
    /// let (x2, y2) = inverse.apply(x, y);
    /// assert!((x2 - 5.0).abs() < 0.0001);
    /// assert!((y2 - 5.0).abs() < 0.0001);
    /// ```
    pub fn inverse(&self) -> Option<Transform> {
        let det = self.a * self.d - self.b * self.c;
        
        if det.abs() < 1e-10 {
            return None;
        }

        let inv_det = 1.0 / det;

        Some(Transform {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            tx: (self.c * self.ty - self.d * self.tx) * inv_det,
            ty: (self.b * self.tx - self.a * self.ty) * inv_det,
        })
    }

    /// Returns the determinant of this transformation matrix.
    ///
    /// The determinant represents the scale factor of the transformation.
    /// A determinant of 0 means the transformation is not invertible.
    #[inline]
    pub fn determinant(&self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    /// Checks if this is an identity transformation.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::Transform;
    ///
    /// let identity = Transform::identity();
    /// assert!(identity.is_identity());
    ///
    /// let translate = Transform::translation(10.0, 0.0);
    /// assert!(!translate.is_identity());
    /// ```
    #[inline]
    pub fn is_identity(&self) -> bool {
        (self.a - 1.0).abs() < 1e-6
            && self.b.abs() < 1e-6
            && self.c.abs() < 1e-6
            && (self.d - 1.0).abs() < 1e-6
            && self.tx.abs() < 1e-6
            && self.ty.abs() < 1e-6
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_identity() {
        let transform = Transform::identity();
        let (x, y) = transform.apply(10.0, 20.0);
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert!(transform.is_identity());
    }

    #[test]
    fn test_translation() {
        let transform = Transform::translation(5.0, 10.0);
        let (x, y) = transform.apply(10.0, 20.0);
        assert_eq!(x, 15.0);
        assert_eq!(y, 30.0);
    }

    #[test]
    fn test_rotation_90_degrees() {
        let transform = Transform::rotation(PI / 2.0);
        let (x, y) = transform.apply(1.0, 0.0);
        assert!((x - 0.0).abs() < 0.0001);
        assert!((y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_scale() {
        let transform = Transform::scale(2.0, 3.0);
        let (x, y) = transform.apply(10.0, 20.0);
        assert_eq!(x, 20.0);
        assert_eq!(y, 60.0);
    }

    #[test]
    fn test_combined_transformations() {
        let translate = Transform::translation(10.0, 0.0);
        let scale = Transform::scale(2.0, 2.0);
        let combined = translate.then(&scale);

        let (x, y) = combined.apply(5.0, 5.0);
        assert_eq!(x, 30.0); // (5 + 10) * 2
        assert_eq!(y, 10.0); // 5 * 2
    }

    #[test]
    fn test_inverse() {
        let transform = Transform::translation(10.0, 20.0)
            .then(&Transform::rotation(PI / 4.0))
            .then(&Transform::scale(2.0, 3.0));

        let inverse = transform.inverse().unwrap();

        let (x, y) = transform.apply(5.0, 7.0);
        let (x2, y2) = inverse.apply(x, y);

        assert!((x2 - 5.0).abs() < 0.0001);
        assert!((y2 - 7.0).abs() < 0.0001);
    }

    #[test]
    fn test_determinant() {
        let identity = Transform::identity();
        assert!((identity.determinant() - 1.0).abs() < 0.0001);

        let scale = Transform::scale(2.0, 3.0);
        assert!((scale.determinant() - 6.0).abs() < 0.0001);
    }
}
