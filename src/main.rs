use crate::frontend::{dcf77, npl};
use dcf77_utils::DCF77Utils;
use npl_utils::NPLUtils;
use radio_datetime_utils::RadioDateTimeUtils;
use std::{env, fs};

mod frontend;

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
        if station_name == "dcf77" && !['0', '1', '_', '\n'].contains(&c) {
            continue;
        }
        if station_name == "npl" && !['0', '1', '2', '3', '4', '_', '\n'].contains(&c) {
            continue;
        }

        let second;
        if station_name == "dcf77" {
            second = dcf77.get_second();
            if dcf77::is_space_bit(second) {
                print!(" ");
            }
            if c != '\n' {
                dcf77.set_current_bit(match c {
                    '0' => Some(false),
                    '1' => Some(true),
                    _ => None, // always '_' in this case, but match must be exhaustive
                });
            }
        } else if station_name == "npl" {
            second = npl.get_second();
            if npl::is_space_bit(second) {
                print!(" ");
            }
            if c != '\n' {
                npl.set_current_bit_a(match c {
                    '0' | '2' => Some(false),
                    '1' | '3' => Some(true),
                    _ => None, // '_' or '4' (the 500ms long BOM marker)
                });
                npl.set_current_bit_b(match c {
                    '0' | '1' => Some(false),
                    '2' | '3' => Some(true),
                    _ => None, // '_' or '4' (the 500ms long BOM marker)
                });
            }
        }
        print!("{}", c);
        if c == '\n' {
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
                frontend::str_u8_02(rdt.get_year()),
                frontend::str_u8_02(rdt.get_month()),
                frontend::str_u8_02(rdt.get_day()),
                if station_name == "dcf77" {
                    dcf77::str_weekday(rdt.get_weekday())
                } else {
                    npl::str_weekday(rdt.get_weekday())
                },
                frontend::str_u8_02(rdt.get_hour()),
                frontend::str_u8_02(rdt.get_minute()),
                frontend::dst_info(dst),
            );
            if station_name == "npl" {
                println!(" DUT1={}", npl::str_i8(npl.get_dut1()));
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
                    dcf77::leap_second_info(rdt.get_leap_second(), dcf77.get_leap_second_is_one())
                );
                println!(
                    "Third-party buffer={}",
                    dcf77::str_hex(dcf77.get_third_party_buffer()),
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
        if station_name == "npl" {
            npl.increase_second();
        } else {
            dcf77.increase_second();
        }
    }
}
