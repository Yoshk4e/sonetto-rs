# Sonetto-rs

### Current supported version: **3.1 (non-steam)**

[![Build and Release](https://github.com/Luotianyi-0712/sonetto-rs-build/actions/workflows/rust.yml/badge.svg)](https://github.com/Luotianyi-0712/sonetto-rs-build/actions/workflows/rust.yml)

---

## What is Sonetto-rs?

Sonetto-rs is a Rust implementation of a server emulator (PS) for *Reverse: 1999* (PC version). This project exists because there was never a PS for Reverse: 1999, so we built one.

![main image](/images/r99-murc.png)

---

## Table of contents

* [Quick start](#quick-start)
* [Requirements](#requirements)
* [Features](#features-what-works-now)
* [Known limitations / Not working (confirmed)](#known-limitations--not-working-confirmed)
* [Bug fixes](#bug-fixes)
* [Prebuilt binaries](#prebuilt-binaries)
* [Contributing](#contributing)
* [Known bugs](#known-bugs)
* [Plans / Roadmap](#plans--roadmap)
* [Todo](#todo)
* [GM Commands](#gm-commands)

---

## Quick start

> These commands assume a Windows command prompt or PowerShell; adjust paths/commands for Linux/macOS.

1. Clone this repository:

```bash
git clone https://github.com/Yoshk4e/sonetto-rs.git
cd sonetto-rs
```

2. Install Rust: [https://rust-lang.org/tools/install](https://rust-lang.org/tools/install)

3. Clone the required data repository and place it where Sonetto-rs can read it:

```bash
git clone https://gitlab.com/yoncodes/sonetto-data.git
```

4. Add `excel2json` to `sonetto-rs/data/` (the project expects an `excel2json` and a `static` folder in the runtime `data` directory).

5. Build the project:

```bash
cargo build --release
```

6. The built binaries will be in `target/release/`:

* `sdkserver`
* `gameserver`

Create a `data/` folder next to those binaries and copy `excel2json` and the `static/` folder into it.

7. Apply the client patch so the game talks to your server:

* Use the [sonetto-patch](https://github.com/yoncodes/sonetto-patch) to make the game client compatible.

8. Start the servers (open two terminals):

```bash
./sdkserver
./gameserver
```

9. Login using an email address in the game client (**DO NOT USE THE REGISTER BUTTON**, if the account doesn't exist it will be created automatically).

![login image](/images/r99-email.png)

---

## Requirements

* Rust toolchain (nightly)
* `sonetto-data` repository (game data files)
* `excel2json` placed in `data/` alongside `static/`
* The PC version of *Reverse: 1999* (official client), the server is built for the PC client.

---

## Features (what works now)

* Self-contained: uses SQLite (no external DB hosting required)
* All skins, heroes and psychubes unlocked by default
* Starter currency (3,000,000)
* Battles and auto-battle (basic)
* Battle replay support
* Battle teams: save and load team configurations
* Username changes
* Profile hero selection and psychube assignment
* Background music (BGM / jukebox)
* Gacha functionality (partial, see limitations)
* Ripple & Standard banners
* Main story progression
* Items can be added to inventory
* Consumables now work (can choose psychubes, currency, and portraits)
* Pawnshop works(can buy all items, now gets added to inventory)
* Fragment shop 
* Equipment can be locked and unlocked
* Characters can be marked as favorite
* Gacha now reduces currency used and converts when low
* GM commands now added (**check friend list**)
* Resonance system added (can update/add talent styles)
* Premium shop added (can purchase premium items)
* Shop now resets (daily, weekly and monthly)
* Mail added (can claim rewards)
* Insight items added (level heroes to i3 lvl 1)
* Auto use expired items

---

## Known limitations / Not working (confirmed)

* Tower battles
* Trial heroes (buggy: replay/load not saved), not fully implemented
* ~~Hero talents aren't persisted or applied correctly~~
* Achievements system
* Tasks / quest systems
* Battle pass
* ~~Full currency logic (some gacha/currency flows are incomplete)~~
* Profile picture upload/management
* Real-time battle logic: currently battles may be fast-forwarded/skipped to the end
* Drop rates and reward balancing need comprehensive testing

---

## Bug fixes

* Psychube upgrade materials no longer locked by default
* Fixed a bug with 7 day sign rewards (users no longer get double rewards)
* Fixed a bug with mail (users can now claim rewards)

---

## Prebuilt binaries

Prebuilt releases are available (thanks to Luotianyi-0712). Check the build repository's Releases page and download the release that matches your client version. Note: compatibility is not guaranteed across every client build, Check the upstream and the date of the build because the build repo syncs with upstream everyday at 2AM UTC.

* Build repo: [https://github.com/Luotianyi-0712/sonetto-rs-build](https://github.com/Luotianyi-0712/sonetto-rs-build)

---

## Contributing

1. Fork this repository.
2. Create a feature branch (e.g. `feature/cool-thing`).
3. Make your changes, run `cargo build --release` and test locally.
4. Open a pull request against the upstream repository with a clear description of your change and any testing notes.

Please follow existing code style and keep changes focused per PR.

---

## Known bugs

* **Ezio moxie display**: Ezio sometimes shows an incorrect (very large) moxie value; (its part of the battle mechanics. since we only have bare minimum passives aren't implemented yet)
* **Month card daily sign-in** sometimes ui pops up twice during sign in

If you encounter other bugs, please open an issue with reproduction steps and relevant logs.

---

## Plans / Roadmap

Short-term:

* Fix confirmed broken handlers and persistence bugs
* Implement a proper account & progress management system (replace current hardcoded/maxed defaults)
* Finish ~~currency~~ gacha logic and balance drops
* Improve battle logic to match the official game behavior

---

## Todo

* Remove dead / unnecessary code
* Remove unused static/starter data
* Improve documentation and setup instructions

---

## GM Commands

* Use currency, item and equip table for ids
* Here are some example commands
* /currency 1 1000
* /item 140001 1000
* /equip 1000 1

---

## Credits

Thanks to the upstream contributors and to Luotianyi-0712 for prebuilt artifacts and CI.

---
