use crate::frontend::{dcf77, npl};
use dcf77_utils::DCF77Utils;
use npl_utils::NPLUtils;
use radio_datetime_utils::RadioDateTimeUtils;
use std::{env, fs};

mod frontend;

fn main() {
    let mut cmd_args = env::args();
    let program_name = cmd_args.next();
    if cmd_args.len() != 2 {
        eprintln!("Usage: {} logtype logfile", program_name.unwrap());
        return;
    }
    let station_name = cmd_args.next().unwrap().to_lowercase();
    if station_name != "dcf77" && station_name != "npl" {
        eprintln!("logtype must be 'dcf77' or 'npl' but is '{station_name}'");
        return;
    }
    let filename = cmd_args.next().unwrap();
    let buffer = fs::read_to_string(&filename);
    if let Err(ref s_error) = buffer {
        eprintln!("Could not read file '{}' : {s_error}", &filename);
        return;
    }
    let mut dcf77 = DCF77Utils::default();
    let mut npl = NPLUtils::default();
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
            dcf77::display_bit(&dcf77, c);
        }
        if station_name == "npl" {
            npl::append_bits(&mut npl, c);
            npl::display_bits(&npl, c);
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
            frontend::display_datetime(
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
                dcf77::display_parities(&dcf77);
            }
            if station_name == "npl" {
                println!(" DUT1={}", npl::str_i8(npl.get_dut1()));
                if !npl.end_of_minute_marker_present(false) {
                    println!("End-of-minute marker absent");
                }
                npl::display_parities(&npl);
            }
            frontend::display_jumps(&rdt);
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
