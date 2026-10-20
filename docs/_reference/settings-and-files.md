---
title: Settings & Files
description: Configuration, credential and cache locations, and what is safe to delete.
nav_order: 0
---

## Where things live

Spotidark keeps its own files, separate from Spotidark's. It never reads,
imports or removes Spotidark's settings or sign-ins. Its directories follow
each platform's conventions for the `engineering` / `darkroom` / `spotidark`
identity (`src/paths.rs`):

| Directory | Linux | macOS | Windows |
| --- | --- | --- | --- |
| Config | `~/.config/spotidark/` | `~/Library/Application Support/engineering.darkroom.spotidark/` | `%APPDATA%\darkroom\spotidark\config\` |
| State | `~/.local/state/spotidark/` | `~/Library/Application Support/engineering.darkroom.spotidark/` | `%LOCALAPPDATA%\darkroom\spotidark\data\` |
| Cache | `~/.cache/spotidark/` | `~/Library/Caches/engineering.darkroom.spotidark/` | `%LOCALAPPDATA%\darkroom\spotidark\cache\` |

On Linux, `XDG_CONFIG_HOME`, `XDG_STATE_HOME` and `XDG_CACHE_HOME` move these
directories when they are set. On macOS, config and state share one folder.

| What | Where | Safe to delete? |
| --- | --- | --- |
| Settings | Config: `settings.json` | Yes, you lose preferences |
| Custom themes | Config: `themes/` | Yes, you lose your palettes |
| Winamp skins | Config: `skins/` | Yes, you add them again |
| MilkDrop presets | Config: `milkdrop/` | Yes, you fetch them again |
| Spotify grants | System credential store | Use Sign out in Settings |
| Proxy password | System credential store | Clear the password and apply the manual proxy settings |
| Credential revocation markers | State: `credential-storage/` | Keep after a failed sign-out deletion |
| Last session | State: `session.json` | Yes |
| Play history | State: `history.json` | Yes |
| Last run's log | State: `spotidark.log` | Always |
| Crash log | State: `panic.log` | Always |
| Legacy Web API grants | State: `shared_web_api_token.json`, `personal_web_api_token.json`, `web_api_token.json` | Removed after migration or sign-out |
| Legacy playback credential | State: `credentials/` | Removed after migration or sign-out |
| Legacy proxy password | State: `proxy_password` | Removed after protected migration |
| Audio cache | Cache: `audio/` | Always |
| Artwork cache | Cache: `art/` | Always |
| Lyrics cache | Cache: `lyrics/` | Always |
| Account-scoped playlist page cache | Cache: `playlists/<account-id>/` | Always |
| Liked Songs metadata | Cache: `liked-songs/` | Always |

Clearing caches never signs you out. Sign-out from Settings covers the shared
and personal Web API grants and the independent playback credential.

Durable grants use **Secret Service on Linux**, **Keychain on macOS**, and
**Credential Manager on Windows**, under the service name
`engineering.darkroom.spotidark`. Entries are separated by application state
location and grant type; Web grants carry their Client ID and must verify as
the same account. Playback and receiver activation require that account too.
Non-secret settings and session data remain readable JSON. Native credential
protection reduces exposure from copying ordinary application files. It does
not protect a usable session from arbitrary code running as the same user;
Linux protection also depends on the desktop keyring's configuration.

On Linux, enable and unlock a Secret Service provider such as GNOME Keyring or
KWallet to remember a new sign-in. An unavailable or locked store
produces an error without blocking the interface. A new sign-in can still be
used for this session, with no new plaintext fallback file.

Each legacy grant found in the state directory is written to the protected store and read back
before its old file is removed. Valid grants migrate without signing in again.
If Spotify rejects a saved refresh grant, only that grant is forgotten so the
next launch cannot keep restoring it. If migration fails, Spotidark reports it and
keeps the original so migration can be retried. That grant can still serve the
current session. A successfully migrated grant is never replaced by a stale
legacy copy. Librespot's reusable grant stays in memory until Spotidark saves
it through this same store. Volume and disposable audio caches are independent.

Sign-out invalidates pending authorization, refresh, and playback connections,
and cancels pending Spotify requests so their results cannot undo a new sign-in.
It records non-secret revocation markers before deleting the protected entries
and all legacy token files, including temporary copies. A locked store or
filesystem failure is reported. Revocation markers prevent a failed protected
entry deletion from restoring the session on restart; keep these markers when
a deletion failed. Removing or changing a personal Client ID clears that app's
old grant.

The legacy file names come from older Spotidark releases, which stored grants
unencrypted. Spotidark never writes new plaintext grants. Keep any legacy files, their temporary copies, the
`credentials/` directory, and credential-store exports out of issue attachments
and diagnostic uploads.

Proxy passwords use a separate protected entry under the same service name.
They belong to the configured host, port, and username. Editing any of these
fields clears the old password; changing only HTTP/SOCKS5 mode keeps it. Off,
System, and Spotify sign-out retain the saved manual proxy password. Clear the
password and apply the manual settings to forget it. A revocation marker keeps
a failed protected deletion from restoring that password.

Older proxy passwords, whether in `settings.json` or the separate state file,
are migrated and read back before their plaintext copies are removed. If that
fails, the originals remain available for retry. Settings changes stay in
memory until migration succeeds, so a save cannot erase the only password or
associate it with a different proxy address. Spotidark reports this condition.
Newly entered passwords have no plaintext fallback. Settings store the confirmed
proxy mode, host, port, and username, never an unapplied draft or password.

Progress through a playlist is periodically cached as a contiguous prefix.
When the playlist has not changed on Spotify, reopening it resumes from that
prefix instead of requesting the same pages again. Spotidark validates the
cache against Spotify's playlist snapshot and reported song count before
showing it. A cache with a mismatched count is replaced by live rows even if
its snapshot matches, so stale cached songs cannot choose the playback order.
Successful playlist edits keep that loaded prefix and save it under Spotify's
new snapshot after all pending writes have succeeded. Pending edits remain
visible immediately, but are not saved as confirmed playlist rows. A failed
write reloads the playlist to reconcile the edit.

Playlist checkpoints stream their JSON to a temporary
file on a background file worker, so saving a large playlist does not need a
second complete JSON buffer in memory. A failed write leaves the previous cache
in place.

Reading a playlist cache also uses
a small buffer on a background file worker, so the full JSON file does not
stay in memory alongside the loaded songs. Missing or invalid caches are
ignored and fetched again.

The artwork loader shares downloaded image bytes with
the background cache writer instead of making a separate copy. A failed cache write does
not prevent the downloaded image from being displayed.

Spotidark limits encoded artwork to 8 MiB per file or response, with four
concurrent transfers and two shared decoding jobs. Decoding rejects dimensions
above 8192 pixels and uses a 64 MiB decoder allocation allowance. That allowance
is a decoder safeguard, not a total process-memory limit. The 64 MiB
artwork retention budget accounts for encoded bytes, decoded pixels and the
estimated texture size together. Visible artwork remains protected. Oversized
or invalid artwork leaves a placeholder; it does not block playback.

Liked Songs metadata is stored separately under `liked-songs/` in the cache
directory, one JSON file per account. Only the verified account's rows are
shown. Fresh cached pages are reused for 15 minutes; older pages refresh in
the background. Refreshing keeps the last usable rows until their replacement
is ready, and a failed refresh leaves those rows visible. The refresh control
requests current data immediately. Partial caches resume from their next page.
Like and Unlike change the rows immediately, and confirmed edits survive a
restart even if Spotify's next read still reports the old state. This cache
contains metadata, not offline audio, and can be deleted without signing out.

The displayed Liked Songs list and save checkpoints share the
canonical song metadata. Cache reads and writes stream through small buffers
on background workers. Existing version 1 cache files remain readable, and a
failed write preserves the previous cache.

The last good playlist folder tree is kept in `session.json`, scoped to the
account that supplied it. This keeps folders visible when local playback is
temporarily unavailable. Live session data is still required for edit grants.

Memory caches retain the open page,
the playing context, and a limited set of recently used playlist, album,
artist, and show pages. Older pages reload when revisited, using the saved
playlist prefix when its snapshot still matches. Pending playlist edits and
their rows stay in memory until the write and its snapshot are confirmed,
even if this temporarily exceeds the usual page limit. Track metadata is
limited to 800 cached tracks; navigation and periodic cleanup trim old entries.

Spotidark also evicts inactive catalogue pages toward 6,000 retained
rows and 64 MiB of estimated metadata. Detached cached windows within a list
are trimmed toward 1,200 rows and 16 MiB. These are retention targets, not hard
RAM caps: the open page, playing context, selected rows and pending edits stay
available even when they exceed a target. Revisiting an evicted page reloads
it. Table views borrow their source metadata and retain only one derived
sorting/filtering cache. Secondary artwork colors and metadata caches are
bounded separately.

## Interface preferences

Open Settings from the account menu. Playback and Appearance remain visible;
specialist sections expand on demand. Settings search reveals matching
controls, and requested personal-app setup or invalid input opens its section.
Search keeps its query field and a compact filter selector available when side
panels leave too little room for the normal header and filter row.

**Appearance → Motion** stores `system`, `full` or `reduced` in the `motion`
setting. The default is `system`. macOS and Windows use their
native accessibility preference; Linux users can select `reduced` explicitly.
Reduced motion removes scaling and animated travel, including lyric following
and automatic scrolling, while retaining color and opacity feedback.

The session remembers separate positions for the main window and the Winamp
mini player. The shade modes are kept in `settings.json`. Wayland compositors
may ignore saved positions. On Windows, a position
whose title bar is no longer on an available monitor's work area is discarded
when reopening the window, keeping its initial on-screen placement instead.

A main window left maximized or full
screen reopens that way, and comes back that way from the mini player. The
remembered size and position describe an ordinary window and are not applied
to one that already fills the screen, because sizing or moving such a window
restores it down.

Album and playlist scrollbars reserve the full track count
as soon as Spotify reports it. Dragging to an unloaded section shows placeholders and requests
that section directly. Loaded windows stay in memory while the page is retained;
returning to one does not download it again. Unavailable entries keep their row
positions. Playlist edits and refreshes invalidate other cached windows because
their server positions may have changed. Only contiguous playlist prefixes are
saved on disk.

Large playlist pages also have a **Go to song** control. Entering a song
number scrolls to that row and loads its 50-item page if needed. Filtering or
sorting returns to the beginning and loads remaining pages as needed, since
local search and ordering require the track metadata. A failed window stops
automatic requests and shows a Retry button in the reserved row space.

## settings.json

Settings are stored in one readable JSON file and written atomically. Missing
fields take their defaults. The fields are:

| Field | Default | Meaning |
| --- | --- | --- |
| `device_name` | `Spotidark` | Name on Spotify Connect |
| `bitrate` | `320` | 96, 160, or 320 kbps |
| `normalisation` | `false` | Volume normalisation |
| `autoplay` | `true` | Keep playing similar music at the end |
| `gapless` | `true` | Gapless playback |
| `audio_backend` | platform | `pulseaudio` or `rodio` on Linux |
| `audio_device` | none | Output device name; none or blank uses the system default |
| `audio_buffer_ms` | `100` | Windows output buffer in milliseconds, 20 to 500; smaller values may click, larger ones delay controls |
| `audio_cache` | `true` | Keep recently played audio on disk |
| `audio_cache_mb` | `1024` | On-disk audio cache budget |
| `volume` | `45874` | Last local volume, 0 to 65535 (70%) |
| `theme` | `system` | Follow the system appearance by default; explicit `dark` and `light` choices remain available |
| `motion` | `system` | `system`, `full`, or `reduced` interface motion |
| `custom_theme` | `null` | Selected JSON filename from the `themes` folder |
| `custom_theme_cache` | absent | Last accepted custom palette; preserves appearance if its file is missing or invalid |
| `system_theme_cache` | absent | Last accepted Omarchy palette for Follow system; retained across restarts |
| `accent_from_art` | `true` | Tint pages with album art |
| `home` | `{}` | Home shelf visibility; see [Home shelves](#home-shelves) |
| `zoom` | `1.0` | Interface zoom, 0.5 to 2.5; Ctrl+plus and Ctrl+minus change it |
| `show_shortcut_hints` | `true` | Stored preference that the interface does not currently read |
| `search_history` | `[]` | Recent search queries |
| `library_sort` | `{}` | Per-section Library order overrides: `library`, `recently_played`, `name`, `recently_added`, `local`, or `spotify`, where supported |
| `sidebar_order` | `[]` | Saved local playlist arrangement, including an unpinned Liked Songs, retained when another sort is selected |
| `pinned_contexts` | `[]` | Local Library pin order; Liked Songs uses `fastpotify:liked-songs`, a local key never sent to Spotify |
| `liked_songs_pinned` | `true` | Keep Liked Songs in the pin block; older settings place it first until moved |
| `sidebar_visible` | `true` | The Library sidebar is visible |
| `sidebar_compact` | `false` | Names only in the library sidebar, no covers |
| `sidebar_width` | `250` | Library sidebar width in points |
| `art_expanded` | `false` | The playing album's art is docked large at the bottom of the sidebar |
| `queue_width` | `360` | Queue panel width in points |
| `lyrics_width` | `360` | Lyrics panel width in points |
| `tracklist_compact` | `false` | One-line track rows without covers |
| `winamp_window` | `false` | The window is the Winamp mini player |
| `winamp_show_taskbar` | `true` | Windows only: show the Winamp window's taskbar button; the main window always keeps its button |
| `skin` | none | File or folder name in the skins folder; blank uses the built-in skin |
| `skin_scale` | by display | Screen pixels per skin pixel, 1 to 4 |
| `winamp_on_top` | `false` | Keep the mini player above other windows |
| `vis` | `bars` | The mini player's visualiser: `bars`, `scope`, or `off` |
| `playlist_open` | `false` | The playlist window is open under the mini player |
| `playlist_height` | `174` | The playlist window's height in skin pixels |
| `eq_open` | `false` | The equalizer window is open under the mini player |
| `eq_on` | `false` | The equalizer shapes local playback |
| `eq_preamp_db` | `0` | The preamp, in decibels, -12 to 12 |
| `eq_bands_db` | ten zeros | The bands from 60 Hz to 16 kHz, in decibels, -12 to 12 |
| `balance` | `0` | Left to right, -1 to 1, for local playback |
| `mono` | `false` | Play both channels the same |
| `playlist_shaded` | `false` | The playlist window is rolled up to its title bar |
| `eq_shaded` | `false` | The equalizer window is rolled up to its title bar |
| `winamp_shaded` | `false` | The main window is rolled up to its title bar |
| `milkdrop_open` | `false` | The MilkDrop window is open |
| `milkdrop_seconds` | `10` | How long each MilkDrop preset plays |
| `milkdrop_fps` | `60` | MilkDrop frame rate; `0` is uncapped |
| `milkdrop_screen_hz` | `0` | Last reported display refresh rate |
| `milkdrop_scale` | `1` | MilkDrop picture resolution: `1` full, `2` half, `4` quarter |
| `milkdrop_fullscreen` | `false` | The MilkDrop window fills the screen |
| `milkdrop_size` | `640, 480` | The MilkDrop window's size in points |
| `keep_playing_in_background` | `true` | Close to tray |
| `check_for_updates` | `true` | Ask GitHub once an hour for a newer release |
| `download_updates_automatically` | `true` | Download verified updates in the background; installation waits for Restart to update |
| `web_client_id` | none | Optional personal Spotify app id used alongside shared coverage |
| `personal_app_nudge_at` | none | Legacy daily-reminder timestamp; not used for new reminders |
| `personal_app_intro_seen` | `false` | Whether the Premium personal-app introduction was dismissed or followed |
| `playback_authorized` | `false` | Local playback was authorized on this machine, so it resumes without prompting |
| `proxy_mode` | `system` | `off`, `system`, `http`, or `socks`. Older files without this field stay on `system` |
| `proxy_host` | none | Host of a manual HTTP or SOCKS5 proxy. Ignored when the mode is `off` or `system` |
| `proxy_port` | none | Port of a manual HTTP or SOCKS5 proxy |
| `proxy_username` | none | Optional proxy login for Web requests; authenticated local playback proxying is not supported |

## Command line

```
spotidark [OPTIONS] [LINK]
spotidark <COMMAND>

  LINK                  A Spotify link to open: spotify:track:…, or an
                        open.spotify.com address
  --device-name <NAME>  Spotify Connect name for this session
  -v, --verbose         More logs from librespot and the Web API client
  -h, --help            Print help
  -V, --version         Print the version
