use npl_utils::NPLUtils;

/// Append the given bit pair to the current NPL structure
///
/// # Arguments
/// `npl` - the structure to append the bit pair to
/// `c` - the bit pair to add
pub fn append_bits(npl: &mut NPLUtils, c: char) {
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
}

/// Display the current bit pair (or the EOM newline), optionally prefixed by a space.
///
/// # Arguments
/// * `npl` - NPL structure containing the second counter
/// * `c` the bit pair to display
pub fn display_bits(npl: &NPLUtils, c: char) {
    if is_space_bit(npl.get_second()) {
        print!(" ");
    }
    print!("{}", c);
}

/// Decide if a space should be printed in front of the bit pair contained in this second.
///
/// # Arguments
/// * `second` - current second
pub fn is_space_bit(second: u8) -> bool {
    [1, 9, 17, 25, 30, 36, 39, 45, 52].contains(&second)
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
        format!("{}", s_value)
    } else {
        String::from("?")
    }
}

/// Display the parity values in English.
///
/// # Arguments
/// * `npl` - structure holding the currently decoded NPL data
pub fn display_parities(npl: &NPLUtils) {
    if npl.get_parity_1() == Some(false) {
        println!("Year parity bad");
    } else if npl.get_parity_1().is_none() {
        println!("Year parity undetermined");
    }
    if npl.get_parity_2() == Some(false) {
        println!("Month/day-of-month parity bad");
    } else if npl.get_parity_2().is_none() {
        println!("Month/day-of-month parity undetermined");
    }
    if npl.get_parity_3() == Some(false) {
        println!("Day-of-week parity bad");
    } else if npl.get_parity_3().is_none() {
        println!("Day-of-week parity undetermined");
    }
    if npl.get_parity_4() == Some(false) {
        println!("Hour/minute parity bad");
    } else if npl.get_parity_4().is_none() {
        println!("Hour/minute parity undetermined");
    }
}
