use dcf77_utils::DCF77Utils;
use npl_utils::NPLUtils;
use radio_datetime_utils::RadioDateTimeUtils;
use std::io;

pub mod dcf77;
pub mod npl;

pub fn analyze_rdt_buffer(station_name: String, buffer: io::Result<String>) {
    let mut dcf77 = DCF77Utils::default();
    let mut npl = NPLUtils::default();
    let mut npl_buffer = [' '; 62]; // ideally this 62 is npl::BIT_BUFFER_SIZE but that is private
    let buffer = buffer.unwrap();
    for c in buffer.chars() {
        if station_name == "dcf77" && !['0', '1', '_', '\n'].contains(&c) {
            continue;
        }
        if station_name == "npl" && !['0', '1', '2', '3', '4', '_', '\n'].contains(&c) {
            continue;
        }

        if station_name == "dcf77" {
            dcf77::append_bit(&mut dcf77, c);
            print!("{}", dcf77::str_bit(&dcf77, c));
        }
        if station_name == "npl" {
            npl::append_bits(&mut npl, c, &mut npl_buffer);
        }
        if c == '\n' {
            let rdt: RadioDateTimeUtils;
            let dst: Option<u8>;
            if station_name == "dcf77" {
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
            } else {
                print!("{}", npl::str_bits(&npl_buffer, npl.get_minute_length()));
                npl.decode_time();
                npl.force_new_minute();
                rdt = npl.get_radio_datetime();
                dst = rdt.get_dst();
                println!(
                    "first_minute={} second={} minute_length={}",
                    npl.get_first_minute(),
                    npl.get_second(),
                    npl.get_minute_length()
                );
            }
            display_datetime(
                &rdt,
                if station_name == "dcf77" {
                    dcf77::str_weekday(rdt.get_weekday())
                } else {
                    npl::str_weekday(rdt.get_weekday())
                },
                dst,
            );
            if station_name == "dcf77" {
                println!(
                    " [{}]",
                    dcf77::leap_second_info(rdt.get_leap_second(), dcf77.get_leap_second_is_one())
                );
                println!(
                    "Third-party buffer={}",
                    dcf77::str_hex(dcf77.get_third_party_buffer())
                );
                for parity in dcf77::str_parities(&dcf77) {
                    println!("{parity}")
                }
            }
            if station_name == "npl" {
                println!(" DUT1={}", npl::str_i8(npl.get_dut1()));
                if !npl.end_of_minute_marker_present(false) {
                    println!("End-of-minute marker absent");
                }
                for parity in npl::str_parities(&npl) {
                    println!("{parity}");
                }
            }
            display_jumps(&rdt);
            println!();
        }
        if station_name == "dcf77" {
            dcf77.increase_second();
        }
        if station_name == "npl" {
            npl.increase_second();
        }
    }
}
/// Return a string version of the given value with leading 0, truncated to two digits or ** for None.
///
/// # Arguments
/// * `value` - value to stringify
pub fn str_u8_02(value: Option<u8>) -> String {
    if let Some(s_value) = value {
        format!("{s_value:>02}")
    } else {
        String::from("**")
    }
}

/// Describe the dst parameter in plain English.
///
/// # Arguments
/// * `dst` - current state of daylight saving time
pub fn dst_info(dst: Option<u8>) -> String {
    let mut s = String::from("");
    if let Some(s_dst) = dst {
        if s_dst & radio_datetime_utils::DST_ANNOUNCED != 0 {
            s += "announced,";
        }
        if s_dst & radio_datetime_utils::DST_PROCESSED != 0 {
            s += "processed,";
        }
        if s_dst & radio_datetime_utils::DST_JUMP != 0 {
            s += "jump,";
        }
        if s_dst & radio_datetime_utils::DST_SUMMER != 0 {
            s += "summer";
        } else {
            s += "winter";
        }
    }
    s
}

/// Display the part of the date and time which is common to all stations.
///
/// # Arguments
/// * `rdt` - structure containing the currently decoded date/time
/// * `weekday` - name of the current weekday, in English
/// * `dst` - current state of daylight saving time
pub fn display_datetime(rdt: &RadioDateTimeUtils, weekday: String, dst: Option<u8>) {
    print!(
        "{}-{}-{} {} {}:{} [{}]",
        str_u8_02(rdt.get_year()),
        str_u8_02(rdt.get_month()),
        str_u8_02(rdt.get_day()),
        weekday,
        str_u8_02(rdt.get_hour()),
        str_u8_02(rdt.get_minute()),
        dst_info(dst)
    );
}

/// Display any unexpected jumps in plain English.
///
/// # Arguments
/// * `rdt` - structure containing the currently decoded date/time
pub fn display_jumps(rdt: &RadioDateTimeUtils) {
    if rdt.get_jump_year() {
        println!("Year jumped");
    }
    if rdt.get_jump_month() {
        println!("Month jumped");
    }
    if rdt.get_jump_day() {
        println!("Day-of-month jumped");
    }
    if rdt.get_jump_weekday() {
        println!("Day-of-week jumped");
    }
    if rdt.get_jump_hour() {
        println!("Hour jumped");
    }
    if rdt.get_jump_minute() {
        println!("Minute jumped");
    }
}