```

A link goes to the running Spotidark when there is one, which then opens
the page and brings its window forward; otherwise the app starts on it. The
desktop's handler for `spotify:` links runs exactly this. Spotify search links
(`https://open.spotify.com/search/…` or `spotify:search:…`) open Search with
the query filled in.

A command controls the running app instead of starting it, then exits. It
prints an error and exits with status 1 when Spotidark is not running:

| Command | Effect |
| --- | --- |
| `play-pause` | Toggle play and pause |
| `play` | Start playback if paused |
| `pause` | Pause playback if playing |
| `next` | Skip to the next track |
| `previous` | Return to the previous track |
| `seek <SECONDS>` | Seek by this many seconds; negative seeks backwards |
| `seek-to <SECONDS>` | Seek to a position, in seconds from the start |
| `volume <PERCENT>` | Set the volume, 0 to 100 |
| `volume-up [PERCENT]` | Raise the volume, by 10 unless given (1 to 100) |
| `volume-down [PERCENT]` | Lower the volume, by 10 unless given (1 to 100) |
| `mute` | Toggle mute |
| `shuffle [on\|off]` | Toggle shuffle, or set it |
| `repeat [off\|context\|track]` | Cycle the repeat mode, or set it |
| `like` | Save the playing track to your library, or remove it |
| `play-uri <URI>` | Play a Spotify URI: a track, album, playlist, artist, or show |
| `devices [--raw]` | List Spotify Connect devices, one per line with the id first; `--raw` prints the JSON instead |
| `transfer <DEVICE_ID>` | Move playback to a device, by the id `devices` prints |
| `now-playing [--raw]` | Print the playing track; `--raw` prints tab-separated fields: state, title, artists, album, position_ms, duration_ms, volume, shuffle, repeat, art_url, saved, device |
| `show` | Bring the running app's window forward |
| `reload-themes` | Reload local palette files without starting the app or interrupting playback |

