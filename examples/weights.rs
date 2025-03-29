use trackfish::app::{load_tracks, settings::Settings, MusicController};

fn main() {
    let dir = Settings::load().directory;
    let tracks = load_tracks(&dir).unwrap();
    println!("{}", dir);
    let mut controller = MusicController::new(tracks, dir);
    
    std::fs::write("./weights.txt", controller.get_weights().map(|weight| weight.to_string()).to_vec().join("\n"));
}
