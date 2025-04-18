# TrackFish
![](https://raw.githubusercontent.com/rhaskia/trackfish/refs/heads/main/exampleimage.png)
This is a music player, made for offline usage while not having to sacrifice online features.
It works just as well as any other music player would (apart from a big lacking in settings/search right now), but the main feature of it is the radio system.
Using ~tags on your mp3/wav/other files (which you may need to tag using picard/onetagger/yourself~ audio features, the application weights songs by how "close" they are to one another, so your massive music collection doesn't leave you skipping over and over again as you try to get to certain songs.

# Compiling on Android
You will need to copy the android folder into `.\target\dx\trackfish\release\android\app\app\src\main\` before building the app. 
You will also need to use my modified version of the dioxus-cli, branch manifest.

# To Do:
 - [x] Audio playing, skipping, etc
 - [x] Working track view
 - [x] Proper Album & Artist Views
    - [x] More view information (time, artists, etc)
    - [x] Track settings (play, play after, start radio)
 - [ ] Proper Search View
 - [x] Shuffle/Unshuffle
 - [x] Custom Music Folder
 - [ ] Media notifications/control
    - [x] Android
    - [ ] Desktop
 - [ ] Playlists 
    - [x] Creation
    - [x] Playing as queue
    - [x] Saving
    - [x] Adding tracks
    - [ ] Deletion
    - [ ] Removing tracks
 - [ ] Auto Playlists
    - [ ] Basics that foobar would have
    - [ ] Sort by audio features
 - [ ] Settings
    - [x] Settings View
    - [ ] Audio settings (volume, fade, etc)
    - [x] Radio settings (weights, temperature, etc)
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
 - [ ] Theming (loading of custom css)
 - [ ] Queue Management
    - [x] Switch queues
    - [x] Select song in queue
    - [ ] Locked Queues/Temp Queues
    - [ ] Drag and Drop
    - [ ] End of queue options - stop, next, repeat, reshuffle etc
