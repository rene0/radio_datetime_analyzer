use radio_datetime_analyzer::transmitters;
use std::{env, fs};

fn main() {
    let mut cmd_args = env::args();
    let program_name = cmd_args.next();
    if cmd_args.len() != 2 {
        eprintln!("Usage: {} station_name logfile", program_name.unwrap());
        return;
    }
    let station_name = cmd_args.next().unwrap().to_lowercase();
    if station_name != "dcf77" && station_name != "msf" {
        eprintln!("station_name must be 'dcf77' or 'msf' but is '{station_name}'");
        return;
    }
    let filename = cmd_args.next().unwrap();
    let buffer = fs::read_to_string(&filename);
    if let Err(ref s_error) = buffer {
        eprintln!("Could not read file '{}' : {s_error}", &filename);
        return;
    }
    match station_name.as_str() {
        "dcf77" => {
            let res = transmitters::dcf77::analyze_buffer(&buffer.unwrap());
            for r in res {
                print!("{r}");
            }
        }
        "msf" => transmitters::msf::analyze_buffer(buffer.unwrap()),
        _ => {}
    }
}
