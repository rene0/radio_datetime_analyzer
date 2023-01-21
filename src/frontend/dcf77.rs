use dcf77_utils::DCF77Utils;

/// Append the given bit to the current DCF77 structure
///
/// # Arguments
/// `dcf77` - the structure to append the bit to
/// `c` - the bit to add
pub fn append_bit(dcf77: &mut DCF77Utils, c: char) {
    if c != '\n' {
        dcf77.set_current_bit(match c {
            '0' => Some(false),
            '1' => Some(true),
            _ => None, // always '_' in this case, but match must be exhaustive
        });
    }
}

/// Display the current bit (or the EOM newline), optionally prefixed by a space.
///
/// # Arguments
/// * `dcf77` - DCF77 structure containing the second counter
/// * `c` the bit to display
pub fn display_bit(dcf77: &DCF77Utils, c: char) {
    if [1, 15, 16, 19, 20, 21, 28, 29, 35, 36, 42, 45, 50, 58, 59].contains(&dcf77.get_second()) {
        print!(" ");
    }
    print!("{}", c);
}

/// Return a string version of the 16-bit decimal value, or 0x**** for None.
///
/// # Arguments
/// * `value` - the value to stringify, if any.
pub fn str_hex(value: Option<u16>) -> String {
    if let Some(s_value) = value {
        format!("{:>#04x}", s_value)
    } else {
        String::from("0x****")
    }
}

/// Describe the leap second parameters in plain English.
///
/// # Arguments
/// * `leap_second` - leap second value as decoded by radio_datetime_utils
/// * `is_one` - the bit value of the leap second (if any)
pub fn leap_second_info(leap_second: Option<u8>, is_one: Option<bool>) -> String {
    let mut s = String::from("");
    if let Some(s_leap) = leap_second {
        if s_leap & radio_datetime_utils::LEAP_ANNOUNCED != 0 {
            s += "announced";
        }
        if s_leap & radio_datetime_utils::LEAP_PROCESSED != 0 {
            s += "processed";
            if is_one.unwrap() {
                s += ",one";
            }
        }
        if s_leap & radio_datetime_utils::LEAP_MISSING != 0 {
            s += "missing";
        }
    }
    s
}

/// Return a textual representation of the weekday, Sunday-Saturday or ? for None.
///
/// # Arguments
/// * `weekday` - optional weekday to stringify
pub fn str_weekday(weekday: Option<u8>) -> String {
    String::from(match weekday {
        Some(1) => "Monday",
        Some(2) => "Tuesday",
        Some(3) => "Wednesday",
        Some(4) => "Thursday",
        Some(5) => "Friday",
        Some(6) => "Saturday",
        Some(7) => "Sunday",
        _ => "?",
    })
}

/// Display the parity values in plain English.
///
/// # Arguments
/// * `dcf77` - structure holding the currently decoded DCF77 data
pub fn display_parities(dcf77: &DCF77Utils) {
    if dcf77.get_parity_1() == Some(true) {
        println!("Minute parity bad");
    } else if dcf77.get_parity_1().is_none() {
        println!("Minute parity undetermined");
    }
    if dcf77.get_parity_1() == Some(true) {
        println!("Hour parity bad");
    } else if dcf77.get_parity_2().is_none() {
        println!("Hour parity undetermined");
    }
    if dcf77.get_parity_1() == Some(true) {
        println!("Date parity bad");
    } else if dcf77.get_parity_3().is_none() {
        println!("Date parity undetermined");
    }
}
