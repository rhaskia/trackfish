use music::queue::{QueueManager, ShuffleMode};
use music::track::load_tracks;

fn main() {
    let mut queue = QueueManager::new(load_tracks("E:/music"));
    queue.propagate_info();

    println!("{:?}", queue.current_track());

    for _ in 0..10 {
        queue.skip();
        println!("{:?}", queue.current_track());
    }
}
