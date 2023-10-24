use radio_datetime_utils::RadioDateTimeUtils;

pub mod transmitters;

/// Return a string version of the given value with leading 0, truncated to two digits or ** for None.
///
/// # Arguments
/// * `value` - value to stringify
fn str_u8_02(value: Option<u8>) -> String {
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
fn dst_info(dst: Option<u8>) -> String {
    // DST_ANNOUNCED is mutually exclusive with DST_PROCESSED
    // see radio_datetime_utils::set_dst()
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

/// Return the part of the date and time which is common to all stations.
///
/// # Arguments
/// * `rdt` - structure containing the currently decoded date/time
/// * `weekday` - name of the current weekday, in English
/// * `dst` - current state of daylight saving time
fn str_datetime(rdt: &RadioDateTimeUtils, weekday: String, dst: Option<u8>) -> String {
    format!(
        "{}-{}-{} {} {}:{} [{}]",
        str_u8_02(rdt.get_year()),
        str_u8_02(rdt.get_month()),
        str_u8_02(rdt.get_day()),
        weekday,
        str_u8_02(rdt.get_hour()),
        str_u8_02(rdt.get_minute()),
        dst_info(dst)
    )
}

/// Return a vector of any unexpected jumps in plain English.
///
/// # Arguments
/// * `rdt` - structure containing the currently decoded date/time
fn str_jumps(rdt: &RadioDateTimeUtils) -> Vec<&str> {
    let mut jumps = Vec::new();
    if rdt.get_jump_year() {
        jumps.push("Year jumped");
    }
    if rdt.get_jump_month() {
        jumps.push("Month jumped");
    }
    if rdt.get_jump_day() {
        jumps.push("Day-of-month jumped");
    }
    if rdt.get_jump_weekday() {
        jumps.push("Day-of-week jumped");
    }
    if rdt.get_jump_hour() {
        jumps.push("Hour jumped");
    }
    if rdt.get_jump_minute() {
        jumps.push("Minute jumped");
    }
    jumps
}

#[cfg(test)]
mod tests {
    use super::*;

    const DST_EMPTY: &str = "";
    const DST_SUMMER: &str = "summer";
    const DST_WINTER: &str = "winter";
    const DST_ANN_SUMMER: &str = "announced,summer";
    const DST_ANN_WINTER: &str = "announced,winter";
    const DST_PROC_SUMMER: &str = "processed,summer";
    const DST_PROC_WINTER: &str = "processed,winter";
    const DST_JUMP_SUMMER: &str = "jump,summer";
    const DST_JUMP_WINTER: &str = "jump,winter";
    const DST_ANN_JUMP_SUMMER: &str = "announced,jump,summer";
    const DST_ANN_JUMP_WINTER: &str = "announced,jump,winter";
    const DST_PROC_JUMP_SUMMER: &str = "processed,jump,summer";
    const DST_PROC_JUMP_WINTER: &str = "processed,jump,winter";

    #[test]
    fn test_dst_none() {
        assert_eq!(dst_info(None), DST_EMPTY);
    }
    #[test]
    fn test_dst_summer() {
        assert_eq!(dst_info(Some(radio_datetime_utils::DST_SUMMER)), DST_SUMMER);
    }
    #[test]
    fn test_dst_winter() {
        assert_eq!(dst_info(Some(0)), DST_WINTER);
    }
    #[test]
    fn test_dst_ann_summer() {
        assert_eq!(
            dst_info(Some(
                radio_datetime_utils::DST_ANNOUNCED | radio_datetime_utils::DST_SUMMER
            )),
            DST_ANN_SUMMER
        );
    }
    #[test]
    fn test_dst_ann_winter() {
        assert_eq!(
            dst_info(Some(radio_datetime_utils::DST_ANNOUNCED)),
            DST_ANN_WINTER
        );
    }
    #[test]
    fn test_dst_proc_summer() {
        assert_eq!(
            dst_info(Some(
                radio_datetime_utils::DST_PROCESSED | radio_datetime_utils::DST_SUMMER
            )),
            DST_PROC_SUMMER
        );
    }
    #[test]
    fn test_dst_proc_winter() {
        assert_eq!(
            dst_info(Some(radio_datetime_utils::DST_PROCESSED)),
            DST_PROC_WINTER
        );
    }
    #[test]
    fn test_dst_jump_summer() {
        assert_eq!(
            dst_info(Some(
                radio_datetime_utils::DST_JUMP | radio_datetime_utils::DST_SUMMER
            )),
            DST_JUMP_SUMMER
        );
    }
    #[test]
    fn test_dst_jump_winter() {
        assert_eq!(
            dst_info(Some(radio_datetime_utils::DST_JUMP)),
            DST_JUMP_WINTER
        );
    }
    #[test]
    fn test_dst_ann_jump_summer() {
        assert_eq!(
            dst_info(Some(
                radio_datetime_utils::DST_JUMP
                    | radio_datetime_utils::DST_ANNOUNCED
                    | radio_datetime_utils::DST_SUMMER
            )),
            DST_ANN_JUMP_SUMMER
        );
    }
    #[test]
    fn test_dst_ann_jump_winter() {
        assert_eq!(
            dst_info(Some(
                radio_datetime_utils::DST_JUMP | radio_datetime_utils::DST_ANNOUNCED
            )),
            DST_ANN_JUMP_WINTER
        );
    }
    #[test]
    fn test_dst_proc_jump_summer() {
        assert_eq!(
            dst_info(Some(
                radio_datetime_utils::DST_JUMP
                    | radio_datetime_utils::DST_PROCESSED
                    | radio_datetime_utils::DST_SUMMER
            )),
            DST_PROC_JUMP_SUMMER
        );
    }
    #[test]
    fn test_dst_proc_jump_winter() {
        assert_eq!(
            dst_info(Some(
                radio_datetime_utils::DST_JUMP | radio_datetime_utils::DST_PROCESSED
            )),
            DST_PROC_JUMP_WINTER
        );
    }
}
