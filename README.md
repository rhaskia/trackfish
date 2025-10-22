# TrackFish
![](https://raw.githubusercontent.com/rhaskia/trackfish/refs/heads/main/image.png)
TrackFish is a music player made for offline usage without having to sacrifice features typically only associated with streaming services - the ability to have similar songs to continue playing after one ends, called autoplay or a radio in many apps. TrackFish also seeks to be a generally comprehensive music player with features such as playlists, autoplaylists, comprehensive queue management. Planned features include scrobbling, tagging, and more. 

# Compiling
On desktop, you will need the dioxus-cli, and to install the following packages:
Linux: WebkitGtk, xdotool
Windows: WebView2 (packaged with Edge),
and aubio on all platforms

## Android
You will need to build the Dioxus CLI from [github](https://github.com/DioxusLabs/dioxus).

# To Do:
 - [x] Audio playing, skipping, etc
 - [x] Working track view
 - [x] Proper Album & Artist Views
    - [x] More view information (time, artists, etc)
    - [x] Track settings (play, play after, start radio)
 - [x] Shuffle/Unshuffle
 - [x] Custom Music Folder
 - [x] All tracks search
 - [ ] Search 
    - [x] Search for albums/artists/genres
    - [x] Search View
    - [ ] Better search algorithm
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
    - [ ] Locking queues to stop them being recached on new load/scan?
    - [ ] End of queue options - stop, next, repeat, reshuffle etc
    - [ ] Sorting features
 - [x] Auto Playlists
    - [x] Sort by metadata
    - [ ] Automatic re-caching onload or on any change?
    - [ ] Sort by audio features
 - [ ] Theming (loading of custom css)
