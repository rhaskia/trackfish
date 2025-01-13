console.log(navigator.mediaCapabilities);
console.log(navigator.mediaSession);

navigator.mediaSession.playbackState = "playing";

navigator.mediaSession.metadata = new MediaMetadata({
    title: "Unforgettable",
    artist: "Nat King Cole",
    album: "The Ultimate Collection (Remastered)",
    artwork: [
    ]
});

navigator.mediaSession.setActionHandler("play", () => {
    console.log("Play");
});
navigator.mediaSession.setActionHandler("pause", () => {
    console.log("Pause");
});
navigator.mediaSession.setActionHandler("stop", () => {
    console.log("Play");
});
navigator.mediaSession.setActionHandler("seekbackward", () => {
    console.log("Seek Back");
});
navigator.mediaSession.setActionHandler("seekforward", () => {
    /* Code excerpted. */
});
navigator.mediaSession.setActionHandler("seekto", () => {
    /* Code excerpted. */
});
navigator.mediaSession.setActionHandler("previoustrack", () => {
    /* Code excerpted. */
});
navigator.mediaSession.setActionHandler("nexttrack", () => {
    console.log("Next Track");
});
navigator.mediaSession.setActionHandler("skipad", () => {
    /* Code excerpted. */
});
