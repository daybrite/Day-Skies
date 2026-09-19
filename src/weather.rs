//! Weather data layer for Day Skies.
//!
//! Fetches from the Open-Meteo forecast API (https://open-meteo.com/en/docs) and processes the JSON
//! into the small `Weather` model the UI renders. A deterministic mock path (env `WEATHER_MOCK=1`)
//! gives stable data so DayScript screenshots are reproducible.
//!
//! Networking goes through `day-part-http` (day's docs/http.md): the platform HTTP stack on
//! macOS, iOS, Android, and Windows (system proxies, VPN, and TLS come from the OS), with a
//! bundled ureq+rustls fallback on Linux and OHOS; this app ships no TLS code of its own.

use serde::Deserialize;

/// A place we can show weather for: the user's [`City`] model (`cities.rs`). `id` seeds the
/// deterministic mock data; unknown ids (custom cities) get the fixed default fixture.
use crate::cities::City;

/// Where the currently-displayed data came from, surfaced in the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataSource {
    /// A real Open-Meteo response.
    Live,
    /// Deterministic fixture requested via `WEATHER_MOCK=1` (for screenshots/tests).
    Mock,
}

/// A fetch/parse failure, displayable (`Error + Send + Sync` so it rides `Load::Failed`).
#[derive(Debug)]
pub struct WeatherError(pub String);

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for WeatherError {}

/// The processed model the UI renders: unit-agnostic values in °C / km/h / hPa.
#[derive(Clone, Debug)]
pub struct Weather {
    pub source: DataSource,
    pub temp: f64,
    pub family: Family,
    pub is_day: bool,
    pub high: f64,
    pub low: f64,
    pub feels_like: f64,
    pub humidity: i64,
    pub wind_kmh: f64,
    pub uv: f64,
    pub pressure: f64,
    pub sunrise: String, // "HH:MM"
    pub sunset: String,  // "HH:MM"
    pub hourly: Vec<Hour>,
    pub daily: Vec<Day>,
}

#[derive(Clone, Debug)]
pub struct Hour {
    /// `None` = the current hour (rendered as the localized "Now").
    pub hour_label: Option<String>,
    pub temp: f64,
    pub family: Family,
    pub is_day: bool,
}

#[derive(Clone, Debug)]
pub struct Day {
    pub name: DayName,
    pub low: f64,
    pub high: f64,
    pub family: Family,
    pub precip: i64,
}

/// A day's row label: today, or a weekday (`0` = Sunday … `6` = Saturday). The UI maps this to a
/// generated localization constant (`res::str::weather_today()` / `res::str::day_mon()` …).
#[derive(Clone, Copy, Debug)]
pub enum DayName {
    Today,
    Weekday(u8),
}

/// The WMO weather-code families we distinguish (https://open-meteo.com/en/docs, `weather_code`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Clear,
    PartlyCloudy,
    Cloudy,
    Fog,
    Drizzle,
    Rain,
    Snow,
    Thunder,
}

impl Family {
    pub fn from_code(code: u16) -> Family {
        match code {
            0 | 1 => Family::Clear,
            2 => Family::PartlyCloudy,
            3 => Family::Cloudy,
            45 | 48 => Family::Fog,
            51 | 53 | 55 | 56 | 57 => Family::Drizzle,
            61 | 63 | 65 | 66 | 67 | 80 | 81 | 82 => Family::Rain,
            71 | 73 | 75 | 77 | 85 | 86 => Family::Snow,
            95 | 96 | 99 => Family::Thunder,
            _ => Family::Cloudy,
        }
    }
}

// ---------------------------------------------------------------------------
// Mock / sample fixtures: deterministic, so screenshots are reproducible.
// ---------------------------------------------------------------------------

/// Is the app forced into deterministic mock mode? Driven by `--env WEATHER_MOCK=1` at launch.
/// `day::env` rather than `std::env`: on web-dom the flag arrives as a page query parameter,
/// not process environment (a browser has none).
pub fn is_mock() -> bool {
    match day::env("WEATHER_MOCK") {
        Some(v) => !(v.is_empty() || v == "0" || v.eq_ignore_ascii_case("false")),
        None => false,
    }
}

/// Per-city deterministic parameters chosen to show off distinct conditions & the day/night sky.
fn mock_params(id: &str) -> (f64, u16, i64) {
    // (base °C, WMO code, local clock hour 0–23)
    match id {
        "san-francisco" => (18.0, 0, 14), // clear afternoon
        "new-york" => (27.0, 2, 12),      // partly cloudy midday
        "london" => (14.0, 63, 10),       // rain, morning
        "tokyo" => (24.0, 1, 22),         // clear night
        "sydney" => (12.0, 3, 16),        // overcast afternoon
        _ => (20.0, 0, 12),
    }
}

