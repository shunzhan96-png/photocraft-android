# PhotoCraft Android mobile improvements

This patch focuses on Android font fallback, the mobile home/menu layout, localization starters,
credits, and removing the launch-time update dialog.

## Included

- Android font discovery now scans `/system/fonts`, `/system_ext/fonts`, `/product/fonts`,
  `/vendor/fonts`, and `/apex/com.android.art/fonts`. This lets the existing script fallback
  mechanism find fonts already installed by Android instead of searching only Unix/Linux paths.
- Added starter UI catalogs for Arabic (`ar`), Hindi (`hi`), Thai (`th`), Bengali (`bn`) and
  Filipino (`fil`). Common controls are translated; missing strings deliberately fall back to
  English. These catalogs are not yet full translations and remain marked incomplete.
- Arabic is already recognized as right-to-left by the existing direction system.
- The touch menu uses 48 dp rows and moves to a side sheet at a narrower tablet breakpoint.
- The home screen title scales down on narrow phones, and Recent rows now fit the available card
  width instead of remaining fixed at 380 dp.
- Added ZedissP to the contributor display list and linked the Android README to
  `https://github.com/zedisspp`.
- Update checks are manual-only. The startup check and its preference row were removed; Help >
  Check for Updates remains available.

## Follow-up / device validation

- Canvas navigation now consumes egui's multi-touch gesture data: pinch zoom is anchored at the
  gesture centre, two-finger translation pans the view, and the active canvas tool is suppressed
  while two or more touches are active. If a live stroke was already underway, it is ended at its
  last valid point before navigation takes over. Test this on physical Android devices; the exact
  pointer stream depends on the GameActivity/winit and touchscreen implementation.
- Arabic, Hindi, Thai, Bengali and Filipino are starter catalogs, not full translations. Missing
  labels intentionally fall back to English. Full coverage needs native-speaker review and the
  existing localization tests.
- The on-screen keyboard/insets behaviour and all layouts still need validation on real phones,
  landscape, tablets and different Android font configurations.

## Validation

Source-level checks should be run before distribution. This environment does not have Cargo/Rust
installed, so Rust formatting, unit tests, Android cross-compilation and APK installation could not
be run here. Build and test with the instructions in `apps/photocraft-android/README.md` before
distributing an APK.
