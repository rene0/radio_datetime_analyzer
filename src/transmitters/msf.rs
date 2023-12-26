use crate::{str_datetime, str_jumps, str_weekday};
use msf60_utils::MSFUtils;
use std::cmp::Ordering;

/// Analyze a MSF logfile, return the input with the results interleaved.
///
/// # Arguments
/// `buffer` - the buffer containing the MSF logfile
pub fn analyze_buffer(buffer: &str) -> Vec<String> {
    let mut msf = MSFUtils::default();
    let mut res = Vec::new();
    let mut msf_buffer = [' '; msf60_utils::BIT_BUFFER_SIZE];
    for c in buffer.chars() {
        if !['0', '1', '2', '3', '4', '_', '\n'].contains(&c) {
            continue;
        }
        append_bits(&mut msf, c, &mut msf_buffer); // does nothing with newline except adding it to msf_buffer
        let last_second = msf.get_second();
        let wanted_len = msf.get_minute_length();
        let eom = msf.end_of_minute_marker_present();
        if c == '\n' {
            if last_second + 1 == wanted_len {
                res.push(str_bits(&msf_buffer, wanted_len));
                msf.decode_time(); // does not affect msf.get_minute_length()
                let rdt = msf.get_radio_datetime();
                res.push(format!(
                    "first_minute={} seconds={} minute_length={}\n",
                    msf.get_first_minute(),
                    last_second + 1,
                    wanted_len
                ));
                res.push(format!(
                    "{} DUT1={}\n",
                    str_datetime(&rdt, str_weekday(rdt.get_weekday(), 0), rdt.get_dst()),
                    str_i8(msf.get_dut1())
                ));
                if !eom {
                    res.push(String::from("End-of-minute marker absent\n"));
                }
                for parity in str_parities(&msf) {
                    res.push(format!("{parity}\n"));
                }
                for jump in str_jumps(&rdt) {
                    res.push(format!("{jump}\n"));
                }
            } else {
                res.push(format!(
                    "Minute is {last_second} seconds instead of {wanted_len} seconds long\n"
                ));
            }
            msf.force_new_minute();
            res.push(String::from("\n"));
        }
        if !eom {
            msf.increase_second();
        }
    }
    res
}

/// Append the given bit pair to the current MSF structure and to the given buffer for later
/// displaying.
///
/// # Arguments
/// * `msf` - the structure to append the bit pair to
/// * `c` - the bit pair to add. The newline is there for showing a new minute, it is a not
///         a bit pair in itself.
/// * `buffer` - buffer storing the bits for later displaying
fn append_bits(msf: &mut MSFUtils, c: char, buffer: &mut [char]) {
    if c != '\n' {
        if c == '4' {
            msf.force_past_new_minute();
        } else {
            // 4 is the 500ms long BOM marker
            msf.set_current_bit_a(match c {
                '0' | '2' => Some(false),
                '1' | '3' => Some(true),
                '_' => None,
                _ => panic!("msf::append_bits(): impossible character '{c}'"),
            });
            msf.set_current_bit_b(match c {
                '0' | '1' => Some(false),
                '2' | '3' => Some(true),
                '_' => None,
                _ => panic!("msf::append_bits(): impossible character '{c}'"),
            });
        }
    }
    buffer[msf.get_second() as usize + (c == '\n') as usize] = c;
}

