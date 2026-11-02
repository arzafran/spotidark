## Functional DAG

```text
Inputs                      Investigation / implementation      Integration       Verification
7653527 + deterministic UI ─> Baseline memory / frame timings ─┐
Artwork and view lifetime ──> Sized decoding / hidden cleanup ─┼─> Combined build ─> Checks + captures + measurements
Tray and backend events ────> Wake notifications / deadlines ──┤
Scrolling and text layout ──> Profile and fix measured costs ──┘
```

# Client optimization follow-up

Baseline: `7653527`. The maintainer authorized artwork sizing, releasing cold
graphics when hidden, event-driven hidden-window work, and profiling scrolling
and text layout before reconsidering a UI migration. The Rust/egui frontend and
all playback features remain in scope; a stack migration is not part of this work.

## Verification contract

- Preserve Retina sharpness, larger artwork views, decoder limits, and cache budgets.
- Preserve playback, media/tray commands, background deadlines, and optimistic UI.
- Keep native platform integrations target-specific.
- Compare deterministic baseline/candidate workloads; distinguish headless layout,
  allocation counts, process memory, and real rendered frame pacing.
- Run focused regressions and the full contribution checks before committing.
- Record platform and live-playback coverage honestly.

## Work record

- Baseline source archived in `/tmp/spotidark-opt2/baseline`; the release demo
  binary is `/tmp/spotidark-opt2/spotidark-before`.
- The former image loader ignored its display-size hint. Display decoding now
  uses physical-pixel buckets and upgrades existing textures in place.
- The macOS menu callback and asynchronous image work no longer retain retired
  window contexts. Hidden cleanup releases decoded artwork and the lyrics backdrop.
- Hidden waiting now uses sticky event notifications and eligible background
  deadlines. Native callbacks remain serviceable during continuous worker wakes.
- Collection tables access canonical rows by index and reuse cached playlist
  position mappings instead of rebuilding full projections each frame.
- Focused regressions, compilation, Clippy and independent review passed.
- The final full contribution checks passed after the duration correction:
  launchers, formatting, default/all-feature Clippy, default/all-feature tests,
  doctests, Rust documentation, Jekyll and the optimized demo build.
- Twelve matching Home, Playlist and Settings captures, in both themes and two
  window sizes, are pixel-identical to the baseline. Native execution was macOS;
  real Spotify playback and other-platform runtime were not exercised locally.
- The existing credential-store round-trip test remains ignored. No new tests
  were skipped and no assertions were weakened to make the checks pass.

## Baseline measurements

Three macOS release demo runs on Apple M1 Max, warm artwork disk cache,
1280×800 logical points, eight-second screenshot delay: peak RSS 279.92,
278.28, and 278.83 MiB; median 278.83 MiB. This includes screenshot capture,
not real playback. Raw values are in `/tmp/spotidark-opt2/before/measurements.json`.

The scratch `frame_profile.rs` exercises the real headless `App::frame_ui`
with a demo playlist, 60 warmup frames and 600 measured frames per case.
Three processes per case test 40, 1,000 and 10,000 loaded rows. Allocation
counting and CPU timing run separately. Artwork/network decoding and GPU
rendering are excluded. Repeated demo song names deliberately keep text
cache behavior stable; this measures list scaling, not realistic multilingual
text shaping or display frame pacing.

At 10,000 rows, stationary frames allocate 2,485,475 requested bytes per frame;
the median of the three timing medians is 0.430 ms. At 40 rows those values
are 139,044 bytes and 0.095 ms. The source rebuilds full borrowed row and
position vectors despite virtualizing the rows it draws. This is the measured
target for the collection change. Requested allocation volume is not live RAM.

The first indexed-row candidate reduced the 10,000-row allocation volume to
126,461 requested bytes per frame, but still took 0.367 ms. A clean upstream
comparison exposed that remaining CPU cost. A separate paired probe retained
the same rows, total and source identity while bypassing the complete-playlist
duration subtitle. Across three processes, complete/bypassed medians were
0.328/0.089 ms at 10,000 rows and identical at 40 rows. This includes the subtitle's
small formatting/layout cost, so it is not a pure duration-function measurement.
The existing table cache now retains the aggregate by source revision. The
unchanged original fixture provides the final comparison below.

