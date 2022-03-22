# Ytui-music
Listen to music from youtube inside terminal with sleek tui

![Ytui-music Search Result](screenshots/search.png)

## [See more screenshots](#screenshots)

---

# Installation
1) Download the latest binary from [release page](https://github.com/sudipghimire533/ytui-music/releases/latest).
If the binary is not available for your platform head on to [build from source](#building-from-source)

2) Give it executable permission and from downloaded directory, in shell:
```
./ytui_music run
```
3) You may need to jump to [Usage Guide](#usage)

---

## Dependencies
ytui-music depends on `mpv` and `youtube-dl`. You may refer to the offical [website of mpv](https://mpv.io) and [website of youtube-dl](https://yt-dl.org).

If you have `choco` for windows or `brew` on mac or one of the popular package managers in linux you can do:

### - Windows (In powershell or cmd)
```
choco install mpv youtube-dl
```

### - MacOS
```
brew install mpv youtube-dl
```

### - Debian/ Ubuntu Derivatives
```
sudo apt update && sudo apt install youtube-dl libmpv1
```

**For other distributions install `youtube-dl` and `mpv` packages any way you please**

---

# Before running ytui-music
Before you start ytui-music make sure the following directory exists and that ytui-music has the write permission in order to save the configuration file.
## Windows
`C:\Users\<username>\AppData\Roaming` or env var `{FOLDERID_RoamingAppData}`

## MacOS
`$HOME/Library/Application Support`

## Linux
`$HOME/.config/` or the env var `$XDG_CONFIG_HOME`

---

# Building from Source
`ytui-music` is written entirely in Rust and thus making it dead simple to build from source. All you have to do is download the source, install `rust` and build with `cargo`.

1) Installing rust. Head to [Rust installation](https://www.rust-lang.org/tools/install). It is basically doing
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2) Get the source code. You can [download archives]() or git clone
```
git clone git@github.com:sudipghimire533/ytui-music
```

3) `cd` into source root and do:
```
cargo build --all --release
```

4) The compiled binary is located in `target/release/` directory. Copy the `ytui_music` binary and place it somewhere where it is easy to run.

5) Ytui-music is now ready to fire. [Head to usage](#usage)

---

# Usage

ytui-music is a single binary so it shouldn't be a hassle to run it. Just make sure you have [installation of dependencies](#dependencies).

### - Running ytui-music
```
ytui_music run
``` 
### - Showing help message
```
ytui_music help
```
### - Showing current configured shortcuts
```
ytui_music info shortcuts
```
### - Showing version information
```
ytui_music info version
```

## Searching
1) Press `/` to go to search box
2) Type
    - `music:Bartika Eam Rai` to search only music results with the query "Bartika Eam Rai"
    - `playlist:Soft pop hits` to search only playlist results with the query "Soft pop hits"
    - `artist:Bibash Jk` to search only artists with the query "Bibash Jk"
    - `Coding music` to search playlists, music and artists at once with the query "Coding music"
3) Press the `Enter` key

## Navigating
- Use `Left arrow` or `Backspace` for backward and `Right arrow` or `Tab` key for forward to **move between Sidebar, Musicbar, Playlistbar and Artistbar**
- Use `Up arrow` or `Down arrow` to move up or down in the list which will **highlight the list item**
- Press `Enter` key to **select an item**

## Playback control
- Press `Space` key **to pause/unpause the playback**
- Press `s` key to **toggle shuffle/unshuffle**
- Press `r` key to **repeat one or all items in playlist**
- Press `>` for forward and `<` for backward **playback seek**
- Press `CTRL+n` for next and `CTRL+p` to **change track**

## Downloading
1) Highlight the item you want to download. Currently supported is downloading of music and playlists.
2) Press `CTRL+d` to **download the selection**

## Quitting
- Press `CTRL+c` to **quit ytui-music**
- If a download is ongoing press `CTRL+ALT+C` to force quit

## Adding to favourites
1) Highlight the item you want to add or remove from favourites
2) Press `f` to add or `u` to remove from favourites
3) To see your list
    - Favourite music is shown in `Liked` section in the sidebar
    - Favourite playlist is shown in `My playlist` section in the sidebar
    - Favourite artist is shown in `Following` section in the sidebar

---

# Screenshots
This is what ytui-music looks like. Your may look even better. ;)
<details>

<summary> Click to see the screenshots</summary>

![Initial Screen](screenshots/initial-screen.png)
![Searching Music](screenshots/music-search.png)
![Search Results](screenshots/search.png)
![Responsive Ui](screenshots/small-screen.png)
![Music Info](screenshots/music-info.png)
![Playing Music](screenshots/playing.png)

</details>
