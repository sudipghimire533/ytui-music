# Ytui-music
A test attempt to bring youtube (only audio [not youtube music]) to tui

## Current Status
Project is pretty much stuck maybe because of poor state machine design.

## About Project
I listen to music a lot and youtube music is currently not available in my country (Nepal :love:) and even If it was available I prefer not to give
a tab of my browser to just listen music. Additionaly, I love to stay in terminal.
This was when I thought to build one. which I thought would be one of another small project while I learn rust and programming in general.


## Future Goal
I was pretty much excited to bring this project to usable form but It isn't there yet. However the base has been built and only remaining is to implement.
I would defenetly love to see this project working but I will be able to work on this only after sometime. If you loved the idea of was thinking of doing
something similar you may for this repo and see the `front-end/src/event.rs`.

I think that's where I failed. To be exact I was stuck to have async call to the `fetcher` backend from `event.rs`. You can see fetecher is only implemented
to get trending feed from youtube (through awesome project called invidious) but that part will be piece of cake I think.

If you decided to work on this I wil be glad if you ping me. Or just open an issue/PR.

## Tools used
Programming language: Rust
Front-end: tui-rs
http client: reqwest
Youtube data extractor: Invidious
