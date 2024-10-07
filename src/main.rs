use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use knyst::{
    audio_backend::{CpalBackend, CpalBackendOptions},
    graph::*,
    prelude::*,
};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Parser)]
#[clap(version, about)]
pub struct Args {
    /// Sound file to play
    #[clap(long, default_value = "sessions/LRMonoPhase4.wav")]
    file: String,
    /// Playback volume. Will use `Mult` node if volume is not 1.0
    #[clap(long, default_value = "1.0")]
    volume: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let (error_sender, _error_receiver) = channel();
    let mut backend = CpalBackend::new(CpalBackendOptions::default())?;
    let _sphere = KnystSphere::start(
        &mut backend,
        SphereSettings {
            num_inputs: 0,
            num_outputs: 1,
            ..Default::default()
        },
        Box::new(move |error| {
            error_sender.send(format!("{error}")).unwrap();
        }),
    );

    let sound_buffer = Buffer::from_sound_file(&args.file)?;
    let buffer_channels = sound_buffer.num_channels();
    let buffer = knyst_commands().insert_buffer(sound_buffer);

    let mut k = knyst_commands();

    // let mut settings = k.default_graph_settings();
    // settings.sample_rate = backend.sample_rate() as f32;
    // settings.block_size = backend.block_size().unwrap_or(64);
    // settings.num_outputs = buffer_channels;
    // settings.num_inputs = 0;

    // k.init_local_graph(settings);
    let playback_node_id = buffer_reader_multi(buffer, 1.0, false, StopAction::FreeSelf);

    if args.volume == 1.0 {
        println!("Outputting raw file");
        // Works
        // graph_output(0, pan_mono_to_stereo().signal(playback_node_id).pan(0.5));
        graph_output(0, playback_node_id);
        // graph_output(0, playback_node_id.out(0));
        // graph_output(1, playback_node_id.out(1));
    } else {
        println!(
            "Outputting through `Mult` with multiplier of {:?}",
            args.volume
        );
        graph_output(0, playback_node_id * args.volume);
    }

    // let note_graph_id = k.upload_local_graph().unwrap();
    // graph_output(0, note_graph_id);

    let inspection = k.request_inspection();

    let total_duration = buffer.duration().to_seconds_f64() as u64;
    let pb = ProgressBar::new(total_duration);
    pb.set_style(
        ProgressStyle::with_template("{elapsed_precise} / {duration_precise} [{wide_bar}]")?
            .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    for _ in 0..total_duration {
        std::thread::sleep(Duration::from_secs(1));
        pb.inc(1);
        if let Ok(inspection) = inspection.try_recv() {
            dbg!(inspection);
        }
    }
    std::thread::sleep(Duration::from_secs(1));
    pb.finish();
    Ok(())
}
