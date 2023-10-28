use crate::{str_datetime, str_jumps};
use dcf77_utils::DCF77Utils;

/// Analyze a DCF77 logfile, return the input with the results interleaved.
///
/// # Arguments
/// `buffer` - the buffer containing the DCF77 logfile
pub fn analyze_buffer(buffer: &str) -> Vec<String> {
    let mut dcf77 = DCF77Utils::default();
    let mut res = Vec::new();
    for c in buffer.chars() {
        if !['0', '1', '_', '\n'].contains(&c) {
            continue;
        }
        append_bit(&mut dcf77, c);
        res.push(str_bit(&dcf77, c));
        if c == '\n' {
            // force-feed the missing EOM bit
            dcf77.set_current_bit(None);
            dcf77.increase_second();

            dcf77.decode_time();
            dcf77.force_new_minute();
            let rdt = dcf77.get_radio_datetime();
            let dst = rdt.get_dst();
            res.push(format!(
                "first_minute={} second={} this_minute_length={} next_minute_length={}\n",
                dcf77.get_first_minute(),
                dcf77.get_second(),
                dcf77.get_this_minute_length(),
                dcf77.get_next_minute_length()
            ));
            res.push(str_datetime(&rdt, str_weekday(rdt.get_weekday()), dst));
            res.push(format!(
                " [{}] [{}]\n",
                leap_second_info(rdt.get_leap_second(), dcf77.get_leap_second_is_one()),
                str_call_bit(&dcf77),
            ));
            res.push(format!(
                "Third-party buffer={}\n",
                str_hex(dcf77.get_third_party_buffer())
            ));
            for parity in str_parities(&dcf77) {
                res.push(format!("{parity}\n"));
            }
            for check in str_check_bits(&dcf77) {
                res.push(format!("{check}\n"));
            }
            for jump in str_jumps(&rdt) {
                res.push(format!("{jump}\n"));
            }
            res.push(String::from("\n"));
        }
        dcf77.increase_second();
    }
    res
}

/// Append the given bit to the current DCF77 structure
///
/// # Arguments
/// `dcf77` - the structure to append the bit to
/// `c` - the bit to add. The newline is there to force a new minute, it is a not a bit in itself.
fn append_bit(dcf77: &mut DCF77Utils, c: char) {
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
fn str_bit(dcf77: &DCF77Utils, c: char) -> String {
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
fn str_hex(value: Option<u16>) -> String {
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
fn leap_second_info(leap_second: Option<u8>, is_one: Option<bool>) -> String {
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
fn str_weekday(weekday: Option<u8>) -> String {
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
fn str_parities(dcf77: &DCF77Utils) -> Vec<&str> {
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
fn str_call_bit(dcf77: &DCF77Utils) -> String {
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
fn str_check_bits(dcf77: &DCF77Utils) -> Vec<&str> {
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

    #[test]
    fn test_analyze_logfile() {
        const LOG: &str = "00000000000000000010100011011110001110001110101001100110011
00000000000000000010110011010110001110001110101001100110011
00000000000000000010100000000000000010000001110000000000000
00000000000000000010110000001000000010000001110000000000000
=
00011011010011100100110101100100010010011011000001100010000
00111001001111000100101101100100010010011011000001100010000
0001111001010110010011110110_______________________________
00000000111000100100110100011100010010011011000001100010000
00101010000001100100101100011100010010011011000001100010000
00001100010010100100111100010100010010011011000001100010000
00001101100000100100100010010100010010011011000001100010000
001000100010010001001______________________________________
00010100010101000100111101011100010010011011000001100010000
00101010011111100100100011011100010010011011000001100010000
=
0 00110100000101 000101 10011010 0000000 111001 111 11000 100010001
0 10010000101011 000101 00000000 1000001 111001 111 11000 100010001
0 00010110001111 010101 10000001 1000001 111001 111 11000 100010001
0 01010000000001 010101 00011011 1000001 111001 111 11000 100010001
0 00010011100011 010101 10011010 1000001 111001 111 11000 100010001
0 01100111010010 011001 00000000 1100000 111001 111 11000 100010001
0 00000010100111 001001 10000001 1100000 111001 111 11000 100010001
0 00010000110001 001001 01000001 1100000 111001 111 11000 100010001
=
01011010111101100101100101011100000110000011111100010010001
00001110011100000101110101010100000110000011111100010010001
00101001110000100101101101010100000110000011111100010010001
01110000011100000101111101011100000110000011111100010010001
000111110100001001011000110111000001100000111111000100100011
0111010001011010010111001101010000011000001111110001001000111
000011011111101001011000000000100001100000111111000100100011
00100101011110100100110000001010000110000011111100010010001
00100111001110100100101000001010000110000011111100010010001
=
01100110011001100100110011010100000100001111100001100010000
01110011000001100100100000000010000100001111100001100010000
00101001010001001100110000001010000100001111100001100010000
01010100101010101100101000001010000100001111100001100010000
01001000101111101100111000000010000100001111100001100010000
00001010011001001100100011011010000100001111100001100010000
01010000000110001100110011010010000100001111100001100010000
01011100001111001010100000000010000100001111100001100010000
00101001000001000010110000001010000100001111100001100010000
00001010111011100010101000001010000100001111100001100010000
=
00000000000000000010100001100110001110001110101001100110011
000000000000000000101100011011100011100011101010011001100110
0000000000000000001010100110111000111000111010100110011001100
00000000000000000010111001100110001110001110101001100110011
=
00011001000011000100101101001001000101000001100100100010000
01100111101010100100111101000001000101000001100100100010000
00100011110001010100100011000001000101000001100100100010000
00101000001010110100110011001001000101000001100100100010000
00001111101000010100100000101001000101000001100100100010000
";
        let analyzed:Vec<String> = vec![String::from("
")
        ];
        assert_eq!(analyze_buffer(LOG), analyzed);
    }

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
