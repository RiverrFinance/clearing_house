use crate::constants::{_ONE_SECOND, ONE_HOUR_NANOSECONDS};

pub fn duration_in_hours(start_time: u64) -> u64 {
    let duration_in_nano_secs = ic_cdk::api::time() - start_time;

    return duration_in_nano_secs / ONE_HOUR_NANOSECONDS;
}

pub fn duration_in_seconds(start_time: u64) -> u64 {
    let duration_in_nano_secs = ic_cdk::api::time() - start_time;

    return duration_in_nano_secs / _ONE_SECOND;
}
