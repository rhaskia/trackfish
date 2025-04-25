# TrackFish
![](https://raw.githubusercontent.com/rhaskia/trackfish/refs/heads/main/exampleimage.png)
This is a music player, made for offline usage while not having to sacrifice online features.
It works just as well as any other music player would (apart from a big lacking in settings/search right now), but the main feature of it is the radio system.
Using ~tags on your mp3/wav/other files (which you may need to tag using picard/onetagger/yourself~ audio features, the application weights songs by how "close" they are to one another, so your massive music collection doesn't leave you skipping over and over again as you try to get to certain songs.

# Compiling
On desktop you will need the dioxus-cli, and to install the following packages:
Linux: WebkitGtk, xdotool
Windows: WebView2 (packaged with Edge),
and aubio on all platforms

## Android
You will need to copy the android folder into `.\target\dx\trackfish\release\android\app\app\src\main\` before building the app. 
You will also need to use my modified version of the dioxus-cli, branch manifest.

# To Do:
 - [x] Audio playing, skipping, etc
 - [x] Working track view
 - [x] Proper Album & Artist Views
    - [x] More view information (time, artists, etc)
    - [x] Track settings (play, play after, start radio)
 - [x] Shuffle/Unshuffle
 - [x] Custom Music Folder
 - [x] All tracks search
 - [ ] Search for albums/artists/genres
 - [ ] Media notifications/control
    - [x] Android
    - [ ] Desktop
 - [x] Playlists 
    - [x] Creation
    - [x] Playing as queue
    - [x] Saving
    - [x] Adding tracks
    - [x] Deletion
    - [x] Removing tracks
 - [ ] Settings
    - [x] Settings View
    - [x] Radio settings (weights, temperature, etc)
    - [ ] Audio settings (volume, fade, etc)
    - [ ] Library settings
    - [ ] Exceptions for albums, artists from shuffle (?)
    - [ ] View Settings
 - [x] Radio playing system
 - [x] More Weighting
    - [x] Spectral
    - [x] Chroma
    - [x] MFCCs
    - [x] Zero Crossing Rate
    - [x] Energy
    - [x] BPM/Tempo
 - [ ] Queue Management
    - [x] Switch queues
    - [x] Select song in queue
    - [x] Drag and drop
    - [x] Add track list to queue
    - [ ] Locked queues/temp queues
    - [ ] End of queue options - stop, next, repeat, reshuffle etc
 - [ ] Proper Search View
 - [ ] Auto Playlists
    - [ ] Basics that foobar would have
    - [ ] Sort by audio features
 - [ ] Theming (loading of custom css)
