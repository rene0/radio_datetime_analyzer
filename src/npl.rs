use npl_utils::NPLUtils;
use std::cmp::Ordering;

/// Append the given bit pair to the current NPL structure and to the given buffer for later displaying
///
/// # Arguments
/// * `npl` - the structure to append the bit pair to
/// * `c` - the bit pair to add
/// * `buffer` - buffer storing the bits for later displaying
pub fn append_bits(npl: &mut NPLUtils, c: char, buffer: &mut [char]) {
    if c != '\n' {
        npl.set_current_bit_a(match c {
            '0' | '2' => Some(false),
            '1' | '3' => Some(true),
            _ => None, // '_' or '4' (the 500ms long BOM marker)
        });
        npl.set_current_bit_b(match c {
            '0' | '1' => Some(false),
            '2' | '3' => Some(true),
            _ => None, // '_' or '4' (the 500ms long BOM marker)
        });
    }
    buffer[npl.get_second() as usize] = c;
}

/// Return a string version of the current bit pair (or the EOM newline), optionally prefixed by a space.
///
/// # Arguments
/// * `buffer` - the buffer to stringify
/// * `minute_length` - the number of bit pairs in this minute
pub fn str_bits(buffer: &[char], minute_length: u8) -> String {
    let mut bits = String::from("");
    let offset = match 60.cmp(&minute_length) {
        Ordering::Less => 1,
        Ordering::Equal => 0,
        Ordering::Greater => -1,
    };
    for (idx, c) in buffer.iter().enumerate() {
        if [
            1,
            9,
            17 + offset,
            25 + offset,
            30 + offset,
            36 + offset,
            39 + offset,
            45 + offset,
            52 + offset,
        ]
        .contains(&(idx as isize))
        {
            bits.push(' ');
        }
        bits.push(*c);
        if idx == minute_length as usize {
            break;
        }
    }
    bits
}

/// Return a textual representation of the weekday, Sunday-Saturday or ? for None.
///
/// # Arguments
/// * `weekday` - optional weekday to stringify
pub fn str_weekday(weekday: Option<u8>) -> String {
    String::from(match weekday {
        Some(0) => "Sunday",
        Some(1) => "Monday",
        Some(2) => "Tuesday",
        Some(3) => "Wednesday",
        Some(4) => "Thursday",
        Some(5) => "Friday",
        Some(6) => "Saturday",
        _ => "?",
    })
}

/// Return a string representation of the given value or ? for None.
///
/// # Arguments
/// * `value` - value to stringify
pub fn str_i8(value: Option<i8>) -> String {
    if let Some(s_value) = value {
        format!("{s_value}")
    } else {
        String::from("?")
    }
}

/// Return a vector containing the parity values in English.
///
/// # Arguments
/// * `npl` - structure holding the currently decoded NPL data
pub fn str_parities(npl: &NPLUtils) -> Vec<&str> {
    let mut parities = Vec::new();
    if npl.get_parity_1() == Some(false) {
        parities.push("Year parity bad");
    } else if npl.get_parity_1().is_none() {
        parities.push("Year parity undetermined");
    }
    if npl.get_parity_2() == Some(false) {
        parities.push("Month/day-of-month parity bad");
    } else if npl.get_parity_2().is_none() {
        parities.push("Month/day-of-month parity undetermined");
    }
    if npl.get_parity_3() == Some(false) {
        parities.push("Day-of-week parity bad");
    } else if npl.get_parity_3().is_none() {
        parities.push("Day-of-week parity undetermined");
    }
    if npl.get_parity_4() == Some(false) {
        parities.push("Hour/minute parity bad");
    } else if npl.get_parity_4().is_none() {
        parities.push("Hour/minute parity undetermined");
    }
    parities
}
