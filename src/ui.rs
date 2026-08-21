//! The weather screen — an Apple-Weather-inspired layout: a hero with the current conditions, an
//! hourly strip, a 10-day forecast with range bars, and a grid of detail cards. Authored once and
//! realized natively on every backend; the `selector` in `lib.rs` makes it a sidebar+detail split
//! on desktop and a push-list on mobile with no branching here.

use crate::cities::City;
use crate::icons::{Glyph, weather_icon};
use crate::res;
use crate::settings;
use crate::weather::{DataSource, DayName, Family, Weather};
use day::prelude::*;
use day::reactive::Load;

/// Map the condition family to its generated localization constant (compile-checked keys).
fn condition_label(f: Family) -> LocalizedText {
    match f {
        Family::Clear => res::str::cond_clear(),
        Family::PartlyCloudy => res::str::cond_partly_cloudy(),
        Family::Cloudy => res::str::cond_cloudy(),
        Family::Fog => res::str::cond_fog(),
        Family::Drizzle => res::str::cond_drizzle(),
        Family::Rain => res::str::cond_rain(),
        Family::Snow => res::str::cond_snow(),
        Family::Thunder => res::str::cond_thunder(),
    }
}

/// Map a day-row label to its localized constant (today, or a weekday).
fn day_label(name: DayName) -> LocalizedText {
    match name {
        DayName::Today => res::str::weather_today(),
        DayName::Weekday(0) => res::str::day_sun(),
        DayName::Weekday(1) => res::str::day_mon(),
        DayName::Weekday(2) => res::str::day_tue(),
        DayName::Weekday(3) => res::str::day_wed(),
        DayName::Weekday(4) => res::str::day_thu(),
        DayName::Weekday(5) => res::str::day_fri(),
        _ => res::str::day_sat(),
    }
}

// Content sits on a coloured sky, so text is light and cards are translucent "frosted" panels.
const TEXT: Color = Color::WHITE;
const TEXT2: Color = Color::rgba(1.0, 1.0, 1.0, 0.72);
const CARD: Color = Color::rgba(1.0, 1.0, 1.0, 0.14);
const RANGE_WARM: Color = Color::hex(0xFFD24A);
const RANGE_COOL: Color = Color::hex(0x8FC7FF);
const PRECIP: Color = Color::hex(0x9FD0FF);

/// Sky gradient per condition & time of day: a deep zenith tone falling to a paler horizon,
/// like the reference weather apps (light text reads well on all of these).
pub fn sky(family: crate::weather::Family, is_day: bool) -> LinearGradient {
    use crate::weather::Family::*;
    let (top, horizon) = match (family, is_day) {
        (Clear, true) => (0x1D5FA8, 0x7FB2E5),
        (Clear, false) => (0x070C22, 0x2C3A66),
        (PartlyCloudy, true) => (0x3A6FA5, 0x8FB7DD),
        (PartlyCloudy, false) => (0x121A35, 0x3A4A73),
        (Cloudy, _) => (0x46586B, 0x78899C),
        (Fog, _) => (0x596672, 0x8C99A6),
        (Drizzle, _) => (0x354555, 0x6A7B8C),
        (Rain, _) => (0x253444, 0x5A6B7C),
        (Snow, _) => (0x51637A, 0x93A2B4),
        (Thunder, _) => (0x14181F, 0x4A5262),
    };
    LinearGradient::vertical(Color::hex(top), Color::hex(horizon))
}

/// The whole page for one place, driven by its reactive `Load<Weather>` signal (docs/async.md).
pub fn weather_page(place: City, state: Signal<Load<Weather>>) -> impl Piece {
    let content = column((
        when(move || state.with(|s| s.is_loading()), loading_view),
        when(
            move || state.with(|s| s.error().is_some()),
            move || {
                let msg = state
                    .get_untracked()
                    .error()
                    .map(|e| e.to_string())
                    .unwrap_or_default();
                failed_view(msg)
            },
        ),
        when(
            move || state.with(|s| s.is_ready()),
            // `Either` rather than erasing both arms: the ready view is the real content and
            // the fallback is a spacer the `when` above never actually shows.
            move || match state.get_untracked() {
                Load::Ready(w) => Either::Left(loaded_view(&place, &w)),
                _ => Either::Right(spacer()),
            },
        ),
    ))
    .spacing(18.0)
    .align(HAlign::Center)
    .padding(16.0);

    // The sky gradient is a canvas-backed shape layered BEHIND the transparent scroll, so the
    // backdrop stays fixed while the forecast scrolls over it.
    let backdrop = rectangle()
        .fill_linear(move || match state.get() {
            Load::Ready(w) => sky(w.family, w.is_day),
            _ => LinearGradient::vertical(Color::hex(0x22304a), Color::hex(0x44546e)),
        })
        .grow();
    // On immersive backends (android edge-to-edge) the page runs under the status bar and
    // floating nav bar: the backdrop fills that space while the scrolled content starts below
    // it. `safe_area()` is zero everywhere else, so this pads nothing on the other targets.
    let inset = day::safe_area();
    zstack((
        backdrop,
        scroll(content.padding(Insets {
            top: inset.top,
            ..Insets::ZERO
        }))
        .grow(),
    ))
    .grow()
}

