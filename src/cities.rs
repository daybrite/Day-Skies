//! The user's city list: the model, its persistence, and the city-management form sections
//! that `settings.rs` composes into the Settings page.
//!
//! The list is a root-lifetime `Signal<Vec<City>>` seeded from day-part-prefs (JSON under one
//! key) and written back on every mutation. `lib.rs` derives the selector's city items from it,
//! so adding or removing a city re-derives the native sidebar rows reactively
//! (https://daybrite.dev/docs/navigation — data-driven items).

use crate::res;
use day::LocalizedText;
use day::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::OnceCell;

/// The prefs key holding the whole list as a JSON array.
const PREF_CITIES: &str = "dayskies.cities";

/// The id the "Use current location" button writes (each fix replaces the previous one).
pub const MY_LOCATION: &str = "my-location";

/// The two static selector items city ids must never shadow.
const RESERVED_IDS: [&str; 2] = ["cities", "settings"];

/// One city on the list. `id` is the selector route key (stable across edits — deep links and
/// DayScript address it) and the mock-weather seed. An empty `name` means "not renamed": the
/// display name comes from the preset catalog (or the "My location" constant) by id, localized;
/// a non-empty `name` is user text and wins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct City {
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

/// A catalog city users can add without typing coordinates. The first five are the app's
/// default list.
struct Preset {
    id: &'static str,
    name: fn() -> LocalizedText,
    latitude: f64,
    longitude: f64,
}

const PRESETS: [Preset; 10] = [
    Preset {
        id: "san-francisco",
        name: res::str::city_san_francisco,
        latitude: 37.7749,
        longitude: -122.4194,
    },
    Preset {
        id: "new-york",
        name: res::str::city_new_york,
        latitude: 40.7128,
        longitude: -74.0060,
    },
    Preset {
        id: "london",
        name: res::str::city_london,
        latitude: 51.5072,
        longitude: -0.1276,
    },
    Preset {
        id: "tokyo",
        name: res::str::city_tokyo,
        latitude: 35.6762,
        longitude: 139.6503,
    },
    Preset {
        id: "sydney",
        name: res::str::city_sydney,
        latitude: -33.8688,
        longitude: 151.2093,
    },
    Preset {
        id: "paris",
        name: res::str::city_paris,
        latitude: 48.8566,
        longitude: 2.3522,
    },
    Preset {
        id: "berlin",
        name: res::str::city_berlin,
        latitude: 52.5200,
        longitude: 13.4050,
    },
    Preset {
        id: "cairo",
        name: res::str::city_cairo,
        latitude: 30.0444,
        longitude: 31.2357,
    },
    Preset {
        id: "singapore",
        name: res::str::city_singapore,
        latitude: 1.3521,
        longitude: 103.8198,
    },
    Preset {
        id: "toronto",
        name: res::str::city_toronto,
        latitude: 43.6532,
        longitude: -79.3832,
    },
];

/// How many presets seed a fresh install (the original five cities).
const DEFAULT_COUNT: usize = 5;

fn defaults() -> Vec<City> {
    PRESETS[..DEFAULT_COUNT]
        .iter()
        .map(|p| City {
            id: p.id.to_string(),
            name: String::new(),
            latitude: p.latitude,
            longitude: p.longitude,
        })
        .collect()
}

thread_local! {
    static STORE: OnceCell<Signal<Vec<City>>> = const { OnceCell::new() };
}

/// The city list signal — created once in a detached scope (the settings-Store pattern: it must
/// outlive any page that touches it first) and seeded from the persistent store.
pub fn cities() -> Signal<Vec<City>> {
    STORE.with(|cell| {
        *cell.get_or_init(|| {
            let seed = match day_part_prefs::get(PREF_CITIES) {
                // A saved list wins, even an empty one (the user removed every city);
                // unparseable JSON falls back to the defaults rather than a dead app.
                Some(json) => serde_json::from_str(&json).unwrap_or_else(|_| defaults()),
                None => defaults(),
            };
            Scope::detached().enter(|| Signal::new(seed))
        })
    })
}

/// A city's display name: user text when renamed, else the localized catalog name by id.
pub fn title(city: &City) -> LocalizedText {
    if !city.name.is_empty() {
        return res::str::city_custom(city.name.clone());
    }
    if city.id == MY_LOCATION {
        return res::str::cities_my_location();
    }
    match PRESETS.iter().find(|p| p.id == city.id) {
        Some(p) => (p.name)(),
        // A nameless custom entry (hand-edited store): show the id rather than nothing.
        None => res::str::city_custom(city.id.clone()),
    }
}

/// Apply one mutation, persist, and publish — every list change funnels through here.
fn update(f: impl FnOnce(&mut Vec<City>)) {
    let sig = cities();
    let mut v = sig.get_untracked();
    f(&mut v);
    if let Ok(json) = serde_json::to_string(&v) {
        day_part_prefs::set(PREF_CITIES, &json);
    }
    sig.set(v);
}

