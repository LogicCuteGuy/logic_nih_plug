//! 3×3 and 4×4 matrix types for OpenGL transforms.
//!
//! All matrices are stored in **column-major** order, matching OpenGL's
//! `glUniformMatrix*` expectations.

/// A 3×3 matrix stored in column-major order.
///
/// Useful for normal transforms, 2D rotations, and 2D affine transforms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3D {
    /// The 9 elements in column-major order: `[col0.x, col0.y, col0.z, col1.x, ...]`.
    pub m: [f32; 9],
}

impl Matrix3D {
    /// Create a matrix from an array of 9 floats in column-major order.
    pub fn from_array(m: [f32; 9]) -> Self {
        Self { m }
    }

    /// Identity matrix.
    pub fn identity() -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 0.0, 1.0,
            ],
        }
    }

    /// Zero matrix.
    pub fn zero() -> Self {
        Self { m: [0.0; 9] }
    }

    /// Create a translation matrix (2D, in the xy plane).
    pub fn translation(tx: f32, ty: f32) -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                tx,  ty,  1.0,
            ],
        }
    }

    /// Create a rotation matrix around the Z axis (angle in radians).
    pub fn rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            m: [
                c,   s,   0.0,
                -s,  c,   0.0,
                0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a scale matrix.
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            m: [
                sx,  0.0, 0.0,
                0.0, sy,  0.0,
                0.0, 0.0, 1.0,
            ],
        }
    }

    /// Multiply two 3×3 matrices: `self * rhs`.
    pub fn multiplied(&self, rhs: &Self) -> Self {
        let a = &self.m;
        let b = &rhs.m;
        let mut out = [0.0f32; 9];
        for col in 0..3 {
            for row in 0..3 {
                out[col * 3 + row] = a[0 * 3 + row] * b[col * 3 + 0]
                    + a[1 * 3 + row] * b[col * 3 + 1]
                    + a[2 * 3 + row] * b[col * 3 + 2];
            }
        }
        Self { m: out }
    }

    /// Transpose this matrix.
    pub fn transposed(&self) -> Self {
        let m = &self.m;
        Self {
            m: [
                m[0], m[3], m[6],
                m[1], m[4], m[7],
                m[2], m[5], m[8],
            ],
        }
    }

    /// Returns a pointer suitable for `glUniformMatrix3fv`.
    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }
}

impl Default for Matrix3D {
    fn default() -> Self {
        Self::identity()
    }
}

/// A 4×4 matrix stored in column-major order.
///
/// This is the standard matrix type for 3D transforms in OpenGL (model,
/// view, projection).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4x4 {
    /// The 16 elements in column-major order: `[col0.x, col0.y, col0.z, col0.w, col1.x, ...]`.
    pub m: [f32; 16],
}

impl Matrix4x4 {
    /// Create a matrix from an array of 16 floats in column-major order.
    pub fn from_array(m: [f32; 16]) -> Self {
        Self { m }
    }

    /// Identity matrix.
    pub fn identity() -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Zero matrix.
    pub fn zero() -> Self {
        Self { m: [0.0; 16] }
    }