/// Build a deterministic `Weather` fixture for a place (used for `WEATHER_MOCK` and tests).
pub fn mock(place: &City) -> Weather {
    let (base, code, hour) = mock_params(&place.id);
    let family = Family::from_code(code);
    let is_day = (6..20).contains(&hour);

    // A smooth diurnal curve peaking mid-afternoon (~15:00), continuous across midnight and
    // anchored so the current hour equals `base` (the headline temp), so the "Now" cell matches.
    let anchor = ((hour as f64 - 15.0) / 24.0 * std::f64::consts::TAU).cos();
    let temp_at = |i: i64| -> f64 {
        let phase = ((hour + i) as f64 - 15.0) / 24.0 * std::f64::consts::TAU;
        base + 6.0 * (phase.cos() - anchor)
    };

    // 24 hourly entries starting at `hour`.
    let mut hourly = Vec::new();
    for i in 0..24 {
        let h = (hour + i) % 24;
        hourly.push(Hour {
            hour_label: if i == 0 {
                None
            } else {
                Some(format!("{:02}", h))
            },
            temp: temp_at(i),
            family,
            is_day: (6..20).contains(&h),
        });
    }

    // 10 daily entries. Codes rotate a little to vary the icons down the list.
    let rotate = [code, 2, 3, 61, 0, 80, 2, 3, 1, 45];
    let mut daily = Vec::new();
    for i in 0..10i64 {
        let f = Family::from_code(rotate[i as usize % rotate.len()]);
        daily.push(Day {
            name: if i == 0 {
                DayName::Today
            } else {
                DayName::Weekday(weekday_index_offset(2026, 7, 15, i))
            },
            low: base - 6.0 - (i % 3) as f64,
            high: base + 3.0 + (i % 4) as f64,
            family: f,
            precip: match f {
                Family::Rain | Family::Thunder => 70 - (i % 3) * 10,
                Family::Drizzle | Family::Snow => 40,
                Family::PartlyCloudy => 10,
                _ => 0,
            },
        });
    }

    Weather {
        source: DataSource::Mock,
        temp: base,
        family,
        is_day,
        high: base + 3.0,
        low: base - 6.0,
        feels_like: base - 1.0,
        humidity: 68,
        wind_kmh: 12.0,
        uv: if is_day { 5.0 } else { 0.0 },
        pressure: 1013.0,
        sunrise: "06:12".into(),
        sunset: "20:34".into(),
        hourly,
        daily,
    }
}

/// Weekday index for a base date advanced by `offset` days (mock only).
fn weekday_index_offset(y: i64, m: i64, d: i64, offset: i64) -> u8 {
    weekday_index(y, m, d + offset)
}

// ---------------------------------------------------------------------------
// Live fetch + shared processing.
// ---------------------------------------------------------------------------

/// Resolve a place to a full `Weather`. Mock data resolves synchronously (the Resource
/// eager-poll case); the live path awaits the platform fetch and parses on the UI thread; an
/// Open-Meteo payload is tens of KB, so the parse cost there is negligible.
pub async fn load(place: City, host: String) -> Result<Weather, WeatherError> {
    if is_mock() {
        return Ok(mock(&place));
    }
    net::fetch(place, host).await.map_err(WeatherError)
}

/// Open-Meteo JSON, only the fields we consume.
#[derive(Deserialize)]
struct ApiResp {
    current: ApiCurrent,
    hourly: ApiHourly,
    daily: ApiDaily,
}
#[derive(Deserialize)]
struct ApiCurrent {
    time: String,
    temperature_2m: f64,
    relative_humidity_2m: f64,
    apparent_temperature: f64,
    is_day: i64,
    weather_code: u16,
    wind_speed_10m: f64,
    pressure_msl: f64,
    uv_index: f64,
}
#[derive(Deserialize)]
struct ApiHourly {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
    weather_code: Vec<u16>,
}
#[derive(Deserialize)]
struct ApiDaily {
    time: Vec<String>,
    weather_code: Vec<u16>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
    sunrise: Vec<String>,
    sunset: Vec<String>,
    precipitation_probability_max: Vec<Option<i64>>,
}