fn loaded_view(place: &City, w: &Weather) -> impl Piece + use<> {
    column((
        hero(place, w),
        hourly_strip(w),
        ten_day(w),
        detail_grid(w),
        footer(w),
    ))
    .spacing(18.0)
    .align(HAlign::Center)
    .grow_w()
}

fn hero(place: &City, w: &Weather) -> impl Piece + use<> {
    let glyph = Glyph::of(w.family, w.is_day);
    column((
        label(crate::cities::title(place))
            .font(Font::Title)
            .color(TEXT)
            .id("hero-city"),
        weather_icon(glyph, 104.0),
        {
            let t = w.temp;
            label(move || settings::temp(t))
                .font(Font::System(80.0))
                .color(TEXT)
                .id("hero-temp")
        },
        label(condition_label(w.family))
            .font(Font::Title3)
            .color(TEXT)
            .id("hero-condition"),
        {
            let (h, l) = (w.high, w.low);
            label(move || {
                res::str::weather_hilo(settings::temp_value(h), settings::temp_value(l)).format()
            })
            .font(Font::Headline)
            .color(TEXT)
            .id("hero-hilo")
        },
    ))
    .spacing(6.0)
    .align(HAlign::Center)
    .padding(8.0)
}

fn section_header(text: LocalizedText) -> impl Piece {
    label(text).font(Font::Caption).color(TEXT2)
}

fn hourly_strip(w: &Weather) -> impl Piece + use<> {
    let cells: Vec<AnyPiece> = w
        .hourly
        .iter()
        .take(8)
        .map(|h| {
            let head: AnyPiece = match &h.hour_label {
                None => label(res::str::weather_now())
                    .font(Font::Caption)
                    .color(TEXT)
                    .any(),
                Some(hh) => label(hh.clone()).font(Font::Caption).color(TEXT).any(),
            };
            let t = h.temp;
            column((
                head,
                weather_icon(Glyph::of(h.family, h.is_day), 30.0),
                label(move || settings::temp(t))
                    .font(Font::Subheadline)
                    .color(TEXT),
            ))
            .spacing(6.0)
            .align(HAlign::Center)
            .grow_w()
            .any()
        })
        .collect();

    card(
        column((
            section_header(res::str::weather_hourly()),
            row(PieceVec(cells)).spacing(2.0),
        ))
        .spacing(10.0)
        .align(HAlign::Leading),
    )
    .id("hourly")
}

fn ten_day(w: &Weather) -> impl Piece + use<> {
    let wmin = w.daily.iter().map(|d| d.low).fold(f64::MAX, f64::min);
    let wmax = w.daily.iter().map(|d| d.high).fold(f64::MIN, f64::max);

    // One grid row per day (docs/grid.md): the day, icon, precip, and temperature columns size
    // to their widest cell, and the range bar's `grow_w` makes its column take the leftover
    // width — no hand-tuned widths, and a plain `spacer()` keeps the precip column aligned on
    // dry days.
    let rows: Vec<AnyPiece> = w
        .daily
        .iter()
        .map(|d| {
            let precip: AnyPiece = if d.precip > 0 {
                label(format!("{}%", d.precip))
                    .font(Font::Caption)
                    .color(PRECIP)
                    .any()
            } else {
                spacer().any()
            };
            grid_row((
                label(day_label(d.name)).font(Font::Body).color(TEXT),
                weather_icon(Glyph::of(d.family, true), 26.0),
                precip,
                {
                    let l = d.low;
                    label(move || settings::temp(l))
                        .font(Font::Body)
                        .color(TEXT2)
                        .grid_align(Alignment::Trailing)
                },
                range_bar(d.low, d.high, wmin, wmax),
                {
                    let h = d.high;
                    label(move || settings::temp(h))
                        .font(Font::Body)
                        .color(TEXT)
                        .grid_align(Alignment::Trailing)
                },
            ))
            .any()
        })
        .collect();

    card(
        column((
            section_header(res::str::weather_10day()),
            grid(PieceVec(rows))
                .column_spacing(8.0)
                .row_spacing(12.0)
                .align(Alignment::Leading)
                .grow_w(),
        ))
        .spacing(10.0)
        .align(HAlign::Leading),
    )
    .id("ten-day")
}

