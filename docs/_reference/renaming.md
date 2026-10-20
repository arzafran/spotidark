---
title: Fork Identity
description: The names Spotidark uses for its command, files, credentials and desktop integration, and the upstream names it keeps.
---

Spotidark is Darkroom Engineering's fork of
[Spotifast](https://github.com/crmne/spotifast) by Carmine Paolino and
contributors, under the MIT license. It has its own app identity, so it does
not share settings, sign-ins or desktop integration with Spotifast and can be
installed beside it. Spotidark does not import or clear Spotifast's data; sign
in again after switching.

## Spotidark names

| What | Name | Defined in |
| --- | --- | --- |
| Product | Spotidark | `README.md` |
| Command | `spotidark` | `Cargo.toml` (`[[bin]]`) |
| Settings, state and cache directories | `ProjectDirs` for `engineering` / `darkroom` / `spotidark` | `src/paths.rs` |
| Log file | `spotidark.log` in the state directory | `src/paths.rs` |
| Credential-store service | `engineering.darkroom.spotidark` | `src/credentials.rs` |
| macOS bundle ID | `engineering.darkroom.spotidark` | `packaging/macos/Info.plist` |
| Linux desktop entry | `spotidark.desktop`, `Icon=spotidark`, `StartupWMClass=spotidark` | `packaging/applications/spotidark.desktop` |
| MPRIS player name | `spotidark` (`playerctl --player=spotidark`) | `src/mpris.rs` |
| Default Spotify Connect name | `Spotidark` | `src/settings.rs` |
| Main window state | `app.ron` in eframe's `spotidark` storage directory | `src/entrypoint.rs` |
| Omarchy hook and template | `spotidark-theme`, `spotidark.json.tpl` | `contrib/omarchy/`, `src/theme/omarchy.rs` |
| Windows installer | `spotidark-windows.exe`, installed to `Programs\Spotidark` for the current user | `packaging/windows/fastpotify.iss` |

[Settings & Files](/settings-and-files/) lists the full paths on each system.

## Upstream names kept on purpose

| What | Name | Why |
| --- | --- | --- |
| Rust package and library | `fastpotify` | Keeps upstream merges straightforward; see `README.md` |
| Compatibility commands | `spotifast`, `fastpotify` | Build from `src/spotifast.rs` and `src/main.rs`; they start Spotidark with the same storage and controls, and are not needed to run it |
| macOS bundle executable | `fastpotify` inside `Spotidark.app` | `packaging/macos/Info.plist` |
| Windows installer script | `packaging/windows/fastpotify.iss` | File name only; it builds the Spotidark installer |
| Local Liked Songs pin key | `fastpotify:liked-songs` | Stored in `settings.json`, never sent to Spotify; `src/settings.rs` |
| Update staging directory | `.fastpotify-update-*` beside the app | `src/updates/install.rs` |
| Translation template | `assets/i18n/fastpotify.pot` | Source template for every PO file |

The inherited Homebrew, AUR, Flatpak and Linux package recipes still use
Spotifast names. They are upstream reference material and do not publish
Spotidark; see [Release packaging](/packaging/).
