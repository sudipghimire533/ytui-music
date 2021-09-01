# Ytui-music
A test attempt to bring youtube (only audio [not youtube music]) to tui

## Current Status
The binary can do following for now:
* Search for youtube playlist, channel and videos
* Play the selection from music list
* Paging of result
* See trending music
* See playlist and videos uploaded in a channel
* See items included in playlist
This implies that project a bit behind from usable state. However all the base for implementing future goals have been pretty much complete.
I would love to see you implement those

## About Project
I listen to music a lot and youtube music is currently not available in my country (Nepal :love:) and even If it was available I prefer not to give
a tab of my browser to just listen music. Additionaly, I love to stay in terminal.
This was when I thought to build one. which I thought would be one of another small project while I learn rust and programming in general.


## Future Goal
* Maintain local data of liked song, favourates song, save for offline, following artist
* Play the whole playlist
* Make configurable by user. (Currently several configurable paramaters have been hardcoded as global constatnt variable eg: REFRESH_RATE, REGION.)
* Implement help window
* Explore whats on youtube music channel through YoutubeCommunity option and so on
Any suggestion are welcome

## Tools used
Programming language: Rust
Front-end: tui-rs
http client: reqwest
backend-player: libmpv-rs (This project seems inactive from long time. Might need reconsider this option or extend the frok)
Youtube data extractor: Invidious

## Project Architecture
Project have two workspace memeber
1) Fetcher
This is a library crate that handles retriving of data from web or reading the file for local data like favourates music.
**Content**
*src/lib.rs* : high level decleration of available function and strructures
*src/utils.rs* : defination and imlementation which extends the decleration

2) Front end
This is the binary crate that handles everything else from Fetcher. This majorly include the communication with *Fetcher* and the ui
**Content**
*src/mian.rs* : Responsible for initilizing the configuration, parsing command line option and spwawning other worker thread
*src/communicator.rs* : Responsible for communicating with the fetcher for sending and initiling data
*src/ui/mod.rs* : High level decleration of ui itself
*src/ui/utils.rs* : Defination to expend *src/ui/mod.rs*
*src/ui/event.rs* : Handles the keypress and calling the proper function of *src/communicator.rs*