/// Turn a parsed Open-Meteo response into our `Weather` model.
fn process(api: ApiResp) -> Weather {
    let current_family = Family::from_code(api.current.weather_code);
    let is_day = api.current.is_day == 1;

    // Hourly: start at the first hour at/after "now", take the next 24.
    let now_key = &api.current.time[..api.current.time.len().min(13)]; // "YYYY-MM-DDTHH"
    let start = api
        .hourly
        .time
        .iter()
        .position(|t| t.as_str() >= now_key)
        .unwrap_or(0);
    let mut hourly = Vec::new();
    for i in 0..24usize {
        let idx = start + i;
        if idx >= api.hourly.time.len() {
            break;
        }
        let t = &api.hourly.time[idx];
        let hh = t.get(11..13).unwrap_or("00").to_string();
        let h: i64 = hh.parse().unwrap_or(12);
        hourly.push(Hour {
            hour_label: if i == 0 { None } else { Some(hh) },
            temp: api.hourly.temperature_2m.get(idx).copied().unwrap_or(0.0),
            family: Family::from_code(api.hourly.weather_code.get(idx).copied().unwrap_or(0)),
            is_day: (6..20).contains(&h),
        });
    }

    // Daily: up to 10.
    let mut daily = Vec::new();
    for i in 0..api.daily.time.len().min(10) {
        let date = &api.daily.time[i];
        let (y, m, d) = parse_ymd(date);
        daily.push(Day {
            name: if i == 0 {
                DayName::Today
            } else {
                DayName::Weekday(weekday_index(y, m, d))
            },
            low: api.daily.temperature_2m_min.get(i).copied().unwrap_or(0.0),
            high: api.daily.temperature_2m_max.get(i).copied().unwrap_or(0.0),
            family: Family::from_code(api.daily.weather_code.get(i).copied().unwrap_or(0)),
            precip: api
                .daily
                .precipitation_probability_max
                .get(i)
                .copied()
                .flatten()
                .unwrap_or(0),
        });
    }

    Weather {
        source: DataSource::Live,
        temp: api.current.temperature_2m,
        family: current_family,
        is_day,
        high: api
            .daily
            .temperature_2m_max
            .first()
            .copied()
            .unwrap_or(api.current.temperature_2m),
        low: api
            .daily
            .temperature_2m_min
            .first()
            .copied()
            .unwrap_or(api.current.temperature_2m),
        feels_like: api.current.apparent_temperature,
        humidity: api.current.relative_humidity_2m.round() as i64,
        wind_kmh: api.current.wind_speed_10m,
        uv: api.current.uv_index,
        pressure: api.current.pressure_msl,
        sunrise: hm(api.daily.sunrise.first()),
        sunset: hm(api.daily.sunset.first()),
        hourly,
        daily,
    }
}

/// The forecast URL for a place: 10 days, °C, auto timezone. The host is the user's configured
/// Open-Meteo-compatible server (settings), defaulting to api.open-meteo.com.
fn forecast_url(place: &City, host: &str) -> String {
    format!(
        "https://{host}/v1/forecast?latitude={:.4}&longitude={:.4}\
         &current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,weather_code,wind_speed_10m,pressure_msl,uv_index\
         &hourly=temperature_2m,weather_code\
         &daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,precipitation_probability_max\
         &timezone=auto&forecast_days=10&temperature_unit=celsius&wind_speed_unit=kmh",
        place.latitude, place.longitude
    )
}

mod net {
    use super::{ApiResp, City, Weather, forecast_url, process};
    use std::time::Duration;

    /// HTTPS GET + parse via the platform HTTP stack, await-style: `fetch_future` starts the
    /// request immediately and its drop (a superseded or disposed Resource fetch) cancels it
    /// where the platform can (docs/http.md's cancel matrix).
    pub async fn fetch(place: City, host: String) -> Result<Weather, String> {
        let url = forecast_url(&place, &host);
        let resp = day_part_http::fetch_future(
            day_part_http::Request::get(url).timeout(Duration::from_secs(15)),
        )
        .await
        .map_err(|e| format!("request failed: {e}"))?;
        // day-part-http treats 4xx/5xx as responses, not errors; surface them here.
        if !(200..300).contains(&resp.status) {
            return Err(format!("request failed: HTTP {}", resp.status));
        }
        let api: ApiResp =
            serde_json::from_slice(&resp.body).map_err(|e| format!("parse failed: {e}"))?;
        Ok(process(api))
    }
}

// ---------------------------------------------------------------------------
// Small date helpers (no chrono dependency).
// ---------------------------------------------------------------------------

fn parse_ymd(date: &str) -> (i64, i64, i64) {
    let mut it = date.split(['-', 'T']);
    let y = it.next().and_then(|s| s.parse().ok()).unwrap_or(2026);
    let m = it.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let d = it.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    (y, m, d)
}

/// Extract "HH:MM" from an ISO timestamp like "2026-07-15T06:12".
fn hm(t: Option<&String>) -> String {
    t.and_then(|s| s.get(11..16)).unwrap_or("--:--").to_string()
}

/// Sakamoto's algorithm → weekday index (`0` = Sunday … `6` = Saturday). Handles day overflow into
/// following months (used with small offsets on the mock path).
fn weekday_index(y: i64, m: i64, mut d: i64) -> u8 {
    // Normalise a day-of-month overflow into the following months (mock offsets stay < 31).
    let mut y = y;
    let mut m = m;
    loop {
        let dim = days_in_month(y, m);
        if d <= dim {
            break;
        }
        d -= dim;
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { y - 1 } else { y };
    (((yy + yy / 4 - yy / 100 + yy / 400 + t[(m - 1) as usize] + d) % 7).rem_euclid(7)) as u8
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}
