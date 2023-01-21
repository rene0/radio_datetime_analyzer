/// Determine if we should print a space before this bit (pair).
pub fn is_space_bit(second: u8) -> bool {
    [1, 9, 17, 25, 30, 36, 39, 45, 52].contains(&second)
}

/// Return a textual representation of the weekday, Sunday-Saturday or ? for None.
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
pub fn str_i8(value: Option<i8>) -> String {
    if let Some(s_value) = value {
        format!("{}", s_value)
    } else {
        String::from("?")
    }
}
