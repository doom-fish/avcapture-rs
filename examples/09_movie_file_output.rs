mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let output = MovieFileOutput::new()?;
    println!(
        "movie file output generic info: {:?}",
        output.output_info()?
    );
    println!(
        "movie file output deferred start: supported={:?}, enabled={:?}",
        output.deferred_start_supported()?,
        output.deferred_start_enabled()?
    );
    println!("movie file output specific info: {:?}", output.info()?);
    output.set_sample_buffer_boundary_handler(|_sample| {})?;
    println!(
        "movie file output sample-buffer boundary callback installed: {}",
        output.sample_buffer_boundary_callback_installed()?
    );

    let recording_path = std::env::current_dir()?
        .join("target")
        .join("example-movie-file-output.mov");
    match output.start_recording_with_handler(&recording_path, |event| {
        println!("movie recording callback: {event:?}");
    }) {
        Ok(()) => {
            println!("movie recording unexpectedly started; stopping immediately");
            output.stop_recording();
        }
        Err(err) => {
            support::print_skip(
                "movie recording (output not attached to a running session)",
                err,
            );
        }
    }

    output.clear_sample_buffer_boundary_handler();
    Ok(())
}
