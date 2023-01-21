use radio_datetime_utils::RadioDateTimeUtils;

pub mod dcf77;
pub mod npl;

/// Return a string version of the given value with leading 0, truncated to two digits or ** for None.
pub fn str_u8_02(value: Option<u8>) -> String {
    if let Some(s_value) = value {
        format!("{:>02}", s_value)
    } else {
        String::from("**")
    }
}

/// Describe the dst parameter in plain English.
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
