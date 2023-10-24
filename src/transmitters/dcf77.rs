use dcf77_utils::DCF77Utils;
use radio_datetime_utils::RadioDateTimeUtils;
use crate::{str_datetime, str_jumps};

/// Analyze a DCF77 logfile.
///
/// # Arguments
/// `buffer` - the buffer containing the DCF77 logfile
pub fn analyze_buffer(buffer: String) /*-> Vec<&str>*/ {
    let mut dcf77 = DCF77Utils::default();
    for c in buffer.chars() {
        if !['0', '1', '_', '\n'].contains(&c) {
            continue;
        }
        append_bit(&mut dcf77, c);
        print!("{}", str_bit(&dcf77, c));
        if c == '\n' {
            let rdt: RadioDateTimeUtils;
            let dst: Option<u8>;
            // force-feed the missing EOM bit
            dcf77.set_current_bit(None);
            dcf77.increase_second();

            dcf77.decode_time();
            dcf77.force_new_minute();
            rdt = dcf77.get_radio_datetime();
            dst = rdt.get_dst();
            println!(
                "first_minute={} second={} this_minute_length={} next_minute_length={}",
                dcf77.get_first_minute(),
                dcf77.get_second(),
                dcf77.get_this_minute_length(),
                dcf77.get_next_minute_length()
            );
            print!("{}", str_datetime(&rdt, str_weekday(rdt.get_weekday()), dst));
            println!(
                " [{}] [{}]",
                leap_second_info(
                    rdt.get_leap_second(),
                    dcf77.get_leap_second_is_one(),
                ),
                str_call_bit(&dcf77),
            );
            println!(
                "Third-party buffer={}",
                str_hex(dcf77.get_third_party_buffer())
            );
            for parity in str_parities(&dcf77) {
                println!("{parity}")
            }
            for check in str_check_bits(&dcf77) {
                println!("{check}")
            }
            for jump in str_jumps(&rdt) {
                println!("{}", jump);
            }
            println!();
        }
        dcf77.increase_second();
    }
}

/// Append the given bit to the current DCF77 structure
///
/// # Arguments
/// `dcf77` - the structure to append the bit to
/// `c` - the bit to add. The newline is there to force a new minute, it is a not a bit in itself.
pub fn append_bit(dcf77: &mut DCF77Utils, c: char) {
    if c != '\n' {
        dcf77.set_current_bit(match c {
            '0' => Some(false),
            '1' => Some(true),
            '_' => None,
            _ => panic!("dcf77::append_bit(): impossible character '{c}'"),
        });
    }
}

/// Return a string version of the current bit (or the EOM newline), optionally prefixed by a space.
///
/// # Arguments
/// * `dcf77` - DCF77 structure containing the second counter
/// * `c` the bit to stringify
pub fn str_bit(dcf77: &DCF77Utils, c: char) -> String {
    let mut bit = String::from("");
    if [1, 15, 16, 19, 20, 21, 28, 29, 35, 36, 42, 45, 50, 58, 59].contains(&dcf77.get_second()) {
        bit.push(' ');
    }
    bit.push(c);
    bit
}

/// Return a string version of the 16-bit decimal value, or 0x**** for None.
///
/// # Arguments
/// * `value` - the value to stringify, if any.
pub fn str_hex(value: Option<u16>) -> String {
    if let Some(s_value) = value {
        format!("0x{s_value:>04x}")
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
        let mut need_comma = false;
        // LEAP_ANNOUNCED is mutually exclusive with [LEAP_PROCESSED, is_one, LEAP_MISSING]
        // see radio_datetime_utils::set_leap_second()
        if s_leap & radio_datetime_utils::LEAP_ANNOUNCED != 0 {
            s += "announced";
            need_comma = true;
        }
        if s_leap & radio_datetime_utils::LEAP_PROCESSED != 0 {
            if need_comma {
                s += ",";
            }
            s += "processed";
            if is_one == Some(true) {
                s += ",one";
            }
            need_comma = true;
        }
        if s_leap & radio_datetime_utils::LEAP_MISSING != 0 {
            if need_comma {
                s += ",";
            }
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
        None => "?",
        _ => {
            panic!(
                "dcf77::str_weekday(): impossible weekday 'Some({})'",
                weekday.unwrap()
            );
        }
    })
}

/// Return a vector containing the parity values in plain English.
///
/// # Arguments
/// * `dcf77` - structure holding the currently decoded DCF77 data
pub fn str_parities(dcf77: &DCF77Utils) -> Vec<&str> {
    let mut parities = Vec::new();
    if dcf77.get_parity_1() == Some(true) {
        parities.push("Minute parity bad");
    } else if dcf77.get_parity_1().is_none() {
        parities.push("Minute parity undetermined");
    }
    if dcf77.get_parity_2() == Some(true) {
        parities.push("Hour parity bad");
    } else if dcf77.get_parity_2().is_none() {
        parities.push("Hour parity undetermined");
    }
    if dcf77.get_parity_3() == Some(true) {
        parities.push("Date parity bad");
    } else if dcf77.get_parity_3().is_none() {
        parities.push("Date parity undetermined");
    }
    parities
}

