# Day Skies — "Sun & Cloud" icon

Master art: `day-icon.svg` — a warm sun peeking behind a soft cumulus cloud on a day-sky gradient.
Sky `#2F80F0 → #8FC4FF`, sun `#FDBE4A` (amber, echoing the Day sunrise mark), cloud
`#FFFFFF → #E4EFFB`. Full-bleed square; each platform applies its own mask/shape. Its top-level
`day:background` / `day:foreground*` ids drive the Android adaptive split, the iOS layered icon
and the monochrome variant.

Nothing derived is checked in. `day prepare` (run by every `day build`, and by the VS Code
extension before it opens a host project) renders the master into `build/day/host/`, and the
checked-in Xcode, Gradle and hvigor projects read from there — see `docs/icons.md` in the `day`
repository for the layout, and `day prepare --check` for the CI drift gate. Edit the master and
build; there is no export step to remember.

## Hand-drawn variants

These are the artist's per-platform renderings of the same motif, kept for reference and
re-export. The pipeline does not read them. An override is a master for one family, full-bleed
square art that `day prepare` shapes the way it shapes `day-icon.svg`: copied to
`resource/icons/<family>.svg` (`ios.svg`, `android.svg`, …), the iOS and Android variants
below would qualify. The macOS variant already carries the rounded body and margin the
pipeline adds itself, so it stays a reference drawing.

- `ios/day-icon-ios.svg` — the motif as a full-bleed square, the shape iOS masks itself.
- `macos/day-icon-macos.svg` — the motif in Apple's rounded body with the transparent margin
  (824 pt art on a 1024 canvas).
- `android/ic_launcher_foreground.svg` — the sun and cloud inside the 66 dp safe zone of the
  108 dp adaptive canvas, transparent background.
- `android/ic_launcher_background.svg` — the sky gradient alone.
- `android/ic_launcher_monochrome.png` — a raster of the themed-icon silhouette; `day prepare`
  now derives a vector monochrome from the master's foreground instead.
