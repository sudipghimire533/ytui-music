# Ytui-music
Listen to youtube from terminal with a decent tui

## Why ytui-music
Youtube have been go-to platform for many of us to stream music. But it is well known that youtube tracks your listening activity for feeding ads.

Addition to that one don't always want to open whole browser window just to listen to music because browser takes huge portion of RAM and CPU. Or it is not possible to open a browser window at all like when you are only using cli from ssh or have no GUI at all.

To download anything from youtube you either have to head to some downloader website or fire youtube-dl again. This may be hassale over time

## Benefits
So, youtube-dl solves exactly those three problem by:
* Being lightweight
* Runs on terminal, no need of GUI or browser
* Download playlist and music with only few press

You would also love
* Keyboard driven workflow
* Simple and easy configuration
* Decent and configurable look and feel
* Fully transparent being open source.
* Adding feature and improvement is always welcome and encouraged by author


## Installing
1) Install mpv and youtube-dl
    * mpv: https://mpv.io/installation
    * youtube-dl: https://ytdl-ord.github.io/youtube-dl/download.html

2) Download latest binary from [Release page](https://github.com/sudipghtimire533/ytui-music/releases/latest)

3) You should have following directory already existing in your system
    * A config root directory where ytui-music can create own directory to store configuration
        - Linux: `$HOME/.config` or `$XDG_CONFIG_HOME`. Eg: `/home/alice/.config`
        - MacOs: `$HOME/Library/Application Support`. Eg: `/Users/Alice/Library/Application Support`
        - Windows: `{FOLDERID_RoamingAppData}`. Eg: C:\Users\Alice\AppData\Roaming
    * A music directory where to download data. This can later be changed from configuration and required only when you reset or create configuration for first time
        - Linux: `$HOME/Music` or `$XDG_MUSIC_DIR`. Eg: `/home/alice/Music`
        - MacOs: `$HOME/Music`. Eg: `/Users/Alice/Music`
        - Windows: `{FOLDERID_Music}`. Eg: `C:\Users\Alice\Music`

4) Start ytui-music
    - Show help message:
        `ytui_music help`

    - Launch:
        `ytui_music run`

    - Show shortcuts/ default keybindings:
        `ytui_music info keys`

5) Show some love
Let me know that you actually have it by giving a [star on github](https://github.com/sudipghimire533/ytui-music)

## Building from source
Refer to [build](CONTRIBUTING.md#building)

## Contributing
I would be pleased to know that you have interest to contribute something. This could be anything from suggestion, bug report, issue or the improvement. If you have any suggestion, facing problem or anything feel super duper free to open an issue.

Also if you would like to contibute code, you may start with [CONTRIBUTING.md](CONTRIBUTING.md)
