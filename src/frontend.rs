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