/// Return a string version of the all the bit pairs (or the EOM newline) in this minute.
/// Each bit pair is optionally prefixed by a space.
///
/// # Arguments
/// * `buffer` - the buffer to stringify
/// * `minute_length` - the number of bit pairs in this minute
fn str_bits(buffer: &[char], minute_length: u8) -> String {
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

/// Return a string representation of the given value or ? for None.
///
/// # Arguments
/// * `value` - value to stringify
fn str_i8(value: Option<i8>) -> String {
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
fn str_parities(msf: &MSFUtils) -> Vec<&str> {
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
    fn test_analyze_logfile() {
        const LOG: &str = include_str!("sample_msf.log");
        let analyzed = vec![
            String::from("4 00000000 22000000 00100000 00011 101000 110 100011 1011001 01133110\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-28 Saturday 23:59 [winter] DUT1=-2\n"),
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000000 0000000 03113310\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"), // bit 53b not yet trusted,
            String::from("20-03-29 Sunday 00:00 [winter] DUT1=-2\n"), // radio_datetime_utils insists on minute > 0
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000000 0000001 03113110\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 00:01 [announced,winter] DUT1=-2\n"),
            String::from("\n"),
            String::from("Minute is 0 seconds instead of 60 seconds long\n"), // test empty(?) minute
            String::from("\n"),
            String::from("Minute is 3 seconds instead of 60 seconds long\n"), // test incomplete minute
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000000 0000100 03113110\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 00:04 [announced,winter] DUT1=-2\n"),
            String::from("Minute jumped\n"), // 00:01 -> 00:04
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000000 0000101 03112310\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 00:05 [announced,winter] DUT1=-2\n"),
            String::from("End-of-minute marker absent\n"), // note the '2' in the last word
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000000 0000110 03113310\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 00:06 [announced,winter] DUT1=-2\n"),
            String::from("\n"),
            String::from("4 00000000 2200000 00100000 00011 101001 000 000000 0000111 03113110\n"),
            String::from("first_minute=false seconds=59 minute_length=59\n"), // artificially remove bit 16
            String::from("20-03-29 Sunday 00:07 [announced,winter] DUT1=-2\n"),
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000000 0001000 03113110\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 00:08 [announced,winter] DUT1=-2\n"),
            String::from("\n"),
            String::from(
                "4 00000000 220000000 00100000 00011 101001 000 000000 0001001 03113310\n",
            ),
            String::from("first_minute=false seconds=61 minute_length=61\n"), // artificially add a second bit 16
            String::from("20-03-29 Sunday 00:09 [announced,winter] DUT1=-2\n"),
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000000 0010000 03113110\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 00:10 [announced,winter] DUT1=-2\n"),
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000000 1011000 03113110\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 00:58 [announced,winter] DUT1=-2\n"),
            String::from("Minute jumped\n"), // 00:10 -> 00:58
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000000 1011001 03113310\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 00:59 [announced,winter] DUT1=-2\n"),
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000010 0000000 03113130\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 02:00 [processed,summer] DUT1=-2\n"), // correct DST transition
            String::from("\n"),
            String::from("4 00000000 22000000 00100000 00011 101001 000 000010 0000001 01113330\n"),
            String::from("first_minute=false seconds=60 minute_length=60\n"),
            String::from("20-03-29 Sunday 02:01 [summer] DUT1=-2\n"),
            String::from("\n"),
        ];
        assert_eq!(analyze_buffer(LOG), analyzed);
    }

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
        assert_eq!(buffer[0..6], ['0', '1', '_', '2', ' ', '3']); // space because \n is not inserted
        msf.increase_second();
        // a '4' calls force_past_new_minute() which resets the second counter to 0
        append_bits(&mut msf, '4', &mut buffer); // BOM
        assert_eq!(msf.get_current_bit_a(), Some(true));
        assert_eq!(msf.get_current_bit_a(), Some(true));
        assert_eq!(buffer[0..6], ['4', '1', '_', '2', ' ', '3']); // space because \n is not inserted
    }

    #[test]
    fn test_str_bits_59() {
        const BUFFER: [char; 60] = [
            '4', // 0
            '0', '0', '0', '0', '0', '0', '0', '0', // 1-8
            '0', '0', '0', '0', '0', '0', '0', // 9-15
            '0', '0', '1', '0', '0', '0', '1', '1', // 16-23
            '1', '0', '0', '0', '0', // 24-28
            '1', '0', '0', '0', '1', '1', // 29-34
            '0', '0', '1', // 35-37
            '1', '0', '0', '0', '1', '1', // 38-43
            '0', '0', '1', '0', '1', '1', '0', // 44-50
            '0', '1', '1', '3', '1', '3', '3', '0', // 51-58
            '\n',
        ];
        const WANTED: &str =
            "4 00000000 0000000 00100011 10000 100011 001 100011 0010110 01131330\n";
        assert_eq!(str_bits(&BUFFER, 59), WANTED);
        assert_eq!(str_bits(&[BUFFER, BUFFER].concat(), 59), WANTED);
    }

    #[test]
    fn test_str_bits_60() {
        const BUFFER: [char; 61] = [
            '4', // 0
            '0', '0', '0', '0', '0', '0', '0', '0', // 1-8
            '0', '0', '0', '0', '0', '0', '0', '0', // 9-16
            '0', '0', '1', '0', '0', '0', '1', '1', // 17-24
            '1', '0', '0', '0', '0', // 25-29
            '1', '0', '0', '0', '1', '1', // 30-35
            '0', '0', '1', // 36-38
            '1', '0', '0', '0', '1', '1', // 39-44
            '0', '0', '1', '0', '1', '1', '0', // 45-51
            '0', '1', '1', '3', '1', '3', '3', '0', // 52-59
            '\n',
        ];
        const WANTED: &str =
            "4 00000000 00000000 00100011 10000 100011 001 100011 0010110 01131330\n";
        assert_eq!(str_bits(&BUFFER, 60), WANTED);
        assert_eq!(str_bits(&[BUFFER, BUFFER].concat(), 60), WANTED);
    }

    #[test]
    fn test_str_bits_61() {
        const BUFFER: [char; 62] = [
            '4', // 0
            '0', '0', '0', '0', '0', '0', '0', '0', // 1-8
            '0', '0', '0', '0', '0', '0', '0', '0', // 9-16
            '0', // 17
            '0', '0', '1', '0', '0', '0', '1', '1', // 18-25
            '1', '0', '0', '0', '0', // 26-30
            '1', '0', '0', '0', '1', '1', // 31-38
            '0', '0', '1', // 37-39
            '1', '0', '0', '0', '1', '1', // 40-45
            '0', '0', '1', '0', '1', '1', '0', // 46-52
            '0', '1', '1', '3', '1', '3', '3', '0', // 53-60
            '\n',
        ];
        const WANTED: &str =
            "4 00000000 000000000 00100011 10000 100011 001 100011 0010110 01131330\n";
        assert_eq!(str_bits(&BUFFER, 61), WANTED);
        assert_eq!(str_bits(&[BUFFER, BUFFER].concat(), 61), WANTED);
    }
}
