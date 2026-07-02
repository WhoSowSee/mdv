//! Default Nerd Font icons for pretty task-list checkboxes.

use crate::cli::CheckboxShape;

pub fn default_icon(shape: CheckboxShape, state: char) -> Option<char> {
    let codepoint = match shape {
        CheckboxShape::Square => square_icon(state)?,
        CheckboxShape::Circle => circle_icon(state)?,
    };
    char::from_u32(codepoint)
}
fn square_icon(state: char) -> Option<u32> {
    Some(match state {
        ' ' => 0xF0131,  // checkbox-blank-outline
        'x' => 0xF0132,  // checkbox-marked
        '-' => 0xF0375,  // minus-box
        '?' => 0xF078B,  // help-box
        '!' => 0xF0027,  // alert-octagon
        '|' => 0xF0856,  // checkbox-intermediate
        '/' => 0xF0856,  // checkbox-intermediate
        '\\' => 0xF0856, // checkbox-intermediate
        _ => return None,
    })
}
fn circle_icon(state: char) -> Option<u32> {
    Some(match state {
        ' ' => 0xF0130,  // checkbox-blank-outline
        'x' => 0xF0133,  // checkbox-marked-circle
        '-' => 0xF0376,  // minus-circle
        '?' => 0xF02D7,  // help-circle
        '!' => 0xF0028,  // alert-circle
        '|' => 0xF0AA1,  // progress
        '/' => 0xF0AA2,  // progress
        '\\' => 0xF0AA0, // progress
        _ => return None,
    })
}
