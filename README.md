# Day Skies

A weather app that draws each city under a sky that matches its forecast, built with
[Day](https://daybrite.dev) in one Rust codebase and rendered with the platform's own widgets on
iPhone, Android, Mac, Windows, Linux, HarmonyOS, and the web.

<p align="center">
  <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/ios-uikit/en/san-francisco.png" width="200" alt="San Francisco on iPhone"></kbd>
  <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/android-mdc/en/london.png" width="200" alt="London on Android"></kbd>
  <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/ios-uikit/en/tokyo.png" width="200" alt="Tokyo on iPhone"></kbd>
  <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/android-mdc/en/sydney.png" width="200" alt="Sydney on Android"></kbd>
</p>

## Run it in one command

Install the `day` CLI, then let it clone, build, and launch the app for your desktop:

```sh
cargo install day-cli
day launch --git https://github.com/daybrite/Day-Skies.git
```

That is the whole setup on a Mac with Xcode's command-line tools. On Linux and Windows,
`day doctor` lists what the toolkit needs and prints the install command for anything missing. The
launch prints where it put the checkout, so you can open the code and change it.

## What you get

Every city opens on the current temperature, the day's high and low, an hourly strip, a ten-day
forecast with a drawn range bar per day, and detail cards for feels-like, humidity, wind, UV,
sunrise and sunset, and pressure. The gradient behind all of it follows the conditions: clear,
cloudy, fog, rain, snow, or thunderstorms.

<p align="center">
  <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/macos-appkit/en/san-francisco.png" width="720" alt="San Francisco on macOS, with the city list beside the forecast"></kbd>
</p>

On a desktop the cities sit in a sidebar next to the forecast. On a phone the same list pushes
each city's screen. Both come from one `selector` in `src/lib.rs`, and Day picks the native
container for the window size.

- Add as many cities as you like, by name, by coordinates, or from your current location.
- Switch between Celsius and Fahrenheit and every temperature on screen updates at once.
- Read it in English, French, Arabic, or Simplified Chinese. Arabic lays out right to left.
- Point the app at your own Open-Meteo-compatible server from Settings.

There is nothing to sign in to and nothing is collected. Forecasts come straight from
[Open-Meteo](https://open-meteo.com).

## The same code on every platform

These captures come from the app's own CI, which runs the walkthrough on every target and
publishes the results to the [gallery](https://daybrite.dev/gallery/Day-Skies/).

| macOS · AppKit | Windows · XAML | Linux · GTK |
|:---:|:---:|:---:|
| <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/macos-appkit/en/new-york.png" width="300" alt="New York on macOS"></kbd> | <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/windows-xaml/en/new-york.png" width="300" alt="New York on Windows"></kbd> | <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/linux-gtk/en/new-york.png" width="300" alt="New York on GTK"></kbd> |

| Linux · Qt | Web · DOM | HarmonyOS · ArkUI |
|:---:|:---:|:---:|
| <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/linux-qt/en/london.png" width="300" alt="London on Qt"></kbd> | <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/web-dom/en/london.png" width="300" alt="London in the browser"></kbd> | <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/harmony-arkui/en/london.png" width="150" alt="London on HarmonyOS"></kbd> |

Settings is a native form on each platform, and the Arabic build mirrors the whole layout:

<p align="center">
  <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/ios-uikit/en/settings.png" width="200" alt="Settings on iPhone"></kbd>
  <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/android-mdc/en/settings.png" width="200" alt="Settings on Android"></kbd>
  <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/ios-uikit/ar/san-francisco.png" width="200" alt="San Francisco in Arabic on iPhone"></kbd>
  <kbd><img src="https://daybrite.github.io/Day-Skies/gallery/android-mdc/zh-CN/tokyo.png" width="200" alt="Tokyo in Simplified Chinese on Android"></kbd>
</p>

## Build from a clone

Day compiles one toolkit backend per binary, so name a target when you build or launch. Every
target the app ships is listed in `Day.toml`.

```sh
day doctor                                   # toolchains present and missing, with fixes
day launch -p macos-appkit                   # build + run with live Open-Meteo data
day launch -p macos-appkit -p macos-gtk -p macos-qt   # all three desktop toolkits side by side
day launch -p ios-uikit                      # needs a booted Simulator
day launch -p android-mdc                    # needs a JDK and a running emulator or device
day launch -p web-dom                        # serves the WebAssembly build locally
```

Set `WEATHER_MOCK=1` to swap live networking for fixed per-city fixtures. Screenshots and tests
use it so every run draws the same skies:

```sh
day launch -p macos-appkit --env WEATHER_MOCK=1
```

### The walkthrough

`scripts/weather.yaml` is a [dayscript](https://daybrite.dev/docs/dayscript): it drives the running
app by element id, checks the hero content against Fluent keys so the same script passes in every
language, and screenshots each city. `scripts/live-check.yaml` confirms a real Open-Meteo fetch
fills the screen.

```sh
day launch -p macos-appkit --env WEATHER_MOCK=1 --script scripts/weather.yaml
day launch -p macos-appkit --env WEATHER_MOCK=1 --locale ar --script scripts/weather.yaml
```

Captures land in `build/day/screenshots/<target>/<locale>/`. CI runs the same script on all eight
targets in all four locales on every push, and a `vX.Y.Z` tag attaches the packaged apps to a
GitHub release.

### Developing against a local Day checkout

`Cargo.toml` takes the `day` crates from git. To build against a checkout next door instead, let
the CLI write and verify the patch table:

```sh
day patch --local /path/to/day
```

## Inside the code

- `src/lib.rs` sets up `root()` and the adaptive shell: a `selector` over cities and the
  per-city reactive weather store.
- `src/weather.rs` is the data layer: the Open-Meteo fetch through `day-part-http`, WMO
  weather-code mapping, and the mock fixtures.
- `src/ui.rs` is the weather screen: the sky-gradient backdrop, the hero, the hourly strip, the
  ten-day `grid` with its range-bar column, and the detail-card grid.
- `src/settings.rs` holds the Settings form and the persisted preferences.
- `src/icons.rs` draws every weather glyph as a group of shapes, so there are no image assets.
- `resource/locales/` carries the Fluent strings for `en`, `fr`, `ar`, and `zh-CN`.
- `platform/` holds the thin native host projects the mobile targets build through.

`day lint` checks routes, element ids, and locale coverage.

Weather data by [Open-Meteo.com](https://open-meteo.com) (CC BY 4.0). Day Skies is open source
under the Apache-2.0 license.
