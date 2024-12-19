use music::queue::{QueueManager, ShuffleMode};
use music::track::load_tracks;

fn main() {
    let mut queue = QueueManager::new(load_tracks("E:/music"));
    queue.propagate_info();

    println!("{:?}", queue.current_track());

    queue.mut_queue().shuffle_mode = ShuffleMode::PlaySimilar;
    queue.skip();

    println!("{:?}", queue.current_track());
}