/// A temperature range bar: a faint full-width track with a warm-to-cool segment for this day's
/// low→high mapped into the whole week's range (like Apple's forecast bars). A size-aware shape
/// group: the segment positions derive from the laid-out width, in one canvas leaf.
fn range_bar(low: f64, high: f64, wmin: f64, wmax: f64) -> impl Piece {
    shape_group_fn(move |size| {
        if size.width <= 0.0 || size.height <= 0.0 {
            return Vec::new();
        }
        let h = 6.0 / size.height;
        let y = 0.5 - h / 2.0;
        let span = (wmax - wmin).max(1.0);
        let x0 = ((low - wmin) / span).clamp(0.0, 1.0);
        let x1 = ((high - wmin) / span).clamp(0.0, 1.0);
        let w = (x1 - x0).max(6.0 / size.width);
        // Cooler at the low end, warmer at the high end — approximate with a two-stop split.
        let mid = x0 + w / 2.0;
        vec![
            rounded_rectangle(3.0)
                .fill(Color::rgba(1.0, 1.0, 1.0, 0.22))
                .at(0.0, y, 1.0, h),
            rounded_rectangle(3.0)
                .fill(RANGE_COOL)
                .at(x0, y, mid - x0, h),
            rounded_rectangle(3.0)
                .fill(RANGE_WARM)
                .at(mid, y, x0 + w - mid, h),
        ]
    })
    .height(22.0)
    .grow_w()
}

fn detail_grid(w: &Weather) -> impl Piece + use<> {
    let feels_c = w.feels_like;
    let humidity = format!("{}%", w.humidity);
    let wind = format!("{} km/h", w.wind_kmh.round() as i64);
    let uv = format!("{}", w.uv.round() as i64);
    let pressure = format!("{} hPa", w.pressure.round() as i64);
    // A real 2-column grid (docs/grid.md): every card grows, so both columns split the width
    // evenly, and the pressure card — a bare child outside any row — spans the full grid.
    grid((
        grid_row((
            detail_card(res::str::detail_feels(), move || settings::temp(feels_c)),
            detail_card(res::str::detail_humidity(), humidity),
        )),
        grid_row((
            detail_card(res::str::detail_wind(), wind),
            detail_card(res::str::detail_uv(), uv),
        )),
        grid_row((
            detail_card(res::str::detail_sunrise(), w.sunrise.clone()),
            detail_card(res::str::detail_sunset(), w.sunset.clone()),
        )),
        detail_card(res::str::detail_pressure(), pressure),
    ))
    .spacing(12.0)
    .align(Alignment::TopLeading)
    .grow_w()
}

fn detail_card<M>(title: LocalizedText, value: impl IntoText<M>) -> impl Piece {
    card(
        column((
            label(title).font(Font::Caption).color(TEXT2),
            label(value).font(Font::Title2).color(TEXT),
        ))
        .spacing(8.0)
        .align(HAlign::Leading),
    )
}

fn footer(w: &Weather) -> impl Piece + use<> {
    let mut items: Vec<AnyPiece> = Vec::new();
    if w.source == DataSource::Mock {
        items.push(
            label(res::str::data_mock())
                .font(Font::Caption2)
                .color(TEXT2)
                .id("data-source")
                .any(),
        );
    }
    items.push(
        label(res::str::data_attribution())
            .font(Font::Caption2)
            .color(TEXT2)
            .any(),
    );
    column(PieceVec(items))
        .spacing(2.0)
        .align(HAlign::Center)
        .padding(6.0)
}

/// Wrap content in a translucent rounded card.
fn card(inner: impl Piece) -> impl Piece {
    inner
        .padding(16.0)
        .background(CARD)
        .corner_radius(18.0)
        .grow_w()
}

fn loading_view() -> impl Piece {
    column((label(res::str::weather_loading())
        .font(Font::Headline)
        .color(TEXT)
        .id("loading"),))
    .align(HAlign::Center)
    .padding(40.0)
}

fn failed_view(msg: String) -> impl Piece {
    column((
        label(res::str::weather_error())
            .font(Font::Headline)
            .color(TEXT)
            .id("error"),
        label(msg).font(Font::Caption).color(TEXT2),
    ))
    .spacing(8.0)
    .align(HAlign::Center)
    .padding(40.0)
}