/// Return if the call bit is active, in plain English.
///
/// # Arguments
/// * `dcf77` - structure holding the currently decoded DCF77 data
pub fn str_call_bit(dcf77: &DCF77Utils) -> String {
    String::from(match dcf77.get_call_bit() {
        Some(false) => "",
        Some(true) => "call",
        None => "?",
    })
}

/// Return a vector containing if bit 0 or 20 are wrong or undetermined, in plain English.
///
/// # Arguments
/// * `dcf77` - structure holding the currently decoded DCF77 data
pub fn str_check_bits(dcf77: &DCF77Utils) -> Vec<&str> {
    let mut checks = Vec::new();
    if dcf77.get_bit_0() == Some(true) {
        checks.push("Bit 0 is wrong");
    } else if dcf77.get_bit_0().is_none() {
        checks.push("Bit 0 is undetermined")
    }
    if dcf77.get_bit_20() == Some(false) {
        checks.push("Bit 20 is wrong");
    } else if dcf77.get_bit_20().is_none() {
        checks.push("Bit 20 is undetermined")
    }
    checks
}

#[cfg(test)]
mod tests {
    use super::*;
    use radio_datetime_utils::{LEAP_ANNOUNCED, LEAP_MISSING, LEAP_PROCESSED};

    const LE_EMPTY: &str = "";
    const LE_ANN: &str = "announced";
    const LE_PROC: &str = "processed";
    const LE_PROC1: &str = "processed,one";
    const LE_PROC_MISSING: &str = "processed,missing";
    const LE_PROC1_MISSING: &str = "processed,one,missing";

    #[test]
    fn test_leap_second_info_none_none() {
        assert_eq!(leap_second_info(None, None), LE_EMPTY);
    }
    #[test]
    fn test_leap_second_info_none_false() {
        assert_eq!(leap_second_info(None, Some(false)), LE_EMPTY);
    }
    #[test]
    fn test_leap_second_info_none_true() {
        assert_eq!(leap_second_info(None, Some(true)), LE_EMPTY);
    }
    #[test]
    fn test_leap_second_info_ann_none() {
        assert_eq!(leap_second_info(Some(LEAP_ANNOUNCED), None), LE_ANN);
    }
    #[test]
    fn test_leap_second_info_ann_false() {
        assert_eq!(leap_second_info(Some(LEAP_ANNOUNCED), Some(false)), LE_ANN);
    }
    #[test]
    fn test_leap_second_info_ann_true() {
        assert_eq!(leap_second_info(Some(LEAP_ANNOUNCED), Some(true)), LE_ANN);
    }
    #[test]
    fn test_leap_second_info_proc_none() {
        assert_eq!(leap_second_info(Some(LEAP_PROCESSED), None), LE_PROC);
    }
    #[test]
    fn test_leap_second_info_proc_false() {
        assert_eq!(leap_second_info(Some(LEAP_PROCESSED), Some(false)), LE_PROC);
    }
    #[test]
    fn test_leap_second_info_proc_true() {
        assert_eq!(leap_second_info(Some(LEAP_PROCESSED), Some(true)), LE_PROC1);
    }
    #[test]
    fn test_leap_second_info_proc_missing_none() {
        assert_eq!(
            leap_second_info(Some(LEAP_PROCESSED | LEAP_MISSING), None),
            LE_PROC_MISSING
        );
    }
    #[test]
    fn test_leap_second_info_proc_missing_false() {
        assert_eq!(
            leap_second_info(Some(LEAP_PROCESSED | LEAP_MISSING), Some(false)),
            LE_PROC_MISSING
        );
    }
    #[test]
    fn test_leap_second_info_proc_missing_true() {
        assert_eq!(
            leap_second_info(Some(LEAP_PROCESSED | LEAP_MISSING), Some(true)),
            LE_PROC1_MISSING
        );
    }

    #[test]
    #[should_panic]
    fn test_append_bit_panic() {
        let mut dcf77 = DCF77Utils::default();
        append_bit(&mut dcf77, '!');
    }

    #[test]
    fn test_append_bits_bunch() {
        let mut dcf77 = DCF77Utils::default();
        append_bit(&mut dcf77, '0');
        assert_eq!(dcf77.get_current_bit(), Some(false));
        dcf77.increase_second();
        append_bit(&mut dcf77, '\n');
        // this normally forces a new minute
        assert_eq!(dcf77.get_current_bit(), None);
        dcf77.increase_second();
        append_bit(&mut dcf77, '1');
        assert_eq!(dcf77.get_current_bit(), Some(true));
        dcf77.increase_second();
        append_bit(&mut dcf77, '_'); // broken/empty bit
        assert_eq!(dcf77.get_current_bit(), None);
        dcf77.increase_second();
    }
}