On Linux, only `reload-themes` works this way. The other commands exit with
status 2 and point to MPRIS, which the running app provides under the player
name `spotidark`, for example `playerctl --player=spotidark play-pause`.

The `spotifast` and `fastpotify` compatibility commands accept the same
options and commands.

Attach `spotidark.log` from the state directory to bug reports. It contains
the last run's output, including extra lines from `spotidark -v`. After a
crash, attach `panic.log` too.

## Demo mode

Builds made with `cargo build --features demo` accept `--demo`, which loads
sample data for screenshots and interface work. Demo mode never writes
settings.

`--demo-page` opens a page, such as `home`, `playlist:pl1`, or `artist:art0`,
and `--demo-show` adds surfaces on top of it as a comma-separated list. Unknown
names are ignored. The accepted names (`src/demo.rs`) are:

| Area | Names |
| --- | --- |
| Queue and Recent | `queue`, `playing-next`, `queue-empty`, `queue-loading`, `queue-error`, `recents`, `recents-empty`, `recents-loading`, `recents-error`, `resume`, `resume-next` |
| Lyrics | `lyrics`, `lyrics-fullscreen`, `lyrics-follow`, `lyrics-empty`, `lyrics-loading`, `lyrics-error`, `lyrics-instrumental`, `lyrics-no-playback` |
| Dialogs | `shortcuts`, `premium`, `create`, `edit`, `duplicate`, `personal-app`, `personal-app-invalid` |
| Devices and updates | `devices`, `many-devices`, `update` |
| Library and layout | `focus`, `compact`, `art`, `folders`, `pins`, `sorted`, `finite-playlist`, `scripts`, `light`, `german`, `faster`, `windows-taskbar` |
| Winamp and MilkDrop | `winamp`, `playlist`, `shade`, `playlist-shade`, `eq`, `eq-shade`, `presets`, `small`, `milkdrop` |

