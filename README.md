# TrackFish
![](https://raw.githubusercontent.com/rhaskia/trackfish/refs/heads/main/exampleimage.png)
This is a music player, made for offline usage while not having to sacrifice online features.
It works just as well as any other music player would (apart from a big lacking in settings/search right now), but the main feature of it is the radio system.
Using ~tags on your mp3/wav/other files (which you may need to tag using picard/onetagger/yourself~ audio features, the application weights songs by how "close" they are to one another, so your massive music collection doesn't leave you skipping over and over again as you try to get to certain songs.

# Compiling on Android
You will need to copy libc++shared.so into `.\target\dx\trackfish\release\android\app\app\src\main\jniLibs` before building the app unfortunately. 
You will also need to use my modified version of the dioxus-cli, branch manifest.

# To Do:
 - [x] Proper Album & Artist Views
      - [ ] More view information (time, artists, etc)
      - [ ] Track settings (play, play after, start radio)
 - [ ] Proper Search View
 - [x] Shuffle/Unshuffle
 - [x] Custom Music Folder
 - [ ] Autoplaylists
    - [ ] Basics that foobar would have
    - [ ] Sort by audio features
 - [ ] Basic Settings (Shuffle, Queue, etc)
 - [ ] Comprehensive Settings
    - [ ] Exceptions for albums, artists from shuffle
    - [ ] View Settings
    - [ ] End of queue action
 - [ ] More Weighting
      - [x] Spectral
      - [x] Chroma
      - [x] MFCC
      - [x] Zero Crossing Rate
      - [ ] Energy
      - [ ] Key
      - [ ] BPM/Tempo
 - [ ] Theming
 - [ ] More Queue Management
      - [ ] Locked Queues/Temp Queues
      - [ ] Drag and Drop
