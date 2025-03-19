use trackfish::queue::QueueManager;
use trackfish::track::load_tracks;

fn main() {
    let mut queue = QueueManager::new(load_tracks("E:/music"));

    println!("{:?}", queue.current_track());

    for _ in 0..10 {
        queue.skip();
        println!("{:?}", queue.current_track());
    }
}
