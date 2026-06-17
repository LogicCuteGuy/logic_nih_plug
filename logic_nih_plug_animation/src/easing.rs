//! Easing function implementations.
//!
//! Easing functions control the rate of change of a value over time,
//! creating smooth and natural-looking animations.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_animation::easing::{EasingFunction, ease_in_out_cubic};
//!
//! let t = 0.5; // Halfway through animation
//! let eased = ease_in_out_cubic(t);
//! ```

/// An easing function that maps a normalized time value [0.0, 1.0] to an eased value.
///
/// The input `t` represents the progress through the animation, where:
/// - 0.0 = start of animation
/// - 1.0 = end of animation
///
/// The output is the eased value, typically also in the range [0.0, 1.0],
/// though some easing functions may overshoot this range.
pub type EasingFunction = fn(f32) -> f32;

/// Linear easing - no acceleration or deceleration.
///
/// Returns the input value unchanged.
#[inline]
pub fn linear(t: f32) -> f32 {
    t
}

/// Ease in using a quadratic curve.
///
/// Starts slowly and accelerates.
#[inline]
pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Ease out using a quadratic curve.
///
/// Starts quickly and decelerates.
#[inline]
pub fn ease_out_quad(t: f32) -> f32 {
    t * (2.0 - t)
}

/// Ease in and out using a quadratic curve.
///
/// Accelerates at the start and decelerates at the end.
#[inline]
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

/// Ease in using a cubic curve.
///
/// Starts slowly and accelerates more than quadratic.
#[inline]
pub fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

/// Ease out using a cubic curve.
///
/// Starts quickly and decelerates more than quadratic.
#[inline]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t1 = t - 1.0;
    t1 * t1 * t1 + 1.0
}

/// Ease in and out using a cubic curve.
///
/// Accelerates at the start and decelerates at the end with a smooth curve.
#[inline]
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t1 = 2.0 * t - 2.0;
        1.0 + t1 * t1 * t1 / 2.0
    }
}

/// Ease in using a quartic curve.
///
/// Starts very slowly and accelerates strongly.
#[inline]
pub fn ease_in_quart(t: f32) -> f32 {
    t * t * t * t
}

/// Ease out using a quartic curve.
///
/// Starts very quickly and decelerates strongly.
#[inline]
pub fn ease_out_quart(t: f32) -> f32 {
    let t1 = t - 1.0;
    1.0 - t1 * t1 * t1 * t1
}

/// Ease in and out using a quartic curve.
///
/// Strong acceleration at the start and deceleration at the end.
#[inline]
pub fn ease_in_out_quart(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let t1 = t - 1.0;
        1.0 - 8.0 * t1 * t1 * t1 * t1
    }
}

/// Ease in using a quintic curve.
///
/// Starts extremely slowly and accelerates very strongly.
#[inline]
pub fn ease_in_quint(t: f32) -> f32 {
    t * t * t * t * t
}

/// Ease out using a quintic curve.
///
/// Starts extremely quickly and decelerates very strongly.
#[inline]
pub fn ease_out_quint(t: f32) -> f32 {
    let t1 = t - 1.0;
    1.0 + t1 * t1 * t1 * t1 * t1
}

/// Ease in and out using a quintic curve.
///
/// Very strong acceleration at the start and deceleration at the end.
#[inline]
pub fn ease_in_out_quint(t: f32) -> f32 {
    if t < 0.5 {
        16.0 * t * t * t * t * t
    } else {
        let t1 = 2.0 * t - 2.0;
        1.0 + t1 * t1 * t1 * t1 * t1 / 2.0
    }
}

/// Ease in using a sine curve.
///
/// Smooth acceleration using a sine wave.
#[inline]
pub fn ease_in_sine(t: f32) -> f32 {
    1.0 - (t * std::f32::consts::FRAC_PI_2).cos()
}

/// Ease out using a sine curve.
///
/// Smooth deceleration using a sine wave.
#[inline]
pub fn ease_out_sine(t: f32) -> f32 {
    (t * std::f32::consts::FRAC_PI_2).sin()
}

/// Ease in and out using a sine curve.
///
/// Smooth acceleration and deceleration using a sine wave.
#[inline]
pub fn ease_in_out_sine(t: f32) -> f32 {
    -(((t * std::f32::consts::PI).cos() - 1.0) / 2.0)
}

/// Ease in using an exponential curve.
///
/// Very slow start with exponential acceleration.
#[inline]
pub fn ease_in_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        2.0f32.powf(10.0 * t - 10.0)
    }
}

