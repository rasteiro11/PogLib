fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/poglib.proto")?;

    //tonic_build::configure()
    //    .out_dir("proto/")
    //    .compile(&["proto/poglib.proto"], &["proto"])
    //    .unwrap();

    Ok(())
}
