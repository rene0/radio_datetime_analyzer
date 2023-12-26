use crate::{str_datetime, str_jumps, str_weekday};
use dcf77_utils::{DCF77Utils, DecodeType};

/// Analyze a DCF77 logfile, return the input with the results interleaved.
///
/// # Arguments
/// `buffer` - the buffer containing the DCF77 logfile
pub fn analyze_buffer(buffer: &str) -> Vec<String> {
    let mut dcf77 = DCF77Utils::new(DecodeType::LogFile);
    let mut res = Vec::new();
    let mut bits = String::from("");
    for c in buffer.chars() {
        if !['0', '1', '_', '\n'].contains(&c) {
            continue;
        }
        append_bit(&mut dcf77, c); // does nothing with newline
        bits.push_str(&str_bit(&dcf77, c));
        let actual_len = dcf77.get_second() + 1;
        let wanted_len = dcf77.get_next_minute_length();
        if c == '\n' {
            res.push(bits.clone());
            bits.clear();
            if actual_len == wanted_len {
                dcf77.decode_time();
                let rdt = dcf77.get_radio_datetime();
                res.push(format!(
                    "first_minute={} seconds={} this_minute_length={} next_minute_length={}\n",
                    dcf77.get_first_minute(),
                    actual_len,
                    dcf77.get_this_minute_length(),
                    dcf77.get_next_minute_length()
                ));
                res.push(format!(
                    "{} [{}] [{}]\n",
                    str_datetime(&rdt, str_weekday(rdt.get_weekday(), 7), rdt.get_dst()),
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
            } else {
                res.push(format!(
                    "Minute is {actual_len} seconds instead of {wanted_len} seconds long\n"
                ));
            }
            dcf77.force_new_minute(); // (this, next) = (next, new_next)
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
    if c != '\n'
        && [1, 15, 16, 19, 20, 21, 28, 29, 35, 36, 42, 45, 50, 58, 59].contains(&dcf77.get_second())
    {
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
    use dcf77_utils::DecodeType;
    use radio_datetime_utils::{LEAP_ANNOUNCED, LEAP_MISSING, LEAP_PROCESSED};

    #[test]
    fn test_analyze_logfile() {
        const LOG: &str = include_str!("test/sample_dcf77.log");
        let analyzed = vec![
            String::from(
                "0 00000000000000 0 001 0 1 0001101 1 110001 1 100011 101 01001 10011001 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("99-12-31 Friday 23:58 [winter] [] []\n"),
            String::from("Third-party buffer=0x0000\n"),
            String::from("\n"),
            String::from(
                "0 00000000000000 0 001 0 1 1001101 0 110001 1 100011 101 01001 10011001 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("99-12-31 Friday 23:59 [winter] [] []\n"),
            String::from("Third-party buffer=0x0000\n"),
            String::from("\n"),
            String::from(
                "0 00000000000000 0 001 0 1 0000000 0 000000 0 100000 011 10000 00000000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("00-01-01 Saturday 00:00 [winter] [] []\n"), // y2k OK
            String::from("Third-party buffer=0x0000\n"),
            String::from("\n"),
            String::from(
                "0 00000000000000 0 001 0 1 1000000 1 000000 0 100000 011 10000 00000000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("00-01-01 Saturday 00:01 [winter] [] []\n"),
            String::from("Third-party buffer=0x0000\n"),
            String::from("\n"),
            String::from("\n"),
            String::from("Minute is 1 seconds instead of 60 seconds long\n"),
            String::from("\n"),
            String::from(
                "0 00110110100111 0 010 0 1 1010110 0 100010 0 100110 110 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:35 [jump,winter] [] []\n"), // "unexpected" DST jump as there was we skipped the announcement.
            String::from("Third-party buffer=0x396c\n"),
            String::from("Year jumped\n"),
            String::from("Month jumped\n"),
            String::from("Day-of-month jumped\n"),
            String::from("Day-of-week jumped\n"),
            String::from("Hour jumped\n"),
            String::from("Minute jumped\n"),
            String::from("\n"),
            String::from(
                "0 01110010011110 0 010 0 1 0110110 0 100010 0 100110 110 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:36 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x1e4e\n"),
            String::from("\n"),
            String::from(
                "0 00111100101011 0 010 0 1 1110110 _ ______ _ ______ ___ _____ ________ _\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:37 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x353c\n"),
            String::from("Minute parity undetermined\n"), // missing data to calculate parities
            String::from("Hour parity undetermined\n"),   // all _ bits are None
            String::from("Date parity undetermined\n"),
            String::from("\n"),
            String::from(
                "0 00000001110001 0 010 0 1 1010001 1 100010 0 100110 110 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:45 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x2380\n"),
            String::from("Minute jumped\n"), // signal restored
            String::from("\n"),
            String::from(
                "0 01010100000011 0 010 0 1 0110001 1 100010 0 100110 110 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:46 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x302a\n"),
            String::from("\n"),
            String::from(
                "0 00011000100101 0 010 0 1 1110001 0 100010 0 100110 110 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:47 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x2918\n"),
            String::from("\n"),
            String::from(
                "0 00011011000001 0 010 0 1 0001001 0 100010 0 100110 110 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:48 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x20d8\n"),
            String::from("\n"),
            String::from(
                "0 01000100010010 0 010 0 1 _______ _ ______ _ ______ ___ _____ ________ _\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:49 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x1222\n"),
            String::from("Minute parity undetermined\n"), // signal lost (again)
            String::from("Hour parity undetermined\n"),
            String::from("Date parity undetermined\n"),
            String::from("\n"),
            String::from(
                "0 00101000101010 0 010 0 1 1110101 1 100010 0 100110 110 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:57 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x1514\n"),
            String::from("Minute jumped\n"), // signal restored (again)
            String::from("\n"),
            String::from(
                "0 01010100111111 0 010 0 1 0001101 1 100010 0 100110 110 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-19 Wednesday 11:58 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x3f2a\n"),
            String::from("\n"),
            String::from("\n"),
            String::from("Minute is 1 seconds instead of 60 seconds long\n"),
            String::from("\n"),
            String::from(
                "0 00110100000101 0 001 0 1 1001101 0 000000 0 111001 111 11000 10001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-03-27 Sunday 00:59 [winter] [] []\n"),
            String::from("Third-party buffer=0x282c\n"),
            String::from("Month jumped\n"),
            String::from("Day-of-month jumped\n"),
            String::from("Day-of-week jumped\n"),
            String::from("Hour jumped\n"),
            String::from("\n"),
            String::from(
                "0 10010000101011 0 001 0 1 0000000 0 100000 1 111001 111 11000 10001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-03-27 Sunday 01:00 [winter] [] []\n"),
            String::from("Third-party buffer=0x3509\n"),
            String::from("\n"),
            String::from(
                "0 00010110001111 0 101 0 1 1000000 1 100000 1 111001 111 11000 10001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-03-27 Sunday 01:01 [announced,winter] [] []\n"), // see bit 16
            String::from("Third-party buffer=0x3c68\n"),
            String::from("\n"),
            String::from(
                "0 01010000000001 0 101 0 1 0001101 1 100000 1 111001 111 11000 10001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-03-27 Sunday 01:58 [announced,winter] [] []\n"),
            String::from("Third-party buffer=0x200a\n"),
            String::from("Minute jumped\n"), // skip boring stuff...
            String::from("\n"),
            String::from(
                "0 00010011100011 0 101 0 1 1001101 0 100000 1 111001 111 11000 10001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-03-27 Sunday 01:59 [announced,winter] [] []\n"),
            String::from("Third-party buffer=0x31c8\n"),
            String::from("\n"),
            String::from(
                "0 01100111010010 0 110 0 1 0000000 0 110000 0 111001 111 11000 10001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-03-27 Sunday 03:00 [processed,summer] [] []\n"), // DST switch OK
            String::from("Third-party buffer=0x12e6\n"),
            String::from("\n"),
            String::from(
                "0 00000010100111 0 010 0 1 1000000 1 110000 0 111001 111 11000 10001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-03-27 Sunday 03:01 [summer] [] []\n"),
            String::from("Third-party buffer=0x3940\n"),
            String::from("\n"),
            String::from(
                "0 00010000110001 0 010 0 1 0100000 1 110000 0 111001 111 11000 10001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-03-27 Sunday 03:02 [summer] [] []\n"),
            String::from("Third-party buffer=0x2308\n"),
            String::from("\n"),
            String::from("\n"),
            String::from("Minute is 1 seconds instead of 60 seconds long\n"),
            String::from("\n"),
            String::from(
                "0 10110101111011 0 010 1 1 0010101 1 100000 1 100000 111 11100 01001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("12-07-01 Sunday 01:54 [summer] [] []\n"), // not trusting bit 19 yet...
            String::from("Third-party buffer=0x37ad\n"),
            String::from("Year jumped\n"),
            String::from("Month jumped\n"),
            String::from("Day-of-month jumped\n"),
            String::from("Hour jumped\n"),
            String::from("Minute jumped\n"),
            String::from("\n"),
            String::from(
                "0 00011100111000 0 010 1 1 1010101 0 100000 1 100000 111 11100 01001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("12-07-01 Sunday 01:55 [summer] [announced] []\n"), // leap second coming...
            String::from("Third-party buffer=0x0738\n"),
            String::from("\n"),
            String::from(
                "0 01010011100001 0 010 1 1 0110101 0 100000 1 100000 111 11100 01001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("12-07-01 Sunday 01:56 [summer] [announced] []\n"),
            String::from("Third-party buffer=0x21ca\n"),
            String::from("\n"),
            String::from(
                "0 11100000111000 0 010 1 1 1110101 1 100000 1 100000 111 11100 01001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("12-07-01 Sunday 01:57 [summer] [announced] []\n"),
            String::from("Third-party buffer=0x0707\n"),
            String::from("\n"),
            String::from(
                "0 00111110100001 0 010 1 1 0001101 1 100000 1 100000 111 11100 01001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("12-07-01 Sunday 01:58 [summer] [announced] []\n"),
            String::from("Third-party buffer=0x217c\n"),
            String::from("\n"),
            String::from(
                "0 11101000101101 0 010 1 1 1001101 0 100000 1 100000 111 11100 01001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=61\n",
            ),
            String::from("12-07-01 Sunday 01:59 [summer] [announced] []\n"),
            String::from("Third-party buffer=0x2d17\n"),
            String::from("\n"),
            String::from(
                "0 00011011111101 0 010 1 1 0000000 0 010000 1 100000 111 11100 01001000 1 1\n",
            ),
            String::from(
                "first_minute=false seconds=61 this_minute_length=61 next_minute_length=60\n",
            ),
            String::from("12-07-01 Sunday 02:00 [summer] [processed,one] []\n"), // leap second OK, artificially set to 1
            String::from("Third-party buffer=0x2fd8\n"),
            String::from("\n"),
            String::from(
                "0 01001010111101 0 010 0 1 1000000 1 010000 1 100000 111 11100 01001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("12-07-01 Sunday 02:01 [summer] [] []\n"),
            String::from("Third-party buffer=0x2f52\n"),
            String::from("\n"),
            String::from(
                "0 01001110011101 0 010 0 1 0100000 1 010000 1 100000 111 11100 01001000 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("12-07-01 Sunday 02:02 [summer] [] []\n"),
            String::from("Third-party buffer=0x2e72\n"),
            String::from("\n"),
            String::from("\n"),
            String::from("Minute is 1 seconds instead of 60 seconds long\n"),
            String::from("\n"),
            String::from(
                "0 11001100110011 0 010 0 1 1001101 0 100000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 01:59 [summer] [] []\n"),
            String::from("Third-party buffer=0x3333\n"),
            String::from("Year jumped\n"),
            String::from("Month jumped\n"),
            String::from("Day-of-month jumped\n"),
            String::from("Hour jumped\n"),
            String::from("Minute jumped\n"),
            String::from("\n"),
            String::from(
                "0 11100110000011 0 010 0 1 0000000 0 010000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 02:00 [summer] [] []\n"),
            String::from("Third-party buffer=0x3067\n"),
            String::from("\n"),
            String::from(
                "0 01010010100010 0 110 0 1 1000000 1 010000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 02:01 [announced,summer] [] []\n"), // change to normal time coming up...
            String::from("Third-party buffer=0x114a\n"),
            String::from("\n"),
            String::from(
                "0 10101001010101 0 110 0 1 0100000 1 010000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 02:02 [announced,summer] [] []\n"),
            String::from("Third-party buffer=0x2a95\n"),
            String::from("\n"),
            String::from(
                "0 10010001011111 0 110 0 1 1100000 0 010000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 02:03 [announced,summer] [] []\n"),
            String::from("Third-party buffer=0x3e89\n"),
            String::from("\n"),
            String::from(
                "0 00010100110010 0 110 0 1 0001101 1 010000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 02:58 [announced,summer] [] []\n"),
            String::from("Third-party buffer=0x1328\n"),
            String::from("Minute jumped\n"), // skip some time...
            String::from("\n"),
            String::from(
                "0 10100000001100 0 110 0 1 1001101 0 010000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 02:59 [announced,summer] [] []\n"),
            String::from("Third-party buffer=0x0c05\n"),
            String::from("\n"),
            String::from(
                "0 10111000011110 0 101 0 1 0000000 0 010000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 02:00 [processed,winter] [] []\n"), // change to normal time OK
            String::from("Third-party buffer=0x1e1d\n"),
            String::from("\n"),
            String::from(
                "0 01010010000010 0 001 0 1 1000000 1 010000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 02:01 [winter] [] []\n"),
            String::from("Third-party buffer=0x104a\n"),
            String::from("\n"),
            String::from(
                "0 00010101110111 0 001 0 1 0100000 1 010000 1 000011 111 00001 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-10-30 Sunday 02:02 [winter] [] []\n"),
            String::from("Third-party buffer=0x3ba8\n"),
            String::from("\n"),
            String::from("\n"),
            String::from("Minute is 1 seconds instead of 60 seconds long\n"),
            String::from("\n"),
            String::from(
                "0 00000000000000 0 001 0 1 0000110 0 110001 1 100011 101 01001 10011001 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("99-12-31 Friday 23:30 [winter] [] []\n"),
            String::from("Third-party buffer=0x0000\n"),
            String::from("Year jumped\n"),
            String::from("Month jumped\n"),
            String::from("Day-of-month jumped\n"),
            String::from("Day-of-week jumped\n"),
            String::from("Hour jumped\n"),
            String::from("Minute jumped\n"),
            String::from("\n"),
            String::from(
                "0 00000000000000 0 001 0 1 1000110 1 110001 1 100011 101 01001 10011001 1 0\n",
            ),
            String::from("Minute is 1 seconds instead of 60 seconds long\n"), // OK, 61 bits
            String::from("\n"),
            String::from(
                "0 00000000000000 0 001 0 1 0100110 1 110001 1 100011 101 01001 10011001 1 00\n",
            ),
            String::from("Minute is 2 seconds instead of 60 seconds long\n"), // 62 mod 60 == 2
            String::from("\n"),
            String::from(
                "0 00000000000000 0 001 0 1 1100110 0 110001 1 100011 101 01001 10011001 1\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("99-12-31 Friday 23:33 [winter] [] []\n"),
            String::from("Third-party buffer=0x0000\n"),
            String::from("Minute jumped\n"), // not really, but we lost track
            String::from("\n"),
            String::from("\n"),
            String::from("Minute is 1 seconds instead of 60 seconds long\n"),
            String::from("\n"),
            String::from(
                "0 00110010000110 0 010 0 1 0110100 1 001000 1 010000 011 00100 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-04-02 Saturday 04:16 [jump,winter] [] []\n"), // unannounced DST change
            String::from("Third-party buffer=0x184c\n"),
            String::from("Year jumped\n"),
            String::from("Month jumped\n"),
            String::from("Day-of-month jumped\n"),
            String::from("Day-of-week jumped\n"),
            String::from("Hour jumped\n"),
            String::from("Minute jumped\n"),
            String::from("\n"),
            String::from(
                "0 11001111010101 0 010 0 1 1110100 0 001000 1 010000 011 00100 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-04-02 Saturday 04:17 [jump,winter] [] []\n"),
            String::from("Third-party buffer=0x2af3\n"),
            String::from("\n"),
            String::from(
                "0 01000111100010 1 010 0 1 0001100 0 001000 1 010000 011 00100 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-04-02 Saturday 04:18 [jump,winter] [] [call]\n"), // bit 15 set!
            String::from("Third-party buffer=0x11e2\n"),
            String::from("\n"),
            String::from(
                "0 01010000010101 1 010 0 1 1001100 1 001000 1 010000 011 00100 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-04-02 Saturday 04:19 [jump,winter] [] [call]\n"),
            String::from("Third-party buffer=0x2a0a\n"),
            String::from("\n"),
            String::from(
                "0 00011111010000 1 010 0 1 0000010 1 001000 1 010000 011 00100 10001000 0\n",
            ),
            String::from(
                "first_minute=false seconds=60 this_minute_length=60 next_minute_length=60\n",
            ),
            String::from("11-04-02 Saturday 04:20 [jump,winter] [] [call]\n"),
            String::from("Third-party buffer=0x02f8\n"),
            String::from("\n"),
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
        let mut dcf77 = DCF77Utils::new(DecodeType::LogFile);
        append_bit(&mut dcf77, '!');
    }

    #[test]
    fn test_append_bits_bunch() {
        let mut dcf77 = DCF77Utils::new(DecodeType::LogFile);
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
