# sonetto-rs

![main image](/images/r99-murc.png)

## What is sonetto-rs?

sonetto-rs is a ps for reverse 1999 made in rust. why? no one tried and succeeded yet.

## How to use sonetto-rs?
- need to install rust
- open terminal or command prompt in project root directory
```bash
cargo build --release
```
sdkserver and gameserver will be in the target/release directory

- need to use the sonetto patch (soon)
- now open two terminals or command prompts

```bash
    .\sdkserver
```
```bash
    .\gameserver
```
- Login with email. if account doesn't exist it will be created automatically

![heroes image](/images/r99-heroes.png)

## Features
Everythings unlocked out the box
- Self contained (uses Sqlite no db hosting needed)
- All skins
- All heroes
- All Psychubes
- 3m currency to start
- Battles work (kinda)
- Battle replay works
- Battle teams can be set and saved now
- Username changes work
- Users can change the profile heros
- Users can change Psychubes on heros
- BGM works (juke box anyone?)
- Gacha works (80%) (need to add currency logic)
- Ripple Banner works

## Not working (confirmed)
- Tower battles
- Setting hero talents
- Achievements
- Tasks
- Battle pass
- Stories (not tested)
- Currency logic (soon)
- Profile picture (soon)
- Real battle logic (right now we skip battle to the end)
- Auto battle (soon)
- Drop rates need to be tested

## KnownBugs
- 7 day sign is bugged (sometimes rewards are given twice 12 am and 12:30 am) (I blame the game for using 3 different time formats)(u64, i64 and i32 lmao)
- Ezio has max number of moxie its a visual bug (no idea why yet) (normal max is 5 he's showing almost a 100)

## Plans for the future

For now I'll just fix the handles that are not working. Then implement a proper system for users to manage their accounts and progress.
Right now everything is hardcoded to be maxed out which isn't ideal for some people. Eventually we'll add proper currency logic and real battle logic.

## Todo
- remove unnecessary code
- remove unused static/starter data
