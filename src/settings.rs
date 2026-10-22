//! App settings (docs/settings.md): the two typed preferences the weather UI reads reactively
//! (temperature unit, forecast host) and the Settings/About declaration `root()` installs:
//! About · General (Day's appearance + language rows) · Weather (unit, server, refresh) ·
//! Cities (the editor in `cities.rs` as a custom group). The piece draws the surface in each
//! platform's idiom: a toolbar-tab panel on macOS, a sidebar window elsewhere on desktop, a
//! pushed page list on the phones and the web.

use crate::res;
use day_piece_settings::prelude::*;

/// The default forecast host; the Weather group lets users point at an Open-Meteo-compatible
/// proxy instead.
pub const DEFAULT_HOST: &str = "api.open-meteo.com";

pref_enum! {
    /// Stored as "c" / "f".
    pub enum Unit { Celsius = "c", Fahrenheit = "f" }
}

/// The temperature unit; the reactive temperature labels re-render from it, no refetch needed
/// (the model stays °C).
pub static UNIT: Pref<Unit> = Pref::new("dayskies.unit", || Unit::Celsius);

/// The Open-Meteo-compatible host, as typed; empty (the default) means [`DEFAULT_HOST`].
/// Every edit persists, and the weather resources track [`host`], so a changed server refetches
/// by itself.
pub static HOST: Pref<String> = Pref::new("dayskies.host", String::new);

/// The GitHub repository the About links to.
const SOURCE_URL: &str = "https://github.com/daybrite/Day-Skies";

/// The selected unit (tracked read; reactive closures re-run when it changes).
pub fn unit() -> Unit {
    UNIT.get()
}

/// A Celsius temperature formatted in the selected unit, e.g. "18°" / "64°" (tracked read).
pub fn temp(celsius: f64) -> String {
    format!("{}\u{00B0}", temp_value(celsius))
}

/// A Celsius temperature rounded in the selected unit (tracked read), for Fluent args.
pub fn temp_value(celsius: f64) -> i64 {
    let v = match unit() {
        Unit::Celsius => celsius,
        Unit::Fahrenheit => celsius * 9.0 / 5.0 + 32.0,
    };
    v.round() as i64
}

/// The forecast host the fetches use (tracked read): the stored value, trimmed, or the default
/// when it is empty.
pub fn host() -> String {
    let h = HOST.get();
    let h = h.trim();
    if h.is_empty() {
        DEFAULT_HOST.to_string()
    } else {
        h.to_string()
    }
}

/// Declare and install Settings and About. On desktop this registers the app menu's Settings…
/// and About items and their windows; everywhere, the handle's `page()` is the sidebar's
/// Settings section. Called once from `root()`, after the locale catalog installs.
pub fn install() -> Prefs {
    let unit = rows::choice(
        &UNIT,
        res::str::settings_unit_label(),
        [
            (Unit::Celsius, res::str::unit_celsius()),
            (Unit::Fahrenheit, res::str::unit_fahrenheit()),
        ],
    )
    .segmented()
    .id(day::id!(unit_picker));
    let server = rows::text(&HOST, res::str::settings_host_label())
        .placeholder(DEFAULT_HOST)
        .id(day::id!(host_field));
    // Refresh re-checks every city against the current host; a changed host already refetched
    // through the resources' tracked read of `host()`.
    let refresh = rows::action(res::str::settings_refresh(), crate::reload_all)
        .id(day::id!(settings_refresh));

    Preferences::new()
        .about(
            About::from_manifest()
                .tagline(res::str::app_tagline())
                .link(res::str::day_about_website(), "https://daybrite.dev")
                .link(res::str::day_about_source(), SOURCE_URL),
        )
        .group(general_group(res::locales::ALL))
        .group(
            Group::new("weather", res::str::settings_weather())
                .icon(res::vectors::prefs_weather)
                .subtitle(res::str::settings_weather_subtitle())
                .section(
                    Section::new()
                        .title(res::str::settings_units_section())
                        .row(unit),
                )
                .section(
                    Section::new()
                        .title(res::str::settings_server_section())
                        .footer(res::str::settings_host_hint())
                        .row(server)
                        .row(refresh),
                ),
        )
        .group(
            Group::new("cities", res::str::settings_cities())
                .icon(res::vectors::prefs_cities)
                .subtitle(|| {
                    let n = crate::cities::cities().get().len() as i64;
                    res::str::settings_cities_count(n).format()
                })
                .custom(crate::cities::editor),
        )
        .install()
}
