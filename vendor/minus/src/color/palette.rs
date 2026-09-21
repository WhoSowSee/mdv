use super::ColorDepth;
use super::perceptual::hue_direction;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Color {
    Indexed(u8),
    Rgb([u8; 3]),
}

const ANSI16: [[u8; 3]; 16] = [
    [0, 0, 0],
    [128, 0, 0],
    [0, 128, 0],
    [128, 128, 0],
    [0, 0, 128],
    [128, 0, 128],
    [0, 128, 128],
    [192, 192, 192],
    [128, 128, 128],
    [255, 0, 0],
    [0, 255, 0],
    [255, 255, 0],
    [0, 0, 255],
    [255, 0, 255],
    [0, 255, 255],
    [255, 255, 255],
];

static ANSI_HUES: LazyLock<[(u8, [f64; 2]); 6]> = LazyLock::new(|| {
    [1, 3, 2, 6, 4, 5].map(|index| (index, hue_direction(ANSI16[usize::from(index)])))
});

/// Return the conventional RGB approximation of an ANSI palette entry.
#[must_use]
pub fn ansi256_to_rgb(index: u8) -> (u8, u8, u8) {
    let rgb = match index {
        0..=15 => ANSI16[usize::from(index)],
        16..=231 => {
            const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
            let index = usize::from(index - 16);
            [LEVELS[index / 36], LEVELS[index / 6 % 6], LEVELS[index % 6]]
        }
        _ => [8 + (index - 232) * 10; 3],
    };
    rgb.into()
}

impl Color {
    pub(super) fn rgb(self) -> [u8; 3] {
        match self {
            Self::Rgb(rgb) => rgb,
            Self::Indexed(index) => ansi256_to_rgb(index).into(),
        }
    }

    pub(super) fn index(self, depth: ColorDepth) -> u8 {
        if let Self::Indexed(index) = self
            && (index < 16 || depth != ColorDepth::Ansi16)
        {
            return index;
        }
        let rgb = self.rgb();
        if depth == ColorDepth::Ansi16 {
            return ansi16_index(rgb);
        }
        (16..=255)
            .min_by_key(|&index| distance(rgb, Self::Indexed(index).rgb()))
            .unwrap()
    }
}

fn ansi16_index(rgb: [u8; 3]) -> u8 {
    let [r, g, b] = rgb.map(i32::from);
    let maximum = r.max(g).max(b);
    let chroma = maximum - r.min(g).min(b);
    let brightness = luma(rgb);
    if maximum < 64 || chroma * 4 < maximum {
        return [0, 8, 7, 15]
            .into_iter()
            .min_by_key(|&index| (brightness - luma(Color::Indexed(index).rgb())).abs())
            .unwrap();
    }
    let [a, b] = hue_direction(rgb);
    let similarity = |direction: [f64; 2]| a.mul_add(direction[0], b * direction[1]);
    let &(index, _) = ANSI_HUES
        .iter()
        .max_by(|(_, left), (_, right)| similarity(*left).total_cmp(&similarity(*right)))
        .unwrap();
    [index, index + 8]
        .into_iter()
        .min_by_key(|&index| (brightness - luma(Color::Indexed(index).rgb())).abs())
        .unwrap()
}

fn luma(rgb: [u8; 3]) -> i32 {
    let [r, g, b] = rgb.map(i32::from);
    299 * r + 587 * g + 114 * b
}

fn distance(left: [u8; 3], right: [u8; 3]) -> i32 {
    let [r, g, b] = std::array::from_fn(|i| i32::from(left[i]) - i32::from(right[i]));
    let red_mean = i32::midpoint(i32::from(left[0]), i32::from(right[0]));
    (((512 + red_mean) * r * r) >> 8) + 4 * g * g + (((767 - red_mean) * b * b) >> 8)
}

pub(super) fn contrasting_foreground(background: u8) -> u8 {
    if luma(Color::Indexed(background).rgb()) >= 128_000 {
        0
    } else {
        15
    }
}
