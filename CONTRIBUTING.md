Aha! good to seey you here!


# Tools used

* Programming Language
Rust. Yes, beautiful code, safe execution and easy development

* Tui-frontend
rust-tui. I could not see anything other than this and there is no need of anything other than this.

* Tui-backend
crossterm. This time there were options and I was in dellima to choose between crossterm and termion. And you know the
reason. Thor from asguard came and told me to use crossterm.

* Youtube data fetcher.
Invidious. Ohh boy! I am fan of this team for doin this great thing. And also would like to thanks all those who are hosting
invisious server along with providing api calls.

* Player backend
Mpv wrapped in libmpv-rs. I actually don't want to write it again but this crate seems dead. There was no problem till now but yall know
using unmaintained librray feels weird.

* Other libraries
See Cargo.toml file in each workspace-folder

### How are things splitted?
The project is cargo workspace with three members. Depending on what you want to improve or fix things:

1) Config
This may be simpliest of all. This simple defines the struct and functions needed to read and generate configuration file

2) fetcher
This lib handles getting and providing data from invidious. If you want to improve something that is related to fetching data from api like getting more field. This is where you should stare.

source files included are

**src/lib.rs**
This file constains the declaration like struct to represent the api result which makes it possible to convert response text to rust struct. Also A fetcher struct that stores the fetched response and helps to feed it in pagination manual. Eg: all the content of playlist are fetched in one call but is served only few at a time and other when use navigate through the list.

**src/utils.rs**
This contains the defination of required function. The communicator (mentioned below) interact with the function defined here. It is responsible of basic request sending storeing the result in own and serve in pagination manner as mentioned above.


3) front-end
This is binary crate. This is realtively big bit not-so-confusing (I guess). As you might suspected this handled the ui itself but not only that. It is easy to explain each file

**src/main.rs**
This file is does not that much. It simply spawn the threads for ui, event handler, and fetcher communicator. Also, the a single config (mentioned later) is initilised here as lazy static. If you want to do something before the ui is painted you may look here or if you want to add something that required another thread. Most of the time you may not look at this file but who knows what genius you are.

**src/communicator.rs**
Remember what I said fetcher do? This files serves as the communicator between ui and fetcher (which I would like to call backend). the ui signify that it needs some data and this is where initlizing of asked data happens. If you want to fix/add/remove/improve something which is related to calling correct function of fetcher and also initilizing the fetched data this is your place. Eg: If user search something the ui is responsibe to tell (which is actually setting the _filled_source_ field of a state struct) that it needs search result of this query then communivcator calls the respective function to fetch music, playlist and artist. On getting result from fetcher, it is also responsible to set those data and which is again later rendered by ui side. In all this, remember it also respects the pagination fetching we talked about

**src/ui/**
This module or sub-folder if you like to call it that way is everything related to ui itself. If you want to do something with something that is not something :laugh: communicator does then for sure reason this is where your time remains. The source file included are:

**src/ui/mod.rs**
This is mostly the collections of structs like the layout of ui, decleration on State structure, struct that defines the source of each fetched list and so on. Addition to that, it also have a `draw_ui` function that handles getting the layout of app, getting the individual widgets and painting them in right place. If you want to do something related to the layout, State struct or as I said high-level painting handeling this is where you start and utils.rs follows.

**src/ui/utils.rs**
This gives the functionality to the things decleared in mod.rs. This is responsible to actually create the ui layout as well as constructing the widgets. This was it keeps `draw_ui` function in mod.rs super simple(may be not "super"-simple). If you want to change something related to the widgets itself, the layout, initikizing of state variable, initilizing of mpv or anything that extend the struct defines in mod.rs, this is where you begin. Eg: I had to modify this file to great extent when I decided that all playlist, artist and musics whould be presented in tabular form.

**src/ui/event.rs**
As the name suggests this is response on the event handeling from the user. This files consists bunch of closure such that each closure handle specific event. If you want to modify something that is related to user interation then this is where you should look at. Eg: If someone wanted to define the 'o' key to open the selction in browser than one should define such logic here.

### Putting everythgin together / Architecture
The entry point is the `front-end/src/main.rs` file and precisly `main`  function defined there. The main function spawn three thread for each of below task
1) `draw_ui` function in `front-end/src/mod.rs` file to keep ui painting independent of other task
2) `event_sender` function in `front-end/src/ui/event.rs` to always keep accepting the user input
3) `communicator` function in `front-end/src/communicator.rs` to keep not much load in event.rs and keeping request fetching part seperate

All these thread however shared the same state variable and a CondVar variable. When any of thread change the state then the condvar variable is notified for all other thread and if needed thread do tasks respectively. Also all of three thread happens to be in a loop and all loop are breaked at once (when user quits)

Event if nothing is changed in state, still the CondVar is notified so that the `draw_ui` is called in interval of `REFRESH_RATE` as defines by user. This behaviour keeps the change not specifically made by user to sync like always showing the correct time-pos value of playing music.

It may be easier to get high level idea of how everythign work together if we alk through the app itself. Lets imagine you launched the tool this is what happend serially

All three thread are spawned as mentioned above. The `draw_ui` function runs rendering the ui.
When user press some key say RIGHT_ARROW, then the event.rs captures that event and call respective closure. The closure then change the state variable indicating that another window is active. Also it have not to notify the CondVar that state variable is now changed.

Other two thread were waiting to get notified. As notifier is already notified above, `draw_ui` repaints the ui this means that the window which was active is now hilighted accordingly.
On the onher hand, the communicator finds that nothing is changed of it's concern. So nothing happens there.

Again, supose user press enter after typing some query in search bar. The event.rs again change the state to signify that now musicbar, playlistbar and artistbar is to be filled from that query result and notify the notifier.

This time, communicator finds that new data is to be fetched, so it asks the fetcher to get music result of search query and initilize it to the searchlist. Do same for playlist and artists.

As not the result is filled in state it agains notify the notifier. As `draw_ui` have no checks it simple repaint the ui on every notifier. So the list just initilized from fetcher is shown in the ui.

And here how the story ends.
