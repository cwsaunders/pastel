mod helper;
mod types;

use helper::{clamp, mod_positive};
use types::Scalar;

/// The hue of a color, represented by an angle (degrees).
#[derive(Debug, Clone, Copy)]
struct Hue {
    unclipped: Scalar,
}

impl Hue {
    pub fn from(unclipped: Scalar) -> Hue {
        Hue { unclipped }
    }

    /// Return a hue value in the interval [0, 360].
    pub fn value(self) -> Scalar {
        if self.unclipped == 360.0 {
            self.unclipped
        } else {
            mod_positive(self.unclipped, 360.0)
        }
    }
}

/// The representation of a color.
///
/// Note:
/// - Colors outside the sRGB gamut (which cannot be displayed on a typical
///   computer screen) can not be represented by `Color`.
/// - The `PartialEq` instance compares two `Color`s by comparing their (integer)
///   RGB values. This is different from comparing the HSL values. For example,
///   HSL has many different representations of black (arbitrary hue and
///   saturation values).
#[derive(Debug, Clone)]
pub struct Color {
    hue: Hue,
    saturation: Scalar,
    lightness: Scalar,
    alpha: Scalar,
}

impl Color {
    pub fn from_hsla(hue: Scalar, saturation: Scalar, lightness: Scalar, alpha: Scalar) -> Color {
        Color {
            hue: Hue::from(hue),
            saturation,
            lightness,
            alpha,
        }
    }

    ///
    pub fn from_hsl(hue: Scalar, saturation: Scalar, lightness: Scalar) -> Color {
        Self::from_hsla(hue, saturation, lightness, 1.0)
    }

    /// Create a `Color` from integer RGB values between 0 and 255 and a floating
    /// point alpha value between 0.0 and 1.0.
    pub fn from_rgba(r: u8, g: u8, b: u8, alpha: Scalar) -> Color {
        // RGB to HSL conversion algorithm adapted from
        // https://en.wikipedia.org/wiki/HSL_and_HSV

        let max_chroma = u8::max(u8::max(r, g), b);
        let min_chroma = u8::min(u8::min(r, g), b);

        let chroma = max_chroma - min_chroma;
        let chroma_s = Scalar::from(chroma) / 255.0;

        let r_s = Scalar::from(r) / 255.0;
        let g_s = Scalar::from(g) / 255.0;
        let b_s = Scalar::from(b) / 255.0;

        let hue = 60.0
            * (if chroma == 0 {
                0.0
            } else {
                if r == max_chroma {
                    mod_positive((g_s - b_s) / chroma_s, 6.0)
                } else if g == max_chroma {
                    (b_s - r_s) / chroma_s + 2.0
                } else {
                    (r_s - g_s) / chroma_s + 4.0
                }
            });

        let lightness = (Scalar::from(max_chroma) + Scalar::from(min_chroma)) / (255.0 * 2.0);
        let saturation = if chroma == 0 {
            0.0
        } else {
            chroma_s / (1.0 - Scalar::abs(2.0 * lightness - 1.0))
        };

        Self::from_hsla(hue, saturation, lightness, alpha)
    }