/// Drag-to-reorder (docs/list.md): the row at `from` now sits at `to`. The JSON array already
/// encodes order, so the same persist path covers it — the sidebar follows the same Vec.
pub fn move_city(from: usize, to: usize) {
    update(|v| {
        if from < v.len() && to < v.len() {
            let c = v.remove(from);
            v.insert(to, c);
        }
    });
}

/// Insert or replace by id. Coordinates may have changed, so the city's memoized weather
/// resource is dropped and rebuilt on next view.
fn upsert(city: City) {
    crate::drop_state(&city.id);
    update(|v| match v.iter_mut().find(|c| c.id == city.id) {
        Some(slot) => *slot = city,
        None => v.push(city),
    });
}

fn remove(id: &str) {
    crate::drop_state(id);
    update(|v| v.retain(|c| c.id != id));
}

/// A stable route key from a display name: lowercased ASCII alphanumerics, runs of everything
/// else collapsed to `-`. Non-ASCII names (nothing survives) become "city".
fn slug(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-');
    if out.is_empty() {
        "city".into()
    } else {
        out.into()
    }
}

/// The slug, made unique against the current list and the static selector keys.
fn unique_id(name: &str, existing: &[City]) -> String {
    let base = slug(name);
    let taken = |id: &str| RESERVED_IDS.contains(&id) || existing.iter().any(|c| c.id == id);
    if !taken(&base) {
        return base;
    }
    let mut n = 2u32;
    loop {
        let cand = format!("{base}-{n}");
        if !taken(&cand) {
            return cand;
        }
        n += 1;
    }
}

/// The management page's one-line outcome readout (id `city-status`, asserted by the
/// walkthrough), localized per variant.
#[derive(Clone, Debug, PartialEq)]
enum Status {
    Idle,
    Saved,
    Removed,
    Exists,
    BadCoords,
    NeedName,
    Locating,
    Located,
    PermissionDenied,
    LocationFailed(String),
}

impl Status {
    fn text(&self) -> String {
        match self {
            Status::Idle => String::new(),
            Status::Saved => res::str::cities_status_saved().format(),
            Status::Removed => res::str::cities_status_removed().format(),
            Status::Exists => res::str::cities_status_exists().format(),
            Status::BadCoords => res::str::cities_status_bad_coords().format(),
            Status::NeedName => res::str::cities_status_need_name().format(),
            Status::Locating => res::str::cities_status_locating().format(),
            Status::Located => res::str::cities_status_located().format(),
            Status::PermissionDenied => res::str::cities_status_permission().format(),
            Status::LocationFailed(e) => {
                res::str::cities_status_location_failed(e.clone()).format()
            }
        }
    }
}

fn parse_coord(s: &str, limit: f64) -> Option<f64> {
    let v: f64 = s.trim().parse().ok()?;
    (v.is_finite() && v.abs() <= limit).then_some(v)
}

/// The city-management form pieces (shared page-scoped signals inside): the current list
/// (edit/remove per row), the preset catalog + name-and-coordinates form, the
/// current-location shortcut, and the one-line outcome readout (`city-status`,
/// walkthrough-asserted). `settings.rs` places the sections in its form and the status line
/// under it.
pub struct CitySections {
    pub list: AnyPiece,
    pub add: AnyPiece,
    pub location: AnyPiece,
    pub status: AnyPiece,
}