    /// Create a translation matrix.
    pub fn translation(tx: f32, ty: f32, tz: f32) -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                tx,  ty,  tz,  1.0,
            ],
        }
    }

    /// Create a scale matrix.
    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        Self {
            m: [
                sx,  0.0, 0.0, 0.0,
                0.0, sy,  0.0, 0.0,
                0.0, 0.0, sz,  0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a rotation matrix around the X axis (angle in radians).
    pub fn rotation_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0,
                0.0, c,   s,   0.0,
                0.0, -s,  c,   0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a rotation matrix around the Y axis (angle in radians).
    pub fn rotation_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            m: [
                c,   0.0, -s,  0.0,
                0.0, 1.0, 0.0, 0.0,
                s,   0.0, c,   0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a rotation matrix around the Z axis (angle in radians).
    pub fn rotation_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            m: [
                c,   s,   0.0, 0.0,
                -s,  c,   0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    /// Create a rotation matrix from Euler angles (XYZ order, radians).
    pub fn rotation_euler_xyz(rx: f32, ry: f32, rz: f32) -> Self {
        Self::rotation_x(rx)
            .multiplied(&Self::rotation_y(ry))
            .multiplied(&Self::rotation_z(rz))
    }

    /// Create a perspective projection matrix.
    ///
    /// - `fovy.radians` — vertical field of view
    /// - `aspect` — width / height
    /// - `near` — near clip plane (must be > 0)
    /// - `far` — far clip plane (must be > near)
    pub fn perspective(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fovy_radians / 2.0).tan();
        let nf = 1.0 / (near - far);
        Self {
            m: [
                f / aspect, 0.0, 0.0,                    0.0,
                0.0,        f,   0.0,                    0.0,
                0.0,        0.0, (far + near) * nf,     -1.0,
                0.0,        0.0, 2.0 * far * near * nf, 0.0,
            ],
        }
    }

    /// Create an orthographic projection matrix.
    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let w = right - left;
        let h = top - bottom;
        let d = far - near;
        Self {
            m: [
                2.0 / w,  0.0,      0.0,       0.0,
                0.0,      2.0 / h,  0.0,       0.0,
                0.0,      0.0,     -2.0 / d,   0.0,
                -(right + left) / w,
                -(top + bottom) / h,
                -(far + near) / d,
                1.0,
            ],
        }
    }

    /// Create a look-at view matrix.
    pub fn look_at(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> Self {
        let f = normalize(sub(center, eye));
        let s = normalize(cross(f, up));
        let u = cross(s, f);

        Self {
            m: [
                s[0],           u[0],          -f[0],          0.0,
                s[1],           u[1],          -f[1],          0.0,
                s[2],           u[2],          -f[2],          0.0,
                -dot(s, eye),  -dot(u, eye),  dot(f, eye),   1.0,
            ],
        }
    }

    /// Multiply two 4×4 matrices: `self * rhs`.
    pub fn multiplied(&self, rhs: &Self) -> Self {
        let a = &self.m;
        let b = &rhs.m;
        let mut out = [0.0f32; 16];
        for col in 0..4 {
            for row in 0..4 {
                out[col * 4 + row] = a[0 * 4 + row] * b[col * 4 + 0]
                    + a[1 * 4 + row] * b[col * 4 + 1]
                    + a[2 * 4 + row] * b[col * 4 + 2]
                    + a[3 * 4 + row] * b[col * 4 + 3];
            }
        }
        Self { m: out }
    }

    /// Transpose this matrix.
    pub fn transposed(&self) -> Self {
        let m = &self.m;
        Self {
            m: [
                m[0], m[4], m[8],  m[12],
                m[1], m[5], m[9],  m[13],
                m[2], m[6], m[10], m[14],
                m[3], m[7], m[11], m[15],
            ],
        }
    }

    /// Transform a 3D point (w=1).
    pub fn transform_point(&self, p: [f32; 3]) -> [f32; 3] {
        let m = &self.m;
        let w = m[3] * p[0] + m[7] * p[1] + m[11] * p[2] + m[15];
        [
            (m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12]) / w,
            (m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13]) / w,
            (m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14]) / w,
        ]
    }

    /// Transform a 3D direction (w=0, ignores translation).
    pub fn transform_direction(&self, d: [f32; 3]) -> [f32; 3] {
        let m = &self.m;
        [
            m[0] * d[0] + m[4] * d[1] + m[8] * d[2],
            m[1] * d[0] + m[5] * d[1] + m[9] * d[2],
            m[2] * d[0] + m[6] * d[1] + m[10] * d[2],
        ]
    }

    /// Returns a pointer suitable for `glUniformMatrix4fv`.
    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }
}

impl Default for Matrix4x4 {
    fn default() -> Self {
        Self::identity()
    }
}

