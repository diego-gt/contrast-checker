use core::fmt;

#[derive(Debug)]
enum HexToDecError {
    RightDigitInvalid,
    LeftDigitInvalid,
    LeftDigitOutOfRange,
    RightDigitOutOfRange,
    InputLengthOutOfRange,
}

#[derive(Debug)]
enum ColorFromHexError {
    InputIsEmpty,
    InputIsNotAscii,
    InvalidInputLength,
}

/// This function expects a trimmed, 2 digit hex value without #
fn hex_to_dec(hex: &str) -> Result<u8, HexToDecError> {
    if hex.len() != 2 {
        return Err(HexToDecError::InputLengthOutOfRange);
    }

    let mut chars = hex.chars();

    // We'll always have 2 chars if we've reached this point, so it's okay to consume the value
    // with unwrap.
    let left = chars.next().unwrap();
    let right = chars.next().unwrap();

    let left_value = match left.to_digit(16) {
        Some(x) => x,
        None => return Err(HexToDecError::LeftDigitInvalid),
    };

    let right_value = match right.to_digit(16) {
        Some(x) => x,
        None => return Err(HexToDecError::RightDigitInvalid),
    };

    // If the input contained characters 'larger' than F
    if left_value > 15 {
        return Err(HexToDecError::LeftDigitOutOfRange);
    }

    if right_value > 15 {
        return Err(HexToDecError::RightDigitOutOfRange);
    }

    // We now that each value is less than 16 and will fit in an u8
    Ok(u8::try_from(left_value).ok().unwrap() * 16u8 + u8::try_from(right_value).ok().unwrap())
}

struct Color {
    red: f32,
    green: f32,
    blue: f32,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(r: {}, g: {}, b: {})", self.red, self.green, self.blue)
    }
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Color {
        Color {
            red: f32::from(r),
            green: f32::from(g),
            blue: f32::from(b),
        }
    }

    /// This function expects an RGB value of 6 hex digits with or without a starting #
    fn from_hex(hex: &str) -> Result<Color, ColorFromHexError> {
        if hex.is_empty() {
            return Err(ColorFromHexError::InputIsEmpty);
        }

        if !hex.is_ascii() {
            return Err(ColorFromHexError::InputIsNotAscii);
        }

        let lowercase = hex.to_lowercase();
        let mut trimmed_input = lowercase.trim();

        // Only allow input like RRGGBB or #RRGGBB
        if trimmed_input.len() < 6 || trimmed_input.len() > 7 {
            return Err(ColorFromHexError::InvalidInputLength);
        }

        if trimmed_input.starts_with("#") {
            trimmed_input = &trimmed_input[1..];
        }

        let red_hex = &trimmed_input[0..2];
        println!("red hex: {red_hex}");
        let green_hex = &trimmed_input[2..4];
        println!("green hex: {green_hex}");
        let blue_hex = &trimmed_input[4..];
        println!("blue hex: {blue_hex}");

        Ok(Color {
            red: f32::from(hex_to_dec(red_hex).unwrap()),
            green: f32::from(hex_to_dec(green_hex).unwrap()),
            blue: f32::from(hex_to_dec(blue_hex).unwrap()),
        })
    }

    fn normalize(&self) -> Color {
        Color {
            red: self.red / 255f32,
            green: self.green / 255f32,
            blue: self.blue / 255f32,
        }
    }
}

/// This expects the input component to be normalized by dividing it by 255.
/// CIE XYZ is a device independent color space
/// Magical values come from the official sRGB spec. https://en.wikipedia.org/wiki/SRGB
fn srgb_component_to_cie_xyz(normalized_component: f32) -> f32 {
    if normalized_component <= 0.04045 {
        normalized_component / 12.92
    } else {
        let tmp: f32 = (normalized_component + 0.055) / 1.055;
        tmp.powf(2.4f32)
    }
}

/// Formula to calculate relative luminance obtained from https://www.w3.org/TR/WCAG21/relative-luminance.html
/// Magical values come from the official sRGB spec. https://en.wikipedia.org/wiki/SRGB
fn relative_luminance(color: &Color) -> f32 {
    let normalized_color = color.normalize();

    let red_component_luminance: f32 = srgb_component_to_cie_xyz(normalized_color.red);
    let green_component_luminance: f32 = srgb_component_to_cie_xyz(normalized_color.green);
    let blue_component_luminance: f32 = srgb_component_to_cie_xyz(normalized_color.blue);

    0.2126 * red_component_luminance
        + 0.7152 * green_component_luminance
        + 0.0722 * blue_component_luminance
}

/// Formula for contrast ratio obtained from https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio
fn contrast_ratio(foreground: &Color, background: &Color) -> f32 {
    let foreground_luminance = relative_luminance(foreground);
    let background_luminance = relative_luminance(background);

    if foreground_luminance > background_luminance {
        (foreground_luminance + 0.05) / (background_luminance + 0.05)
    } else {
        (background_luminance + 0.05) / (foreground_luminance + 0.05)
    }
}

fn main() {
    let white = Color::from_hex("#FFFFFF").unwrap();
    println!("white from hex: {white}");
    let white_luminance = relative_luminance(&white);

    let target = Color::new(242, 108, 167);
    println!("target from rgb: {target}");
    let target_luminance = relative_luminance(&target);

    println!("luminance of white is {}", white_luminance);
    println!("luminance of target is {}", target_luminance);
    println!(
        "contrast target, white is {}",
        contrast_ratio(&target, &white)
    );
    println!(
        "contrast white, target is {}",
        contrast_ratio(&white, &target)
    );
}
