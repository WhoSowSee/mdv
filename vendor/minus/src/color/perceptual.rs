// Normalization requires nonzero chroma.
pub(super) fn hue_direction(rgb: [u8; 3]) -> [f64; 2] {
    let linear = rgb.map(|channel| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    });
    let response = [
        [0.412_221_470_8, 0.536_332_536_3, 0.051_445_992_9],
        [0.211_903_498_2, 0.680_699_545_1, 0.107_396_956_6],
        [0.088_302_461_9, 0.281_718_837_6, 0.629_978_700_5],
    ]
    .map(|row| dot(row, linear).cbrt());
    let a = dot([1.977_998_495_1, -2.428_592_205, 0.450_593_709_9], response);
    let b = dot([0.025_904_037_1, 0.782_771_766_2, -0.808_675_766], response);
    let chroma = a.hypot(b);
    [a / chroma, b / chroma]
}

fn dot(weights: [f64; 3], values: [f64; 3]) -> f64 {
    weights[0].mul_add(
        values[0],
        weights[1].mul_add(values[1], weights[2] * values[2]),
    )
}
