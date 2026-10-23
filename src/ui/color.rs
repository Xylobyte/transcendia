use egui::Color32;

/// Parses `#RGB`, `#RRGGBB`, `#RRGGBBAA`, `rgb(r,g,b)` and `rgba(r,g,b,a)` (a in 0..=1).
pub fn try_parse_color(input: &str) -> Option<Color32> {
    let s = input.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if !hex.is_ascii() {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        let nibble = |i: usize| u8::from_str_radix(&hex[i..i + 1], 16).ok().map(|n| n * 17);
        return match hex.len() {
            3 => Some(Color32::from_rgb(nibble(0)?, nibble(1)?, nibble(2)?)),
            6 => Some(Color32::from_rgb(byte(0)?, byte(2)?, byte(4)?)),
            8 => Some(Color32::from_rgba_unmultiplied(
                byte(0)?,
                byte(2)?,
                byte(4)?,
                byte(6)?,
            )),
            _ => None,
        };
    }
    let lower = s.to_ascii_lowercase();
    let (inner, has_alpha) = if let Some(r) = lower.strip_prefix("rgba(") {
        (r.strip_suffix(')')?, true)
    } else {
        let r = lower.strip_prefix("rgb(")?;
        (r.strip_suffix(')')?, false)
    };
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    if parts.len() != if has_alpha { 4 } else { 3 } {
        return None;
    }
    let channel = |p: &str| -> Option<u8> {
        let v: f32 = p.parse().ok()?;
        v.is_finite().then(|| v.round().clamp(0.0, 255.0) as u8)
    };
    let (r, g, b) = (channel(parts[0])?, channel(parts[1])?, channel(parts[2])?);
    let a = if has_alpha {
        let v: f32 = parts[3].parse().ok()?;
        if !v.is_finite() {
            return None;
        }
        (v.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    };
    Some(Color32::from_rgba_unmultiplied(r, g, b, a))
}

/// Like `try_parse_color`, falling back to `default` on invalid input.
pub fn parse_color(input: &str, default: Color32) -> Color32 {
    try_parse_color(input).unwrap_or(default)
}

/// `#RRGGBB` when opaque, `#RRGGBBAA` otherwise.
pub fn format_color(color: Color32) -> String {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    if a == 255 {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEF: Color32 = Color32::WHITE;

    #[test]
    fn parses_hex_forms() {
        assert_eq!(parse_color("#f00", DEF), Color32::from_rgb(255, 0, 0));
        assert_eq!(parse_color("#102030", DEF), Color32::from_rgb(16, 32, 48));
        assert_eq!(
            parse_color("#10203080", DEF),
            Color32::from_rgba_unmultiplied(16, 32, 48, 128)
        );
    }

    #[test]
    fn parses_functional_forms() {
        assert_eq!(
            parse_color("rgb(227, 227, 227)", DEF),
            Color32::from_rgb(227, 227, 227)
        );
        assert_eq!(
            parse_color("rgba(0,0,0,0.5)", DEF),
            Color32::from_rgba_unmultiplied(0, 0, 0, 128)
        );
    }

    #[test]
    fn invalid_gives_default() {
        for s in ["", "red", "#12", "#ggg", "#12345", "rgb(1,2)", "rgba(1,2,3)", "rgb(a,b,c)", "#é1é"] {
            assert_eq!(parse_color(s, Color32::BLUE), Color32::BLUE, "{s}");
        }
    }

    #[test]
    fn round_trips() {
        for s in ["#FFFFFF", "#000000", "#12AB34", "#12AB3480"] {
            assert_eq!(format_color(parse_color(s, DEF)), s);
        }
        assert_eq!(format_color(parse_color("#fff", DEF)), "#FFFFFF");
        assert_eq!(format_color(parse_color("rgb(1,2,3)", DEF)), "#010203");
    }
}
