use dcf77_utils::DCF77Utils;

/// Return a string version of the 16-bit decimal value, or 0x**** for None.
pub fn str_hex(value: Option<u16>) -> String {
    if let Some(s_value) = value {
        format!("{:>#04x}", s_value)
    } else {
        String::from("0x****")
    }
}

/// Describe the leap second parameters in plain English.
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

/// Determine if we should print a space before this bit (pair).
pub fn is_space_bit(second: u8) -> bool {
    [1, 15, 16, 19, 20, 21, 28, 29, 35, 36, 42, 45, 50, 58, 59].contains(&second)
}

/// Return a textual representation of the weekday, Sunday-Saturday or ? for None.
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
