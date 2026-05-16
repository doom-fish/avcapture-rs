mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let output = PhotoOutput::new()?;
    println!("photo output generic info: {:?}", output.output_info()?);
    println!("photo output specific info: {:?}", output.info()?);

    if let Err(err) = output.capture_photo(|result| {
        println!("photo capture callback: {result:?}");
    }) {
        support::print_skip("photo capture (output not attached to a running session)", err);
    }

    Ok(())
}
