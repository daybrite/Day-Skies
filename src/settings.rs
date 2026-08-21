//! App settings, one page: About, language, appearance, temperature unit, the user's city
//! list (`cities.rs` builds those sections), and the Open-Meteo host — persisted via
//! day-part-prefs and exposed as signals the UI reads reactively. A native Form
//! (docs/forms.md): grouped section cards on every platform.

use crate::res;
use day::prelude::*;
use std::cell::OnceCell;

/// The default forecast host; the settings page lets users point at an Open-Meteo-compatible
/// proxy instead.
pub const DEFAULT_HOST: &str = "api.open-meteo.com";

const PREF_UNIT: &str = "dayskies.unit"; // "c" | "f"
const PREF_HOST: &str = "dayskies.host";
const PREF_LOCALE: &str = "dayskies.locale"; // a res::locales::ALL tag; absent = system
const PREF_THEME: &str = "dayskies.theme"; // "light" | "dark"; absent = system

/// Apply the persisted language and theme overrides — called once from `root()`, right after
/// the locale catalog installs and before the first page builds. The shared piece owns the
/// mechanics (docs/windows.md), including the env-wins rule: a `DAY_THEME`/`--locale` launch
/// keeps its override no matter what an earlier run persisted.
pub fn apply_startup() {
    day_piece_settings::apply_startup(PREF_THEME, PREF_LOCALE);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    Celsius,
    Fahrenheit,
}

/// The settings signals, created once in the root scope (first access must happen during the
/// root build) and seeded from the persistent store.
struct Store {
    /// Picker index: 0 = Celsius, 1 = Fahrenheit.
    unit: Signal<usize>,
    /// Host name as edited in the field (persisted + applied by the Save action).
    host: Signal<String>,
    /// The host the fetches actually use — written ONLY by Save, so the weather `Resource`s
    /// (which track [`host`]) refetch on Save, never per keystroke in the field.
    applied_host: Signal<String>,
}

thread_local! {
    static STORE: OnceCell<Store> = const { OnceCell::new() };
}

fn with_store<R>(f: impl FnOnce(&Store) -> R) -> R {
    STORE.with(|cell| {
        f(cell.get_or_init(|| {
            // App-lifetime signals: a detached scope (the matrix-core pattern), NOT the build
            // scope of whichever page happens to touch settings first — that scope is disposed
            // when the page rebuilds, and a disposed signal panics on read.
            Scope::detached().enter(|| {
                let unit = Signal::new(match day_part_prefs::get(PREF_UNIT).as_deref() {
                    Some("f") => 1,
                    _ => 0,
                });
                let host = Signal::new(
                    day_part_prefs::get(PREF_HOST).unwrap_or_else(|| DEFAULT_HOST.to_string()),
                );
                let applied_host = Signal::new(host.get_untracked());
                // The unit applies (and persists) immediately on selection; the reactive
                // temperature labels re-render from the signal, no refetch needed (the model
                // stays °C).
                watch(
                    move || unit.get(),
                    |new, _| {
                        day_part_prefs::set(PREF_UNIT, if *new == 1 { "f" } else { "c" });
                    },
                );
                Store {
                    unit,
                    host,
                    applied_host,
                }
            })
        }))
    })
}

/// The selected unit (tracked read — reactive closures re-run when it changes).
pub fn unit() -> Unit {
    if with_store(|s| s.unit.get()) == 1 {
        Unit::Fahrenheit
    } else {
        Unit::Celsius
    }
}

/// A Celsius temperature formatted in the selected unit, e.g. "18°" / "64°" (tracked read).
pub fn temp(celsius: f64) -> String {
    format!("{}\u{00B0}", temp_value(celsius))
}

/// A Celsius temperature rounded in the selected unit (tracked read) — for Fluent args.
pub fn temp_value(celsius: f64) -> i64 {
    let v = match unit() {
        Unit::Celsius => celsius,
        Unit::Fahrenheit => celsius * 9.0 / 5.0 + 32.0,
    };
    v.round() as i64
}

/// The currently-applied forecast host — a TRACKED read of the Save-applied signal, so a
/// weather `Resource` using it as its source refetches when Save applies a new host.
pub fn host() -> String {
    let h = with_store(|s| s.applied_host.get());
    let h = h.trim();
    if h.is_empty() {
        DEFAULT_HOST.to_string()
    } else {
        h.to_string()
    }
}

