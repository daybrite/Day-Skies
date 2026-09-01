# Day Skies

A weather app built with [Day](https://daybrite.dev) — one Rust codebase, native widgets on every
platform. Current conditions under a condition-aware sky gradient, an hourly strip, a 10-day
forecast, and detail cards, laid out like a modern weather app and adapting between mobile and
desktop. Weather comes from [Open-Meteo](https://open-meteo.com/en/docs).

A Settings section (a native Form on every platform) selects Celsius or Fahrenheit — applied
instantly to every temperature label — and sets the Open-Meteo-compatible host, for users who
route requests through their own proxy. Both persist across launches via day-part-prefs.

## Platforms

Declared shipping targets (`Day.toml`): `windows-xaml`, `macos-appkit`, `linux-gtk`, `linux-qt`,
`ios-uikit`, `android-mdc`, `harmony-arkui`, `web-dom`. On a macOS dev box the toolkits are
exercised locally as `macos-appkit`, `macos-gtk`, `macos-qt`, `ios-uikit`, `android-mdc`,
`harmony-arkui`, and `web-dom` (GTK/Qt
are portable, so `macos-gtk`/`macos-qt` stand in for the Linux pairs).

## Day dependency

`Cargo.toml` resolves `day` and `day-build` from git (`https://github.com/daybrite/day.git`), so
the project builds on CI and on machines without a day checkout. To develop against a local
checkout instead, put a `[patch]` in the gitignored `.cargo/config.toml`:

```toml
[patch."https://github.com/daybrite/day.git"]
day = { path = "/path/to/day/crates/day" }
day-build = { path = "/path/to/day/crates/day-build" }
```

## Run it

`day launch --git` clones this repo, builds it for your desktop, and runs it — no checkout needed:

```sh
cargo install day-cli
day doctor                                                 # what's installed, what's missing
day launch --git https://github.com/daybrite/Day-Skies.git
```

`day doctor` prints the fix for anything it can't find. `day launch --git` prints where it put the
checkout, so you can `cd` there and edit the code.

From inside a clone, pick a target — Day compiles one backend per binary:

```sh
day launch -p macos-appkit                   # build + run (LIVE Open-Meteo data)
day launch -p macos-appkit -p macos-gtk -p macos-qt   # all three desktop toolkits
day launch -p ios-uikit                      # needs a booted Simulator
JAVA_HOME=$(brew --prefix openjdk@21)/libexec/openjdk.jdk/Contents/Home \
  day launch -p android-mdc               # needs JDK 21 + a running emulator/device
```

### Deterministic / mock data

Set `WEATHER_MOCK=1` to replace live networking with fixed per-city fixtures — consistent data for
screenshots and UI tests, no network required:

```sh
day launch -p macos-appkit --env WEATHER_MOCK=1
```

## Testing (DayScript)

`scripts/weather.yaml` drives the running app by element id, asserts hero content against Fluent
keys (so it passes in every language), and screenshots each city. `scripts/live-check.yaml` verifies
the real Open-Meteo fetch populates the UI.

```sh
day launch -p macos-appkit --env WEATHER_MOCK=1 --script scripts/weather.yaml
day launch -p macos-appkit --env WEATHER_MOCK=1 --locale ar --script scripts/weather.yaml
```

Screenshots land in `build/day/screenshots/<target>/<locale-or-default>/`.

## Localization

English (default), French, Arabic, and Simplified Chinese live in `locales/<code>/app.ftl`
(registered in `src/lib.rs`). Arabic lays out right-to-left automatically when launched with
`--locale ar`. Temperatures/times are pre-formatted in Rust; condition labels, weekdays, city names,
and units flow through Fluent.

## What's inside

- `src/lib.rs` — `root()` and the adaptive shell: a `selector` over cities (sidebar + detail on
  desktop, a list that pushes the detail on mobile) and the per-city reactive weather store.
- `src/weather.rs` — the data layer: Open-Meteo fetch (`day-part-http`, all platforms), WMO weather-code
  mapping, and the deterministic mock fixtures.
- `src/ui.rs` — the weather screen: sky-gradient backdrop, hero, hourly strip, a 10-day forecast
  `grid` (content-sized columns, flexible range-bar column), and a 2-column detail-card `grid`
  whose pressure card spans the full width.
- `src/settings.rs` — the Settings form (unit picker + host field) and the persistent settings store.
- `src/icons.rs` — original weather glyphs as declarative shape groups (`line`, `polygon`,
  ellipses, rounded rects) — one canvas leaf per glyph, no image assets.
- `locales/` — Fluent strings for en / fr / ar / zh-CN.
- `scripts/` — DayScript UI tests.
- `platform/` — the thin native host projects (Xcode / Gradle / hvigor) the mobile targets build
  through.

## Scope & roadmap

The app and the `day/` framework evolve in tandem — the sky gradient rides day's linear-gradient
canvas support, added for this app. Current notes:

- **Live networking runs on every platform.** The fetch goes through `day-part-http`: the
  platform HTTP stack on macOS, iOS, Android, and Windows (system proxies, VPN, and
  TLS come from the OS), the browser's own `fetch()` on `web-dom`, and a bundled ureq + rustls
  fallback on Linux and OHOS.
- **Hourly strip doesn't scroll horizontally** yet. Day's `scroll` grew a `.horizontal()` mode;
  the round-2 plan is to wire the strip to it instead of capping at 8 cells.
- **`harmony-arkui` compiles** (Rust cross-compile succeeds, TLS included) but packaging the `.hap`
  needs the OpenHarmony command-line tools (`hvigor`/`ohpm`).
- **`windows-xaml`, `linux-gtk`, `linux-qt`** build on their native hosts in CI.

## CI

`.github/workflows/ci.yml` calls the shared
[`daybrite/actions` build-day-app workflow](https://github.com/daybrite/actions) on every push,
manual trigger, or `vX.Y.Z` tag: it builds all 8 shipping targets, runs the mock-data walkthrough
in all four locales, and packs a distributable per target. Tag builds attach the packages and
screenshot zips to the GitHub release.

Weather data by [Open-Meteo.com](https://open-meteo.com) (CC BY 4.0).
