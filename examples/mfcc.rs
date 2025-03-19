use crate::gui::*;

use plotters::prelude::*;

const OUT_FILE_NAME: &str = "matshow.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let file = File::open("E:\\Music\\Lemon [H5ZPCcnLXt4].mp3");
    let (samples, sample_rate) = load_samples("/home/rhaskia/Downloads/octave.wav");
    //let samples = linear_resample(&samples, sample_rate as usize, 44100);
    println!("{}", samples.len());

    let frame_size = 1024;
    let mfcc_len = 13;

    // let result = calculate_mfcc(&mut mono_samples, sample_rate);
    // let mean = result.iter().sum::<f32>() / mfcc_len as f32;
    // let std = std_dev(result.clone(), mean);
    // let result = result.iter().map(|n| (n - mean) / std).collect::<Vec<f32>>();
    // println!("{result:?}");

    let chroma_vectors = extract_chroma(&samples, sample_rate as usize);

    let mut mean_chroma: Vec<f32> = vec![0.0; 12];

    for i in 0..12 {
        for j in 0..chroma_vectors.len() {
            mean_chroma[i] += chroma_vectors[j][i];
        }

        mean_chroma[i] /= chroma_vectors.len() as f32;
    }
    println!("{mean_chroma:?}");

    // let root = BitMapBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();
    //
    // root.fill(&WHITE)?;
    // 
    // let len = chroma_vectors.len();
    //
    // let mut chart = ChartBuilder::on(&root)
    //     .margin(5)
    //     .top_x_label_area_size(40)
    //     .y_label_area_size(40)
    //     .build_cartesian_2d(0i32..len as i32, 0i32..12i32)?;
    //
    // chart
    //     .configure_mesh()
    //     .x_labels(15)
    //     .max_light_lines(4)
    //     // .x_label_offset(35)
    //     // .y_label_offset(25)
    //     .disable_x_mesh()
    //     .disable_y_mesh()
    //     .label_style(("sans-serif", 20))
    //     .draw()?;
    //
    // chart.draw_series(
    //     chroma_vectors
    //         .iter()
    //         .zip(0..)
    //         .flat_map(|(l, y)| l.iter().zip(0..).map(move |(v, x)| (y, x, v)))
    //         .map(|(x, y, v)| {
    //             Rectangle::new(
    //                 [(x, y), (x + 1, y + 1)],
    //                 val_to_hsl(*v)
    //                 .filled(),
    //             )
    //         }),
    // )?;
    //
    // // let pitches = vec![("B", 40), ("A", 70), ("L", 100)];
    // let pitches = vec![("B", 65), ("A", 190), ("G", 305), ("F", 420), ("E", 485), ("D", 605), ("C", 720)];
    // let style = TextStyle::from(("sans-serif", 20).into_font());
    //
    // for (pitch, height) in pitches {
    //     root.draw_text(
    //         pitch,
    //         &style,
    //         (20, height),
    //     );
    // }
    //
    // root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    // println!("Result has been saved to {}", OUT_FILE_NAME);
    //
    // Ok(())
}

fn val_to_hsl(v: f32) -> HSLColor {
    let v = (v * 7.0) - 0.4;
    let hue = if v > 0.5 {
        0.0
    } else {
        0.6
    };

    let mut value = if v > 0.5 {
       1.0 - (v as f64 - 0.5)
    } else {
        v as f64 + 0.5
    };

    if value < 0.5 {
        value = 0.5;
    }

    HSLColor(
        hue,
        0.8,
        value
    )
}
