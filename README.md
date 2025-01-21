# TrackFish
![./exampleimage.png](./exampleimage.png)
This is a music player, made for offline usage while not having to sacrifice online features.

It works just as well as any other music player (apart from a major lack of settings/search right now), but its main feature is the autoplay system.

Using tags on your mp3/wav/other files (which you may need to tag using Picard/Onetagger/yourself), the application weights songs by how "close" they are to one another, so your massive music collection doesn't leave you skipping over and over again as you try to get to certain songs. To decide how close the songs are, TrackFish mainly uses genres, but first encodes them into a latent space (shown by the Python folder with all the various tries at machine learning with song genres) to capture the rough "vibe" of a song's genres. TrackFish also decreases weights from artists recently listened to in the queue as well, to make sure one artist that may have a very unique subset of genres from repeating.

# Installation
To install TrackFish, you can either get a prebuilt binary from releases or build the application using dioxus-cli.

Firstly, [install rust](https://www.rust-lang.org/tools/install), and then use `cargo install dioxus-cli` to install the cli.

Then, you can run `dx serve --platform PLATFORM`, where PLATFORM is either desktop, android, or iOS (although iOS may not have the correct permissions).

# Features
- Track view and playback
- Explorer screens for artists, albums, genres and more!
- Queues for the above categories and basic queue management
- Autoplay System

# To Do:
- Proper Album & Artist Views
- Autoplaylists (implemented on the backend just need UI)
- More View Information (time, artists, etc)
- Comprehensive Settings
- Exceptions for albums, artists from shuffle
- View Settings
- More Weighting
- Theming
- More Queue Management

# Contributing 
Feel free to contribute! Running the program is described above and any issues can be posted to this repo.
