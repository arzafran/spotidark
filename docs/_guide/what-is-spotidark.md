---
title: What is Spotidark?
description: What Spotidark offers, what you need to use it, and its current limits.
nav_order: 0
---

## Why Spotidark

**Spotify, native and fast.** Spotidark is a lightweight Spotify app for
macOS, Windows, and Linux by Darkroom Engineering. It opens in well under a second and typically uses
100–250 MB of memory, while Spotify's desktop app often uses 600 MB to over 1 GB.

Spotidark is a fork of [Spotifast](https://github.com/crmne/spotifast) by
Carmine Paolino and contributors, released under the MIT license. It has its
own settings, sign-ins and app identity, so it can be installed beside
Spotidark. Your Spotify playlists stay on your account.

**Playback needs Spotify Premium.** Free accounts can browse and search, but
cannot play music through Spotidark on this computer or another device.

![Spotidark Home with the playlist library, recommendations, queue, and player visible](/screenshot.png)

## What it does

- **Plays music on this computer.** Spotidark appears as a Spotify Connect
  device. Select it from your phone or play music in the app. Playback has
  no added gaps between songs and supports up to 320 kbps. Optional settings
  keep songs at a similar loudness and reduce repeat downloads by keeping
  recently played audio.
- **Controls other devices.** Move playback to a speaker, a phone, or
  another computer from the device picker, and keep controlling it: play,
  pause, skip, seek, shuffle, repeat, volume.
- **Library.** Browse playlists, Liked Songs, saved albums, followed artists,
  podcasts, and saved episodes. Create, edit, and reorder your playlists.
- **Search** across songs, artists, albums, playlists, podcasts, and
  episodes, with artist pages, discographies, and related artists.
- **Background playback.** Closing the window keeps the music playing from
  the system tray. Use your keyboard's media keys to play, pause, and skip.
- **Themes.** Choose light, dark, or your own colours. On Omarchy, Spotidark
  can follow your desktop theme automatically. Pages can also take a colour
  from album art.

<a id="account-safety"></a>

## Will my Spotify account get banned?

**We're not aware of any confirmed account bans caused by normal Premium
listening through Spotidark or other players using the same playback
software, [librespot](https://github.com/librespot-org/librespot).**

Spotidark plays music using your Spotify Premium subscription. It does not
unlock Premium for Free accounts, remove ads, export songs, or bypass
Spotify's copy protection. You sign in on Spotify's own website, and
Spotidark never receives your Spotify password.

Spotidark is independent of Spotify, so we cannot guarantee Spotify's future
decisions. Changes at Spotify can also temporarily interrupt playback until
the app is updated.

## What it does not do

Spotidark has a limited scope:

- **Playing needs Spotify Premium**, both on this computer and when
  controlling another device. Free accounts can browse and search.
- Setup asks for two Spotify approvals: one to access your library, another
  to play music on this computer. [How it connects](/how-it-connects/)
  explains why.
- **Spotify Lossless is not available.** Playback supports up to 320 kbps.
  The playback software Spotidark uses, librespot, cannot play Spotify's
  protected lossless audio. Spotidark will reconsider this if
  [librespot adds lawful support](https://github.com/librespot-org/librespot/issues/1583).
- No video podcasts or social features.
- Spotidark is an **unofficial** app. Changes at Spotify can temporarily
  break features until Spotidark is updated.

Bug reports should include `spotidark.log`, `panic.log` after a crash, and
steps to reproduce the problem. See the
[issue form](https://github.com/darkroomengineering/spotidark/issues/new/choose)
and [where the log is kept](/settings-and-files/#where-things-live). The log
records the app version, operating system, and graphics details. If the window fails to open, the log can
help explain why, even when you started the app from your desktop.

## Prior art

Spotidark is built on Spotifast and keeps its architecture. It is written in
Rust, with [egui](https://github.com/emilk/egui) for its interface and [librespot](https://github.com/librespot-org/librespot) for
Spotify playback. It takes inspiration from
[spotify-tui](https://github.com/Rigellute/spotify-tui),
[spotify-player](https://github.com/aome510/spotify-player),
[ncspot](https://github.com/hrkfdn/ncspot), and
[Omarchy Spotify](https://github.com/stappmus/Omarchy-Spotify).

Spotidark is an independent fork, not affiliated with or endorsed by
Spotify AB or the Spotifast team. Spotify is a trademark of Spotify AB.