/// The Settings page: About, language, appearance (where the backend supports a runtime
/// override — `Cap::Appearance`), units, the city-management sections, and the weather
/// server, as one Form.
pub fn settings_page() -> impl Piece {
    let (unit_sig, host_sig) = with_store(|s| (s.unit, s.host));

    // Language + appearance: the shared settings rows (docs/windows.md — day-piece-settings).
    // Same ids (`language-picker`/`theme-picker`), same persistence keys, same live apply;
    // the appearance row is Cap::Appearance-gated inside the piece (empty when unsupported).
    let language_row = day_piece_settings::language_picker(PREF_LOCALE, res::locales::ALL);
    let theme_row = day_piece_settings::appearance_picker(PREF_THEME);

    // The segmented picker has no ArkUI backend yet; HarmonyOS gets a native toggle instead.
    // One id call for both variants — the branches are cfg-disjoint, so a single
    // call site keeps the dayscript address stable AND keeps `day lint`'s duplicate-id check
    // (which reads text, not cfg) from seeing two.
    let unit_row = {
        #[cfg(not(target_env = "ohos"))]
        let (unit_label, unit_control) = (
            res::str::settings_unit_label(),
            picker(
                [
                    res::str::unit_celsius().format(),
                    res::str::unit_fahrenheit().format(),
                ],
                unit_sig,
            )
            .segmented(),
        );
        #[cfg(target_env = "ohos")]
        let (unit_label, unit_control) = {
            let fahrenheit = Signal::new(unit_sig.get_untracked() == 1);
            watch(
                move || fahrenheit.get(),
                move |on, _| unit_sig.set(if *on { 1 } else { 0 }),
            );
            (res::str::unit_fahrenheit(), toggle(fahrenheit))
        };
        labeled(unit_label, unit_control.id("unit-picker"))
    };

    let save = button(res::str::settings_save())
        .action(move || {
            let host = host_sig.get_untracked();
            let host = host.trim();
            let host = if host.is_empty() { DEFAULT_HOST } else { host };
            host_sig.set(host.to_string());
            with_store(|s| s.applied_host.set(host.to_string()));
            day_part_prefs::set(PREF_HOST, host);
            crate::reload_all();
        })
        .prominent()
        .id("settings-save");

    let city = crate::cities::sections();

    // About, app-level preferences, the city list, then the server — heterogeneous and
    // partly conditional, so a PieceVec rather than a tuple.
    let mut parts: Vec<AnyPiece> = vec![
        // About: name, version, build date (stamped by build.rs), and the project page.
        AnyPiece::new(
            section((
                labeled(
                    res::str::settings_name_label(),
                    label(res::str::app_title()),
                ),
                labeled(
                    res::str::settings_version_label(),
                    label(env!("CARGO_PKG_VERSION")).id("about-version"),
                ),
                labeled(
                    res::str::settings_build_label(),
                    label(env!("DAY_SKIES_BUILD_DATE")).id("about-build"),
                ),
                link(
                    res::str::settings_github(),
                    "https://github.com/Day-Skies/Day-Skies",
                )
                .id("about-github"),
            ))
            .title(res::str::settings_about_section()),
        ),
        AnyPiece::new(section((language_row,)).title(res::str::settings_language_section())),
    ];
    // The appearance row gates itself on Cap::Appearance (empty piece when unsupported) —
    // an empty section card would still render, so keep the section gate too.
    if capability(Cap::Appearance) != Support::Unsupported {
        parts.push(AnyPiece::new(
            section((theme_row,)).title(res::str::settings_theme_section()),
        ));
    }
    parts.push(AnyPiece::new(
        section((unit_row,)).title(res::str::settings_units_section()),
    ));
    parts.push(city.list);
    parts.push(city.add);
    parts.push(city.location);
    parts.push(AnyPiece::new(
        section((
            labeled(
                res::str::settings_host_label(),
                text_field(host_sig)
                    .placeholder(DEFAULT_HOST.to_string())
                    .id("host-field"),
            ),
            label(res::str::settings_host_hint()).font(Font::Footnote),
            save,
        ))
        .title(res::str::settings_server_section()),
    ));

    scroll(
        column((form(PieceVec(parts)), city.status))
            .spacing(12.0)
            .align(HAlign::Leading)
            // Immersive backends (android edge-to-edge): start below the transparent chrome;
            // `safe_area()` is zero everywhere else, so this is 16.0 all round elsewhere.
            .padding(Insets {
                top: 16.0 + day::safe_area().top,
                leading: 16.0,
                bottom: 16.0,
                trailing: 16.0,
            }),
    )
    .grow()
}
