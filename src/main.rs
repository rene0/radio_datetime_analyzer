use std::{env, fs};
use radio_datetime_analyzer::analyze_rdt_buffer;

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
    analyze_rdt_buffer(station_name, buffer);
}