    /// Create a `Color` from integer RGB values between 0 and 255.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Self::from_rgba(r, g, b, 1.0)
    }

    /// Create a `Color` from RGB and alpha values between 0.0 and 1.0. Values outside this range
    /// will be clamped.
    pub fn from_rgba_scaled(r: Scalar, g: Scalar, b: Scalar, alpha: Scalar) -> Color {
        let r = Scalar::round(clamp(0.0, 255.0, 255.0 * r)) as u8;
        let g = Scalar::round(clamp(0.0, 255.0, 255.0 * g)) as u8;
        let b = Scalar::round(clamp(0.0, 255.0, 255.0 * b)) as u8;

        Self::from_rgba(r, g, b, alpha)
    }

    /// Create a `Color` from RGB values between 0.0 and 1.0. Values outside this range will be
    /// clamped.
    pub fn from_rgb_scaled(r: Scalar, g: Scalar, b: Scalar) -> Color {
        Self::from_rgba_scaled(r, g, b, 1.0)
    }

    /// Convert a `Color` to its hue, saturation, lightness and alpha values. The hue is given
    /// in degrees, as a number between 0.0 and 360.0. Saturation, lightness and alpha are numbers
    /// between 0.0 and 1.0.
    pub fn to_hsla(&self) -> HSLA {
        HSLA {
            h: self.hue.value(),
            s: self.saturation,
            l: self.lightness,
            alpha: self.alpha,
        }
    }

    /// Convert a `Color` to its red, green, blue and alpha values. The RGB values are integers in
    /// the range from 0 to 255. The alpha channel is a number between 0.0 and 1.0.
    pub fn to_rgba(&self) -> RGBA<u8> {
        let c = self.to_rgba_scaled();
        let r = Scalar::round(255.0 * c.r) as u8;
        let g = Scalar::round(255.0 * c.g) as u8;
        let b = Scalar::round(255.0 * c.b) as u8;

        RGBA {
            r,
            g,
            b,
            alpha: self.alpha,
        }
    }

    /// Convert a `Color` to its red, green, blue and alpha values. All numbers are from the range
    /// between 0.0 and 1.0.
    pub fn to_rgba_scaled(&self) -> RGBA<Scalar> {
        let h_s = self.hue.value() / 60.0;
        let chr = (1.0 - Scalar::abs(2.0 * self.lightness - 1.0)) * self.saturation;
        let m = self.lightness - chr / 2.0;
        let x = chr * (1.0 - Scalar::abs(h_s % 2.0 - 1.0));

        struct RGB(Scalar, Scalar, Scalar);

        let col = if h_s < 1.0 {
            RGB(chr, x, 0.0)
        } else if 1.0 <= h_s && h_s < 2.0 {
            RGB(x, chr, 0.0)
        } else if 2.0 <= h_s && h_s < 3.0 {
            RGB(0.0, chr, x)
        } else if 3.0 <= h_s && h_s < 4.0 {
            RGB(0.0, x, chr)
        } else if 4.0 <= h_s && h_s < 5.0 {
            RGB(x, 0.0, chr)
        } else {
            RGB(chr, 0.0, x)
        };

        RGBA {
            r: col.0 + m,
            g: col.1 + m,
            b: col.2 + m,
            alpha: self.alpha,
        }
    }

    /// Pure black.
    pub fn black() -> Color {
        Color::from_hsl(0.0, 0.0, 0.0)
    }

    /// Pure white.
    pub fn white() -> Color {
        Color::from_hsl(0.0, 0.0, 1.0)
    }

    /// Create a gray tone from a lightness value (0.0 is black, 1.0 is white)
    pub fn gray(lightness: Scalar) -> Color {
        Color::from_hsl(0.0, 0.0, lightness)
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        dbg!(other);
        dbg!(self.to_rgba()) == dbg!(other.to_rgba())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RGBA<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub alpha: Scalar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HSLA {
    pub h: Scalar,
    pub s: Scalar,
    pub l: Scalar,
    pub alpha: Scalar,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_mod_positive() {
        assert_relative_eq!(0.5, mod_positive(2.9, 2.4));
        assert_relative_eq!(1.7, mod_positive(-0.3, 2.0));
    }

    #[test]
    fn test_hue_clipping() {
        assert_eq!(43.0, Hue::from(43.0).value());
        assert_eq!(13.0, Hue::from(373.0).value());
        assert_eq!(300.0, Hue::from(-60.0).value());
        assert_eq!(360.0, Hue::from(360.0).value());
    }

    #[test]
    fn test_color_partial_eq() {
        assert_eq!(
            Color::from_hsl(120.0, 0.3, 0.5),
            Color::from_hsl(360.0 + 120.0, 0.3, 0.5),
        );
        assert_eq!(
            Color::from_rgba(1, 2, 3, 0.3),
            Color::from_rgba(1, 2, 3, 0.3),
        );
        assert_eq!(Color::black(), Color::from_hsl(123.0, 0.3, 0.0));
        assert_eq!(Color::white(), Color::from_hsl(123.0, 0.3, 1.0));

        assert_ne!(
            Color::from_hsl(120.0, 0.3, 0.5),
            Color::from_hsl(122.0, 0.3, 0.5),
        );
        assert_ne!(
            Color::from_hsl(120.0, 0.3, 0.5),
            Color::from_hsl(120.0, 0.32, 0.5),
        );
        assert_ne!(
            Color::from_hsl(120.0, 0.3, 0.5),
            Color::from_hsl(120.0, 0.3, 0.52),
        );
        assert_ne!(
            Color::from_hsla(120.0, 0.3, 0.5, 0.9),
            Color::from_hsla(120.0, 0.3, 0.5, 0.901),
        );
        assert_ne!(
            Color::from_rgba(1, 2, 3, 0.3),
            Color::from_rgba(2, 2, 3, 0.3),
        );
        assert_ne!(
            Color::from_rgba(1, 2, 3, 0.3),
            Color::from_rgba(1, 3, 3, 0.3),
        );
        assert_ne!(
            Color::from_rgba(1, 2, 3, 0.3),
            Color::from_rgba(1, 2, 4, 0.3),
        );
    }

    #[test]
    fn test_rgb_to_hsl_conversion() {
        assert_eq!(
            Color::from_hsl(0.0, 0.0, 1.0),
            Color::from_rgb_scaled(1.0, 1.0, 1.0)
        ); // white
        assert_eq!(
            Color::from_hsl(0.0, 0.0, 0.5),
            Color::from_rgb_scaled(0.5, 0.5, 0.5)
        ); // gray
        assert_eq!(
            Color::from_hsl(0.0, 0.0, 0.0),
            Color::from_rgb_scaled(0.0, 0.0, 0.0)
        ); // black
        assert_eq!(
            Color::from_hsl(0.0, 1.0, 0.5),
            Color::from_rgb_scaled(1.0, 0.0, 0.0)
        ); // red
        assert_eq!(
            Color::from_hsl(60.0, 1.0, 0.375),
            Color::from_rgb_scaled(0.75, 0.75, 0.0)
        ); //yellow-green
        assert_eq!(
            Color::from_hsl(120.0, 1.0, 0.25),
            Color::from_rgb_scaled(0.0, 0.5, 0.0)
        ); // green
        assert_eq!(
            Color::from_hsl(240.0, 1.0, 0.75),
            Color::from_rgb_scaled(0.5, 0.5, 1.0)
        ); // blue
        assert_eq!(
            Color::from_hsl(49.5, 0.893, 0.497),
            Color::from_rgb_scaled(0.941, 0.785, 0.053)
        ); // yellow
        assert_eq!(
            Color::from_hsl(162.4, 0.779, 0.447),
            Color::from_rgb_scaled(0.099, 0.795, 0.591)
        ); // cyan 2
    }

    #[test]
    fn test_roundtrip_conversion() {
        let roundtrip = |h, s, l| {
            let color1 = Color::from_hsl(h, s, l);
            let rgb = color1.to_rgba();
            let color2 = Color::from_rgb(rgb.r, rgb.g, rgb.b);
            assert_eq!(color1, color2);
        };

        roundtrip(0.0, 0.0, 1.0);
        roundtrip(0.0, 0.0, 0.5);
        roundtrip(0.0, 0.0, 0.0);
        roundtrip(60.0, 1.0, 0.375);
        roundtrip(120.0, 1.0, 0.25);
        roundtrip(240.0, 1.0, 0.75);
        roundtrip(49.5, 0.893, 0.497);
        roundtrip(162.4, 0.779, 0.447);

        for degree in 0..360 {
            roundtrip(Scalar::from(degree), 0.5, 0.8);
        }
    }
}