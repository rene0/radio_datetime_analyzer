use dcf77_utils::DCF77Utils;
use npl_utils::NPLUtils;
use radio_datetime_utils::RadioDateTimeUtils;
use std::{env, fs};

/// Return a string version of the given value with leading 0, truncated to two digits or ** for None.
fn str_u8_02(value: Option<u8>) -> String {
    if let Some(s_value) = value {
        format!("{:>02}", s_value)
    } else {
        String::from("**")
    }
}

/// Return a string representation of the given value or ? for None.
fn str_i8(value: Option<i8>) -> String {
    if let Some(s_value) = value {
        format!("{}", s_value)
    } else {
        String::from("?")
    }
}

/// Return a string version of the 16-bit decimal value, or 0x**** for None.
fn str_hex(value: Option<u16>) -> String {
    if let Some(s_value) = value {
        format!("{:>#04x}", s_value)
    } else {
        String::from("0x****")
    }
}

/// Describe the leap second parameters in plain English.
fn leap_second_info(leap_second: Option<u8>, is_one: Option<bool>) -> String {
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

/// Describe the dst parameter in plain English.
fn dst_info(dst: Option<u8>) -> String {
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

/// Return a textual representation of the weekday, Sunday-Saturday or ? for None.
fn str_weekday(station_name: &String, weekday: Option<u8>) -> String {
    String::from(match weekday {
        Some(0) => {
            if station_name == "npl" {
                "Sunday"
            } else {
                "?"
            }
        }
        Some(1) => "Monday",
        Some(2) => "Tuesday",
        Some(3) => "Wednesday",
        Some(4) => "Thursday",
        Some(5) => "Friday",
        Some(6) => "Saturday",
        Some(7) => {
            if station_name == "dcf77" {
                "Sunday"
            } else {
                "?"
            }
        }
        _ => "?",
    })
}

/// Determine if we should print a space before this bit (pair).
fn is_space_bit(station_name: &String, second: u8) -> bool {
    if station_name == "dcf77" {
        [1, 15, 16, 19, 20, 21, 28, 29, 35, 36, 42, 45, 50, 58, 59].contains(&second)
    } else {
        [1, 9, 17, 25, 30, 36, 39, 45, 52].contains(&second)
    }
}

fn main() {
    let station_name = env::args()
        .nth(1)
        .expect("Expected one log type to be given.")
        .to_lowercase();
    if station_name != "dcf77" && station_name != "npl" {
        eprintln!("Log type must be dcf77 or npl");
        return;
    }
    let filename = env::args()
        .nth(2)
        .expect("Expected one filename to analyze.");
    let mut dcf77 = DCF77Utils::default();
    let mut npl = NPLUtils::default();
    let buffer = fs::read_to_string(&filename);
    for c in buffer
        .unwrap_or_else(|_| panic!("Could not read '{}' !", &filename))
        .chars()
    {
        if (station_name == "dcf77" && !['0', '1', '_', '\n'].contains(&c))
            || !['0', '1', '2', '3', '4', '_', '\n'].contains(&c)
        {
            continue;
        }
        let second = if station_name == "dcf77" {
            dcf77.get_second()
        } else {
            npl.get_second()
        };
        if is_space_bit(&station_name, second) {
            print!(" ");
        }
        print!("{}", c);
        match c {
            '0' => {
                if station_name == "npl" {
                    npl.set_current_bit_a(Some(false));
                    npl.set_current_bit_b(Some(false));
                } else {
                    dcf77.set_current_bit(Some(false));
                }
            }
            '1' => {
                if station_name == "npl" {
                    npl.set_current_bit_a(Some(true));
                    npl.set_current_bit_b(Some(false));
                } else {
                    dcf77.set_current_bit(Some(true));
                }
            }
            '2' => {
                if station_name == "npl" {
                    npl.set_current_bit_a(Some(false));
                    npl.set_current_bit_b(Some(true));
                }
            }
            '3' => {
                if station_name == "npl" {
                    npl.set_current_bit_a(Some(true));
                    npl.set_current_bit_b(Some(true));
                }
            }
            '4' => {
                // This is the 500ms long begin-of-minute marker.
                if station_name == "npl" {
                    npl.set_current_bit_a(None);
                    npl.set_current_bit_b(None);
                }
            }
            '_' => {
                // Bit received improperly.
                if station_name == "npl" {
                    npl.set_current_bit_a(None);
                    npl.set_current_bit_b(None);
                } else {
                    dcf77.set_current_bit(None);
                }
            }
            '\n' => {
                let rdt: RadioDateTimeUtils;
                let dst: Option<u8>;
                if station_name == "npl" {
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
                } else {
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
                }
                print!(
                    "{}-{}-{} {} {}:{} [{}]",
                    str_u8_02(rdt.get_year()),
                    str_u8_02(rdt.get_month()),
                    str_u8_02(rdt.get_day()),
                    str_weekday(&station_name, rdt.get_weekday()),
                    str_u8_02(rdt.get_hour()),
                    str_u8_02(rdt.get_minute()),
                    dst_info(dst),
                );
                if station_name == "npl" {
                    println!(" DUT1={}", str_i8(npl.get_dut1()));
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
                } else {
                    println!(
                        " [{}]",
                        leap_second_info(rdt.get_leap_second(), dcf77.get_leap_second_is_one())
                    );
                    println!(
                        "Third-party buffer={}",
                        str_hex(dcf77.get_third_party_buffer()),
                    );
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
                println!();
            }
            _ => {}
        }
        if station_name == "npl" {
            npl.increase_second();
        } else {
            dcf77.increase_second();
        }
    }
}
