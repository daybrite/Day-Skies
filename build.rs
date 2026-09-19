//! Generate typed resource + localization constants from `resource/` (day-build): the `res` module
//! in `src/lib.rs` surfaces `res::str::<key>()` for every Fluent message, checked at compile time.
//! Also stamps the build date (UTC, ISO) into `DAY_SKIES_BUILD_DATE` for the settings About
//! section, from `SOURCE_DATE_EPOCH` when a reproducible-build harness sets it, else now.

fn main() {
    day_build::generate_resources().expect("day-build: resource codegen");

    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    let secs = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        });
    let (y, m, d) = ymd_from_days(secs.div_euclid(86_400));
    println!("cargo:rustc-env=DAY_SKIES_BUILD_DATE={y:04}-{m:02}-{d:02}");
}

/// Civil date from days since the Unix epoch (Howard Hinnant's `civil_from_days`): a dozen
/// lines beat a chrono dependency for one date stamp.
fn ymd_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}
