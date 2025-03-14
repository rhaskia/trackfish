use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use mfcc::mfcc::Transform;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("/mnt/sdcard/music/Lemon [H5ZPCcnLXt4].mp3")?;
    let source = Decoder::new(BufReader::new(file))?;

    let samples: Vec<i16> = source.convert_samples().collect();
    println!("{:?}", samples.len());

    let sample_rate = 44_100;
    let frame_size = 1024;    
    let n_filters = 20;       
    let mfcc_len = 13;        

    let mut transformer = Transform::new(sample_rate, frame_size);
        // .nfilters(n_filters, 40)
        // .normlength(mfcc_len);

    let mut mfcc_output = vec![0.0; n_filters * 3];
    let mut mfccs = Vec::new();

    for chunk in samples.chunks(frame_size) {
        let mut frame = vec![0; frame_size];
        frame[..chunk.len()].copy_from_slice(chunk);

        transformer.transform(&frame, &mut mfcc_output);
        mfccs.push(mfcc_output[..mfcc_len].to_vec());
    }

    let mut average = vec![0.0; mfcc_len];

    for i in 0..mfcc_len {
        for j in 0..mfccs.len() {
            average[i] = mfccs[j][i];
        }
        //average[i] /= mfccs.len() as f64;
    }

    println!("{:?}", average);

    Ok(())
}