The scratch `art_profile.rs` loads 16 deterministic 640×640 PNGs from the
ordinary disk cache through `App::attach` and the registered loaders. At a
96-physical-pixel request, the baseline retains 25 MiB decoded pixels and
25 MiB texture payload. A subsequent 512-pixel request still uses the original
640-pixel images. After `window_gone`, detaching the waker and dropping the
context, 25 MiB of decoded pixels remain in the application's image loader.
Texture payload is counted from actual texture metadata, not process RSS or
driver allocations. The fixture waits for physical texture dimensions to
meet each request, so candidate measurements also verify resolution upgrades.

## Final fixture measurements

The same release fixtures ran against the corrected candidate, again using
three processes per frame case and separate timing/allocation runs.

| Loaded playlist rows | Baseline median frame, ms | Candidate, ms | Baseline requested bytes/frame | Candidate |
| ---: | ---: | ---: | ---: | ---: |
| 40 | 0.094584 | 0.093250 | 139,044 | 130,110 |
| 1,000 | 0.123292 | 0.090917 | 273,631 | 126,457 |
| 10,000 | 0.429917 | 0.088167 | 2,485,475 | 126,461 |

At 10,000 rows, headless frame time fell 79.5% and requested allocation volume
fell 94.9%. These are stationary, warm, synthetic playlist frames, not display
frame pacing or whole-app memory. Liked Songs still takes a snapshot of shared
row handles while drawing; this does not establish constant work for every
collection type.

The artwork fixture now retains 1 MiB decoded pixels and 1 MiB texture payload
for the 16 small covers, down from 25 MiB each. Its 96-pixel requests produce
128-pixel textures. Upgrading the same URLs to 512 pixels produces 512-pixel
textures and 16 MiB of each payload. Hidden decoded retention is zero, down
from 25 MiB. Temporary full-source decoding remains unchanged.

Three final process-memory runs with builds paused measured peak RSS of 273.69,
273.83 and 271.66 MiB, median 273.69 MiB. Compared with the 278.83 MiB baseline
median, that is 5.14 MiB (1.8%) lower in this small demo workload. It is not a
general RAM guarantee. An earlier candidate batch overlapped checks and measured
208.23, 272.42 and 272.20 MiB; the unusually low first value prompted the quiet
repeat. Both datasets are retained in the comparison artifact.

A separate `text_profile.rs` fixture lays out 30 fixed long labels through
`bidi::layout`, including truncation, with 60 warmup and 600 measured frames.
Across three processes, the median of medians is 0.005 ms for ASCII and
0.238 ms for mixed Arabic/Hebrew/Latin labels. These are warm headless layout
times, not first-use shaping or display frame times. The absolute cost does
not justify an additional text cache in this pass; the existing font/galley
caches and multilingual behavior remain intact.

## UI stack assessment

The measured collection issue comes from rebuilding complete row projections
inside the application. Its existing visible-row virtualization can be retained.
Artwork retention and hidden-window wake scheduling are also application-level
costs. Changing frameworks would not automatically correct those ownership and
scheduling decisions.

The [egui project](https://github.com/emilk/egui) describes its immediate-mode
tradeoff and recommends limiting layout to visible rows. A framework comparison
would need an equivalent player screen and matching workloads; these measurements
do not establish that egui is faster or slower than another frontend.

[Slint](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backends_and_renderers/)
offers native desktop backends and is a plausible candidate for a later isolated
frontend prototype. Its benefit here remains unmeasured. A platform-specific
frontend would also introduce separate UI maintenance for the other supported
platforms.

[Tauri uses system WebViews](https://v2.tauri.app/concept/process-model/).
That conflicts with this project's no-browser-engine boundary, so it is not a
candidate for this optimization pass.

Recommendation: keep Rust/egui. The measured costs were resolved within the
existing application, without a frontend migration. A later alternative should
be evaluated for a concrete remaining interaction or accessibility requirement
and benchmarked with equivalent data. No alternative frontend was measured here,
so these results do not support a framework performance ranking.
