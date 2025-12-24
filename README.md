# sonetto-rs

## What is sonetto-rs?

sonetto-rs is a ps for reverse 1999 made in rust. why? no one tried and succeeded yet.

![main image](/images/r99-murc.png)

## How to use sonetto-rs?
- Clone the repo
- Install [rust](https://rust-lang.org/tools/install/)
- Clone [Sonetto-Data](https://gitlab.com/yoncodes/sonetto-data) 
- Open the project folder for sonetto-rs
- Add excel2json to the data folder
- open terminal or command prompt in sonetto-rs root directory
```bash
cargo build --release
```

- sdkserver and gameserver will be in the target/release directory
- you can move these two files to another location if you want
- in the same folder as sdkserver and gameserver make a data folder and copy excel2json and static to it

- need to use the [sonetto patch](https://github.com/yoncodes/sonetto-patch) to make the game work with the server
- now open two terminals or command prompts

```bash
    .\sdkserver
```
```bash
    .\gameserver
```
- Login with email. **NOT REGISTER** if the account doesn't exist it will be created automatically
![login image](/images/r99-email.png)

## Expectations
- This is the first release of sonetto-rs, expect bugs and issues. Please report any issues you find.

![heroes image](/images/r99-heroes.png)

## Features
Everythings unlocked out the box
- Self contained (uses Sqlite no db hosting needed)
- All skins
- All heroes
- All Psychubes
- 3m currency to start
- Battles work (kinda)
- Auto battle works
- Battle replay works
- Battle teams can be set and saved now
- Username changes work
- Users can change the profile heros
- Users can change Psychubes on heros
- BGM works (juke box anyone?)
- Gacha works (80%) (need to add currency logic)
- Ripple Banner works
- Standard Banner works
- Main story now works


## Not working (confirmed)
- Tower battles
- Trial heros are bugged (doesn't save in battle replay or load)(not implemented yet)
- Setting hero talents
- Achievements
- Tasks
- Battle pass
- Currency logic (soon)
- Profile picture (soon)
- Real battle logic (right now we skip battle to the end)
- Drop rates need to be tested

## Known Bugs
- ~~7 day sign is bugged (sometimes rewards are given twice 12 am and 12:30 am) (I blame the game for using 3 different time formats)(u64, i64 and i32 lmao)~~ (fixed)
- Ezio has max number of moxie its a visual bug (no idea why yet) (normal max is 5 he's showing almost a 100)

## Plans for the future

For now I'll just fix the handles that are not working. Then implement a proper system for users to manage their accounts and progress.
Right now everything is hardcoded to be maxed out which isn't ideal for some people. Eventually we'll add proper currency logic and real battle logic.

## Todo
- remove unnecessary code
- remove unused static/starter data
