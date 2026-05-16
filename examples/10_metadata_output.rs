mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let output = match MetadataOutput::new() {
        Ok(output) => output,
        Err(err) => {
            support::print_skip("metadata output", err);
            return Ok(());
        }
    };

    output.set_metadata_object_types(Vec::<String>::new())?;
    output.set_metadata_objects_handler(Some("avcapture-example-metadata"), |event| {
        println!("metadata callback: {event:?}");
    })?;
    println!("metadata output generic info: {:?}", output.output_info()?);
    println!("metadata output specific info: {:?}", output.info()?);
    output.clear_metadata_objects_handler();
    Ok(())
}
