---
title: Download
description: Download Spotidark for macOS or Windows, or build it from source.
nav_order: 1
---

Spotidark publishes two downloads on its
[latest release](https://github.com/darkroomengineering/spotidark/releases/latest).
Every successful build of `main` replaces the previous release, so this link
always points at the current version. Installed copies update themselves; see
[releases and automatic updates](https://github.com/darkroomengineering/spotidark/blob/main/docs/releases.md).

Spotidark keeps its own settings and sign-ins, separate from Spotifast. Sign in
again when switching from Spotifast. Homebrew, AUR, Flatpak and other Spotifast
packages install the upstream app, not Spotidark.

## macOS

Download `spotidark-macos.dmg` from the
[latest release](https://github.com/darkroomengineering/spotidark/releases/latest).
It is one universal app for Apple silicon and Intel Macs.

Open the downloaded file and drag **Spotidark** to **Applications**. Move it out
of the disk image before opening it, so automatic updates can replace it.

### First open on macOS

The Mac download is currently ad-hoc signed and not notarized by Apple, so
macOS may block the first launch. The release notes state the signing mode of
each build. If macOS refuses to open the app, open **System Settings → Privacy &
Security** and choose **Open Anyway** for Spotidark.

## Windows

Download and run `spotidark-windows.exe` from the
[latest release](https://github.com/darkroomengineering/spotidark/releases/latest).
It is an installer for 64-bit Intel and AMD PCs. It installs for the current
user, adds Spotidark to the Start menu, and does not need an administrator
password.

The installer is not Authenticode signed, so SmartScreen may warn about an
unknown publisher on first run; choose **More info**, then **Run anyway**.

To choose which app opens Spotify links, use **Settings → Apps → Default apps**
in Windows.

## Linux and building from source

There is no Linux download. Build Spotidark from source:

```sh
git clone https://github.com/darkroomengineering/spotidark.git
cd spotidark
cargo run --locked --release --bin spotidark
```

The [README](https://github.com/darkroomengineering/spotidark#build-and-run)
lists the build requirements for each system. On Linux, install
`packaging/applications/spotidark.desktop` and `packaging/icons/spotidark.svg`
to add the app to your launcher and let it open `spotify:` links. The
`spotidark` command must be on your PATH. Source builds do not update
themselves.
