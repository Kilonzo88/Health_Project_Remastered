fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = tonic_build::configure();

    config.build_server(true);

    let proto_files: Vec<_> = std::fs::read_dir("proto")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "proto" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    config.compile(&proto_files, &["proto"])?;

    Ok(())
}