/// Ease out using an exponential curve.
///
/// Very fast start with exponential deceleration.
#[inline]
pub fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0f32.powf(-10.0 * t)
    }
}

/// Ease in and out using an exponential curve.
///
/// Exponential acceleration and deceleration.
#[inline]
pub fn ease_in_out_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        2.0f32.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0f32.powf(-20.0 * t + 10.0)) / 2.0
    }
}

/// Ease in using a circular curve.
///
/// Acceleration following a circular arc.
#[inline]
pub fn ease_in_circ(t: f32) -> f32 {
    1.0 - (1.0 - t * t).sqrt()
}

/// Ease out using a circular curve.
///
/// Deceleration following a circular arc.
#[inline]
pub fn ease_out_circ(t: f32) -> f32 {
    let t1 = t - 1.0;
    (1.0 - t1 * t1).sqrt()
}

/// Ease in and out using a circular curve.
///
/// Circular acceleration and deceleration.
#[inline]
pub fn ease_in_out_circ(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
    } else {
        ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
    }
}

/// Ease in with a back (overshoot) effect.
///
/// Pulls back slightly before accelerating forward.
#[inline]
pub fn ease_in_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    C3 * t * t * t - C1 * t * t
}

/// Ease out with a back (overshoot) effect.
///
/// Overshoots the target before settling.
#[inline]
pub fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    let t1 = t - 1.0;
    1.0 + C3 * t1 * t1 * t1 + C1 * t1 * t1
}

/// Ease in and out with a back (overshoot) effect.
///
/// Pulls back at the start and overshoots at the end.
#[inline]
pub fn ease_in_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C2: f32 = C1 * 1.525;
    
    if t < 0.5 {
        ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
    } else {
        let t1 = 2.0 * t - 2.0;
        (t1.powi(2) * ((C2 + 1.0) * t1 + C2) + 2.0) / 2.0
    }
}

/// Ease in with an elastic (spring) effect.
///
/// Creates a spring-like oscillation at the start.
#[inline]
pub fn ease_in_elastic(t: f32) -> f32 {
    const C4: f32 = (2.0 * std::f32::consts::PI) / 3.0;
    
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        -2.0f32.powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * C4).sin()
    }
}

/// Ease out with an elastic (spring) effect.
///
/// Creates a spring-like oscillation at the end.
#[inline]
pub fn ease_out_elastic(t: f32) -> f32 {
    const C4: f32 = (2.0 * std::f32::consts::PI) / 3.0;
    
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        2.0f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * C4).sin() + 1.0
    }
}

/// Ease in and out with an elastic (spring) effect.
///
/// Creates spring-like oscillations at both start and end.
#[inline]
pub fn ease_in_out_elastic(t: f32) -> f32 {
    const C5: f32 = (2.0 * std::f32::consts::PI) / 4.5;
    
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        -(2.0f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0
    } else {
        (2.0f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0 + 1.0
    }
}

/// Ease in with a bounce effect.
///
/// Simulates a bouncing motion at the start.
#[inline]
pub fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

/// Ease out with a bounce effect.
///
/// Simulates a bouncing motion at the end.
#[inline]
pub fn ease_out_bounce(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t1 = t - 1.5 / D1;
        N1 * t1 * t1 + 0.75
    } else if t < 2.5 / D1 {
        let t1 = t - 2.25 / D1;
        N1 * t1 * t1 + 0.9375
    } else {
        let t1 = t - 2.625 / D1;
        N1 * t1 * t1 + 0.984375
    }
}

/// Ease in and out with a bounce effect.
///
/// Simulates bouncing motions at both start and end.
#[inline]
pub fn ease_in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert_eq!(linear(0.0), 0.0);
        assert_eq!(linear(0.5), 0.5);
        assert_eq!(linear(1.0), 1.0);
    }

    #[test]
    fn test_easing_bounds() {
        let easing_functions: Vec<EasingFunction> = vec![
            linear,
            ease_in_quad,
            ease_out_quad,
            ease_in_out_quad,
            ease_in_cubic,
            ease_out_cubic,
            ease_in_out_cubic,
            ease_in_sine,
            ease_out_sine,
            ease_in_out_sine,
        ];

        for func in easing_functions {
            assert_eq!(func(0.0), 0.0, "Easing function should start at 0.0");
            assert_eq!(func(1.0), 1.0, "Easing function should end at 1.0");
        }
    }

    #[test]
    fn test_ease_in_quad() {
        let result = ease_in_quad(0.5);
        assert!((result - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_ease_out_quad() {
        let result = ease_out_quad(0.5);
        assert!((result - 0.75).abs() < 0.001);
    }
}