// --- vec3 helpers (private) ---

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = dot(v, v).sqrt();
    if len < 1e-8 {
        [0.0; 3]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    fn mat4_approx_eq(a: &Matrix4x4, b: &Matrix4x4) -> bool {
        a.m.iter().zip(b.m.iter()).all(|(x, y)| approx_eq(*x, *y))
    }

    fn mat3_approx_eq(a: &Matrix3D, b: &Matrix3D) -> bool {
        a.m.iter().zip(b.m.iter()).all(|(x, y)| approx_eq(*x, *y))
    }

    // --- Matrix3D tests ---

    #[test]
    fn mat3_identity() {
        let i = Matrix3D::identity();
        assert_eq!(i.m, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn mat3_zero() {
        assert_eq!(Matrix3D::zero().m, [0.0; 9]);
    }

    #[test]
    fn mat3_translation() {
        let t = Matrix3D::translation(3.0, 4.0);
        assert!(approx_eq(t.m[6], 3.0));
        assert!(approx_eq(t.m[7], 4.0));
    }

    #[test]
    fn mat3_rotation_z() {
        let r = Matrix3D::rotation_z(std::f32::consts::FRAC_PI_2); // 90°
        // (1,0) should rotate to (0,1)
        let x = r.m[0]; // cos(90) ≈ 0
        let y = r.m[1]; // sin(90) ≈ 1
        assert!(approx_eq(x, 0.0));
        assert!(approx_eq(y, 1.0));
    }

    #[test]
    fn mat3_scale() {
        let s = Matrix3D::scale(2.0, 3.0);
        assert!(approx_eq(s.m[0], 2.0));
        assert!(approx_eq(s.m[4], 3.0));
    }

    #[test]
    fn mat3_multiplied_identity() {
        let a = Matrix3D::translation(1.0, 2.0);
        let i = Matrix3D::identity();
        let r = a.multiplied(&i);
        assert!(mat3_approx_eq(&r, &a));
    }

    #[test]
    fn mat3_transpose() {
        let m = Matrix3D::from_array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let t = m.transposed();
        assert_eq!(t.m, [1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0]);
    }

    #[test]
    fn mat3_transpose_twice() {
        let m = Matrix3D::from_array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        let tt = m.transposed().transposed();
        assert!(mat3_approx_eq(&m, &tt));
    }

    #[test]
    fn mat3_as_ptr() {
        let m = Matrix3D::identity();
        let ptr = m.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn mat3_default_is_identity() {
        let m = Matrix3D::default();
        assert_eq!(m, Matrix3D::identity());
    }

    // --- Matrix4x4 tests ---

    #[test]
    fn mat4_identity() {
        let i = Matrix4x4::identity();
        assert_eq!(i.m, [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ]);
    }

    #[test]
    fn mat4_zero() {
        assert_eq!(Matrix4x4::zero().m, [0.0; 16]);
    }

    #[test]
    fn mat4_translation() {
        let t = Matrix4x4::translation(1.0, 2.0, 3.0);
        // Column 3 (indices 12..15) stores translation
        assert!(approx_eq(t.m[12], 1.0));
        assert!(approx_eq(t.m[13], 2.0));
        assert!(approx_eq(t.m[14], 3.0));
    }

    #[test]
    fn mat4_scale() {
        let s = Matrix4x4::scale(2.0, 3.0, 4.0);
        assert!(approx_eq(s.m[0], 2.0));
        assert!(approx_eq(s.m[5], 3.0));
        assert!(approx_eq(s.m[10], 4.0));
    }

    #[test]
    fn mat4_rotation_x_90() {
        let r = Matrix4x4::rotation_x(std::f32::consts::FRAC_PI_2);
        // Y-axis (0,1,0) should map to (0,0,1)
        let p = r.transform_direction([0.0, 1.0, 0.0]);
        assert!(approx_eq(p[0], 0.0));
        assert!(approx_eq(p[1], 0.0));
        assert!(approx_eq(p[2], 1.0));
    }

    #[test]
    fn mat4_rotation_y_90() {
        let r = Matrix4x4::rotation_y(std::f32::consts::FRAC_PI_2);
        // X-axis (1,0,0) should map to (0,0,-1)
        let p = r.transform_direction([1.0, 0.0, 0.0]);
        assert!(approx_eq(p[0], 0.0));
        assert!(approx_eq(p[1], 0.0));
        assert!(approx_eq(p[2], -1.0));
    }

    #[test]
    fn mat4_rotation_z_90() {
        let r = Matrix4x4::rotation_z(std::f32::consts::FRAC_PI_2);
        // X-axis (1,0,0) should map to (0,1,0)
        let p = r.transform_direction([1.0, 0.0, 0.0]);
        assert!(approx_eq(p[0], 0.0));
        assert!(approx_eq(p[1], 1.0));
        assert!(approx_eq(p[2], 0.0));
    }

    #[test]
    fn mat4_rotation_euler_xyz() {
        // Rotation by zero should be identity
        let r = Matrix4x4::rotation_euler_xyz(0.0, 0.0, 0.0);
        assert!(mat4_approx_eq(&r, &Matrix4x4::identity()));
    }

    #[test]
    fn mat4_perspective_basic() {
        let p = Matrix4x4::perspective(
            std::f32::consts::FRAC_PI_4, // 45°
            1.0,                          // aspect
            0.1,                          // near
            100.0,                        // far
        );
        // Should not be identity
        assert!(!mat4_approx_eq(&p, &Matrix4x4::identity()));
        // The [3][2] element should be -1.0 (perspective divide)
        assert!(approx_eq(p.m[11], -1.0));
    }

    #[test]
    fn mat4_ortho_basic() {
        let o = Matrix4x4::ortho(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0);
        // Should map [-1,1] to [-1,1]
        let p = o.transform_point([0.0, 0.0, -1.0]);
        assert!(approx_eq(p[0], 0.0));
        assert!(approx_eq(p[1], 0.0));
    }

    #[test]
    fn mat4_look_at() {
        let eye = [0.0, 0.0, 5.0];
        let center = [0.0, 0.0, 0.0];
        let up = [0.0, 1.0, 0.0];
        let v = Matrix4x4::look_at(eye, center, up);
        // Looking down -Z from (0,0,5), the origin should map to (0,0,-5)
        let p = v.transform_point([0.0, 0.0, 0.0]);
        assert!(approx_eq(p[0], 0.0));
        assert!(approx_eq(p[1], 0.0));
        assert!(approx_eq(p[2], -5.0));
    }

    #[test]
    fn mat4_multiplied_identity() {
        let a = Matrix4x4::translation(1.0, 2.0, 3.0);
        let i = Matrix4x4::identity();
        let r = a.multiplied(&i);
        assert!(mat4_approx_eq(&r, &a));
    }

    #[test]
    fn mat4_transpose() {
        let m = Matrix4x4::from_array([
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        ]);
        let t = m.transposed();
        assert_eq!(t.m, [
            1.0, 5.0, 9.0,  13.0,
            2.0, 6.0, 10.0, 14.0,
            3.0, 7.0, 11.0, 15.0,
            4.0, 8.0, 12.0, 16.0,
        ]);
    }

    #[test]
    fn mat4_transpose_twice() {
        let m = Matrix4x4::from_array([
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        ]);
        let tt = m.transposed().transposed();
        assert!(mat4_approx_eq(&m, &tt));
    }

    #[test]
    fn mat4_transform_point_with_translation() {
        let t = Matrix4x4::translation(10.0, 20.0, 30.0);
        let p = t.transform_point([1.0, 2.0, 3.0]);
        assert!(approx_eq(p[0], 11.0));
        assert!(approx_eq(p[1], 22.0));
        assert!(approx_eq(p[2], 33.0));
    }

    #[test]
    fn mat4_transform_direction_ignores_translation() {
        let t = Matrix4x4::translation(100.0, 200.0, 300.0);
        let d = t.transform_direction([1.0, 0.0, 0.0]);
        assert!(approx_eq(d[0], 1.0));
        assert!(approx_eq(d[1], 0.0));
        assert!(approx_eq(d[2], 0.0));
    }

    #[test]
    fn mat4_as_ptr() {
        let m = Matrix4x4::identity();
        let ptr = m.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn mat4_default_is_identity() {
        let m = Matrix4x4::default();
        assert_eq!(m, Matrix4x4::identity());
    }

    // --- vec3 helper tests ---

    #[test]
    fn vec3_sub() {
        let r = sub([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        assert_eq!(r, [-3.0, -3.0, -3.0]);
    }

    #[test]
    fn vec3_dot() {
        assert!(approx_eq(dot([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), 0.0));
        assert!(approx_eq(dot([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]), 14.0));
    }

    #[test]
    fn vec3_cross() {
        let c = cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(approx_eq(c[0], 0.0));
        assert!(approx_eq(c[1], 0.0));
        assert!(approx_eq(c[2], 1.0));
    }

    #[test]
    fn vec3_normalize() {
        let n = normalize([0.0, 3.0, 4.0]);
        assert!(approx_eq(n[0], 0.0));
        assert!(approx_eq(n[1], 0.6));
        assert!(approx_eq(n[2], 0.8));
    }

    #[test]
    fn vec3_normalize_zero() {
        let n = normalize([0.0, 0.0, 0.0]);
        assert_eq!(n, [0.0, 0.0, 0.0]);
    }
}
