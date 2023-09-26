use msf60_utils::MSFUtils;
use std::cmp::Ordering;

/// Append the given bit pair to the current MSF structure and to the given buffer for later
/// displaying.
///
/// # Arguments
/// * `msf` - the structure to append the bit pair to
/// * `c` - the bit pair to add. The newline is there to force a new minute, it is a not a bit pair
///         in itself.
/// * `buffer` - buffer storing the bits for later displaying
pub fn append_bits(msf: &mut MSFUtils, c: char, buffer: &mut [char]) {
    if c != '\n' {
        // 4 is the 500ms long BOM marker
        msf.set_current_bit_a(match c {
            '0' | '2' => Some(false),
            '1' | '3' => Some(true),
            '4' | '_' => None,
            _ => panic!("msf::append_bits(): impossible character '{c}'"),
        });
        msf.set_current_bit_b(match c {
            '0' | '1' => Some(false),
            '2' | '3' => Some(true),
            '4' | '_' => None,
            _ => panic!("msf::append_bits(): impossible character '{c}'"),
        });
    }
    buffer[msf.get_second() as usize] = c;
}

/// Return a string version of the all the bit pairs (or the EOM newline) in this minute.
/// Each bit pair is optionally prefixed by a space.
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
            // cut off any remaining characters,
            // i.e. the \n and any empty space to accommodate for positive leap seconds
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
        None => "?",
        _ => {
            panic!("msf::str_weekday(): impossible weekday 'Some({})'", weekday.unwrap());
        }
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
/// * `msf` - structure holding the currently decoded MSF data
pub fn str_parities(msf: &MSFUtils) -> Vec<&str> {
    let mut parities = Vec::new();
    if msf.get_parity_1() == Some(false) {
        parities.push("Year parity bad");
    } else if msf.get_parity_1().is_none() {
        parities.push("Year parity undetermined");
    }
    if msf.get_parity_2() == Some(false) {
        parities.push("Month/day-of-month parity bad");
    } else if msf.get_parity_2().is_none() {
        parities.push("Month/day-of-month parity undetermined");
    }
    if msf.get_parity_3() == Some(false) {
        parities.push("Day-of-week parity bad");
    } else if msf.get_parity_3().is_none() {
        parities.push("Day-of-week parity undetermined");
    }
    if msf.get_parity_4() == Some(false) {
        parities.push("Hour/minute parity bad");
    } else if msf.get_parity_4().is_none() {
        parities.push("Hour/minute parity undetermined");
    }
    parities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_append_bits_panic() {
        let mut buffer = [' '; msf60_utils::BIT_BUFFER_SIZE];
        let mut msf = MSFUtils::default();
        append_bits(&mut msf, '!', &mut buffer);
    }

    #[test]
    fn test_append_bits_bunch() {
        let mut buffer = [' '; msf60_utils::BIT_BUFFER_SIZE];
        let mut msf = MSFUtils::default();
        append_bits(&mut msf, '0', &mut buffer);
        assert_eq!(msf.get_current_bit_a(), Some(false));
        assert_eq!(msf.get_current_bit_b(), Some(false));
        msf.increase_second();
        append_bits(&mut msf, '1', &mut buffer);
        assert_eq!(msf.get_current_bit_a(), Some(true));
        assert_eq!(msf.get_current_bit_b(), Some(false));
        msf.increase_second();
        append_bits(&mut msf, '_', &mut buffer); // broken bit
        assert_eq!(msf.get_current_bit_a(), None);
        assert_eq!(msf.get_current_bit_b(), None);
        msf.increase_second();
        append_bits(&mut msf, '2', &mut buffer);
        assert_eq!(msf.get_current_bit_a(), Some(false));
        assert_eq!(msf.get_current_bit_b(), Some(true));
        msf.increase_second();
        append_bits(&mut msf, '\n', &mut buffer);
        // not added to msf.bit_*, this normally forces a new minute
        assert_eq!(msf.get_current_bit_a(), None);
        assert_eq!(msf.get_current_bit_b(), None);
        msf.increase_second();
        append_bits(&mut msf, '3', &mut buffer);
        assert_eq!(msf.get_current_bit_a(), Some(true));
        assert_eq!(msf.get_current_bit_b(), Some(true));
        msf.increase_second();
        append_bits(&mut msf, '4', &mut buffer); // BOM
        assert_eq!(msf.get_current_bit_a(), None);
        assert_eq!(msf.get_current_bit_a(), None);
        assert_eq!(buffer[0..7], ['0', '1', '_', '2', '\n', '3', '4']);
    }
}