`update` shows a sample update badge for checking its layout.
`personal-app` shows the personal Spotify app introduction.
`personal-app-invalid` opens Settings with an invalid Client ID, its inline hint,
and a disabled Authorize button. Use it with both themes and window sizes when
reviewing the [personal app setup flow](https://github.com/darkroomengineering/spotidark/blob/main/docs/spotify-app.md).
`--demo-language <LOCALE>` previews a translation; see
[Translating Spotidark](/translating/) for the locales.

`--demo-shot <PATH>` writes the window to a PNG and exits, which is useful for
making deterministic screenshots for these pages:

```
cargo run --release --features demo --bin spotidark -- \
  --demo-shot docs/screenshot.png --demo-page playlist:pl1 --demo-show queue
```

The image uses the current window size. `--demo-size WIDTHxHEIGHT` sets that
size in logical pixels for a shot (for example `760x800` or `1240x800`).
`--demo-shot-delay <MS>` sets how long to wait for cover art before taking it
(6000 by default). Demo windows ignore saved window geometry and do not read or
save the normal window's framework state. Built-in appearance settings still
apply. `--demo-data <DIRECTORY>` keeps demo caches and logs under
that directory's `cache` and `state` folders, with settings read from `config`.
Custom palettes and their cache are used only with an explicit `--demo-data`
directory; the ordinary demo does not scan your real themes folder.

## Home shelves

You can hide **Made for you** and **Recommended for you**
from Home independently. Quit Spotidark before editing `settings.json`, then
restart it. Add this field to hide both:

```json
"home": {
  "made_for_you": { "visible": false },
  "recommendations": { "visible": false }
}
```

Set either `visible` value to `true` to show that shelf again. Omitted
preferences keep both shelves visible. Other Home sections keep their normal
order and contents. This changes what is displayed; hidden shelves still
refresh in the background.

## Custom themes

Create a `themes` folder beside `settings.json` and put JSON files in it.
Run `spotidark reload-themes` if the app is already open, then select the
filename under **Settings → Appearance → Theme**.
The default is **Follow system**. It uses your desktop’s light/dark appearance,
or the current Omarchy palette when the Omarchy hook is installed. Saved Dark,
Light and custom choices are preserved when updating. The picker starts with
**Follow system**, **Light**, and **Dark**, then a separator. **Omarchy** comes
next when the integration is available, followed by the other local palettes.
Themes change colors and keep the app's existing fonts.
The **Open themes folder** icon button beside the picker creates the folder if
needed and opens it in your file
manager, using the same button style as the Winamp skins folder.
After adding or editing a JSON file, run `spotidark reload-themes` to refresh
the list and the selected palette without restarting playback.
Choosing a built-in theme clears the custom selection.

For example, `themes/gruvbox.json`:

```json
{
  "base": "dark",
  "colors": {
    "window": "#282828",
    "panel": "#1d2021",
    "surface": "#32302f",
    "text": "#ebdbb2",
    "accent": "#b8bb26"
  }
}
```

`base` is `dark` (the default) or `light`. Omitted colors inherit that palette.
Supported colors are `window`, `panel`, `surface`, `surface_hover`,
`surface_active`, `outline`, `text`, `secondary`, `dim`, `accent`,
`accent_hover`, `on_accent`, `danger`, `warning`, `overlay`, and `shadow`.
Values must be `#RRGGBB` or `#RRGGBBAA`.

Files are read in the background at launch and when `spotidark reload-themes`
is called. The command updates the selected palette without interrupting
playback, changing your selection or showing the window. It does not start a
stopped app. Repeated requests are combined while a scan is running, and no
continuous file watcher or polling timer is added. Use regular UTF-8 `.json` files, not symbolic links or
subdirectories. Each file is limited to 64 KiB. Keep at most 128 JSON files and
512 total entries in the themes folder; the saved selection is still checked
when a folder exceeds these limits.

Invalid files are skipped with a warning in the log. Spotidark remembers the
last accepted custom palette in `settings.json`. If the selected file is
removed or becomes invalid, that appearance stays in place, including after
a restart, and the Theme row explains the problem. Other preferences are
preserved. A custom selection without any usable cached colors uses the
built-in choice. Choosing Dark, Light or Follow system clears the custom
selection and its cache. Editing or deleting the optional cache does not
reset unrelated settings.

A custom palette's `base` controls both its inherited colors and the light or
dark styling of standard controls. Album-art tinting remains an independent
setting; turn it off for fixed colors throughout. Palettes apply to the main
window; Winamp skins remain separate. This first format controls colors only.

### Follow an Omarchy theme

Spotidark follows an Omarchy desktop theme through a template and a
theme-change hook in the repository:
[`contrib/omarchy/spotidark.json.tpl`](https://github.com/darkroomengineering/spotidark/blob/main/contrib/omarchy/spotidark.json.tpl)
and
[`contrib/omarchy/spotidark-theme`](https://github.com/darkroomengineering/spotidark/blob/main/contrib/omarchy/spotidark-theme).
Omarchy resolves its light/dark mode and colors through its
[template system](https://omarchy.org/manual/making-your-own-theme/).
The hook copies the result atomically into `themes/omarchy.json`, then asks a
running Spotidark to reload it. It does not change your desktop theme or your
Spotidark selection, and it never starts a stopped app.

Install the two files from a checkout:

```sh
mkdir -p ~/.config/omarchy/themed
install -m 644 contrib/omarchy/spotidark.json.tpl ~/.config/omarchy/themed/
omarchy hook install theme-set contrib/omarchy/spotidark-theme
```

Apply a theme through Omarchy's theme picker, then choose **Follow system** or
**Omarchy** in Spotidark's **Settings → Appearance → Theme**. Later Omarchy
changes update that palette while music keeps playing. Turn off album-art
tinting if every page should keep the theme's fixed colors.

The hook reads `spotidark.json` from Omarchy's current theme at
`~/.local/state/omarchy/current/theme` and writes to
`${XDG_CONFIG_HOME:-~/.config}/spotidark/themes`. Set `SPOTIDARK_THEMES_DIR` in
the installed hook to use another themes directory. The hook calls the native
`spotidark` command. Missing or invalid palettes leave the last accepted
appearance in place.

When Spotidark is installed with its Omarchy assets in
`share/spotidark/omarchy` beside its `bin` directory, the first launch on an
Omarchy desktop installs the template and hook in your user configuration
itself, and prepares a palette for the current theme. It never replaces an
existing template, hook or palette, and never changes your selected theme.
Cargo builds and demo mode do not do this.

To stop following Omarchy, choose Dark, Light or another custom theme in
Spotidark. To remove the integration, delete
`~/.config/omarchy/hooks/theme-set.d/spotidark-theme` and
`~/.config/omarchy/themed/spotidark.json.tpl`. Other hooks remain in place.