pub fn sections() -> CitySections {
    let list_sig = cities();
    let name = Signal::new(String::new());
    let lat = Signal::new(String::new());
    let lon = Signal::new(String::new());
    // Which city the form is editing; `None` = the Save button adds a new one.
    let editing: Signal<Option<String>> = Signal::new(None);
    let status = Signal::new(Status::Idle);
    let preset_ix = Signal::new(0usize);

    let clear_form = move || {
        name.set(String::new());
        lat.set(String::new());
        lon.set(String::new());
        editing.set(None);
    };

    // Your cities: one recycling-list row per city (drag to reorder — the order IS the sidebar
    // order, persisted with the list), edit loads it into the form, remove deletes it.
    let rows = list(
        items(move || list_sig.get(), |c: &City| c.id.clone()),
        move |slot| {
            // Recycling rows (docs/list.md): a physical cell REBINDS to different cities as the
            // list changes or reorders, so actions read the slot's CURRENT key at click time and
            // the ids re-register reactively (`id_of`) — a build-time key would go stale.
            row((
                label(move || title(&slot.get()).format()).grow(),
                button(res::str::cities_edit())
                    .action(move || {
                        let key = slot.key();
                        let Some(c) = list_sig.get_untracked().into_iter().find(|c| c.id == key)
                        else {
                            return;
                        };
                        name.set(c.name.clone());
                        lat.set(c.latitude.to_string());
                        lon.set(c.longitude.to_string());
                        editing.set(Some(c.id));
                        status.set(Status::Idle);
                    })
                    .id_of(move || format!("city-edit-{}", slot.field(|c| c.id.clone()))),
                button(res::str::cities_remove())
                    .action(move || {
                        remove(&slot.key());
                        status.set(Status::Removed);
                    })
                    .id_of(move || format!("city-remove-{}", slot.field(|c| c.id.clone()))),
            ))
            .spacing(8.0)
            .padding(Insets::symmetric(4.0, 0.0))
        },
    )
    .row_height(RowHeight::Uniform(44.0))
    .reorderable(true)
    .on_reorder(move_city)
    .id("city-rows")
    .height(280.0);

    // Add from the catalog. The picker's options are fixed, so already-added cities answer
    // with the "already on your list" status instead of duplicating.
    let preset_names: Vec<String> = PRESETS.iter().map(|p| (p.name)().format()).collect();
    let add_preset = button(res::str::cities_add_preset())
        .action(move || {
            let p = &PRESETS[preset_ix.get_untracked().min(PRESETS.len() - 1)];
            if list_sig.get_untracked().iter().any(|c| c.id == p.id) {
                status.set(Status::Exists);
                return;
            }
            upsert(City {
                id: p.id.to_string(),
                name: String::new(),
                latitude: p.latitude,
                longitude: p.longitude,
            });
            status.set(Status::Saved);
        })
        .id("city-add-preset");

    // Save: update the city being edited, else add a new one under a slug id.
    let save = button(res::str::cities_save())
        .action(move || {
            let (Some(latitude), Some(longitude)) = (
                parse_coord(&lat.get_untracked(), 90.0),
                parse_coord(&lon.get_untracked(), 180.0),
            ) else {
                status.set(Status::BadCoords);
                return;
            };
            let typed = name.get_untracked().trim().to_string();
            let city = match editing.get_untracked() {
                Some(id) => City {
                    id,
                    name: typed,
                    latitude,
                    longitude,
                },
                None => {
                    if typed.is_empty() {
                        status.set(Status::NeedName);
                        return;
                    }
                    City {
                        id: unique_id(&typed, &list_sig.get_untracked()),
                        name: typed,
                        latitude,
                        longitude,
                    }
                }
            };
            upsert(city);
            clear_form();
            status.set(Status::Saved);
        })
        .prominent()
        .id("city-save");

    let clear = button(res::str::cities_clear())
        .action(move || {
            clear_form();
            status.set(Status::Idle);
        })
        .id("city-clear");

    // One coarse fix is plenty for city-level weather. The explicit permission ask happens
    // only where the OS HAS one (Apple, Android — day-part-location never prompts by itself,
    // docs/location.md). The browser's Permissions API cannot prompt at all: its prompt lives
    // inside the geolocation call, so on web we go straight to the fix and let it ask.
    let locate = button(res::str::cities_use_location())
        .action(move || {
            status.set(Status::Locating);
            day::task(async move {
                use day_part_permissions::Permission;
                if day_part_permissions::can_prompt(Permission::Location)
                    && !day_part_permissions::request_future(Permission::Location)
                        .await
                        .is_granted()
                {
                    status.set(Status::PermissionDenied);
                    return;
                }
                match day_part_location::current_future(day_part_location::Accuracy::Coarse).await {
                    Ok(fix) => {
                        upsert(City {
                            id: MY_LOCATION.to_string(),
                            name: String::new(),
                            latitude: fix.latitude,
                            longitude: fix.longitude,
                        });
                        status.set(Status::Located);
                    }
                    Err(day_part_location::LocationError::PermissionDenied) => {
                        status.set(Status::PermissionDenied)
                    }
                    Err(e) => status.set(Status::LocationFailed(e.to_string())),
                }
            });
        })
        .id("city-locate");

    CitySections {
        list: AnyPiece::new(section((rows,)).title(res::str::cities_list_section())),
        add: AnyPiece::new(
            section((
                labeled(
                    res::str::cities_preset_label(),
                    picker(preset_names, preset_ix).id("city-preset-picker"),
                ),
                add_preset,
                labeled(
                    res::str::cities_name_label(),
                    text_field(name).id("city-name"),
                ),
                labeled(res::str::cities_lat_label(), text_field(lat).id("city-lat")),
                labeled(res::str::cities_lon_label(), text_field(lon).id("city-lon")),
                row((save, clear)).spacing(8.0),
            ))
            .title(res::str::cities_add_section()),
        ),
        location: AnyPiece::new(section((locate,)).title(res::str::cities_location_section())),
        status: label(move || status.get().text())
            .font(Font::Footnote)
            .id("city-status")
            .any(),
    }
}
