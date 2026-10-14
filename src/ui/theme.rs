//! Figma-style custom styling, dark gradient colors, and mathematical easing functions
use eframe::egui::Color32;

pub const BG_LEFT: Color32 = Color32::from_rgb(26, 26, 28);
pub const BG_RIGHT: Color32 = Color32::from_rgb(82, 82, 88);
pub const BG_PANEL: Color32 = Color32::from_rgb(32, 32, 36);

pub const CARD_BG: Color32 = Color32::from_rgb(70, 70, 75);
pub const CARD_BG_HOVER: Color32 = Color32::from_rgb(84, 84, 90);
pub const CARD_BORDER: Color32 = Color32::from_rgb(95, 95, 102);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(215, 215, 222);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(170, 170, 180);

/// Cubic ease-out: starts fast, smoothly decelerates
#[inline]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Quadratic ease-out
#[inline]
pub fn ease_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Smoothstep hermite interpolation
#[inline]
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
