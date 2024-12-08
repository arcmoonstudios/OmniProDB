fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    
    tonic_build::configure()
        .file_descriptor_set_path(format!("{}/descriptor.bin", out_dir))
        .compile(
            &["proto/database.proto", "proto/omnipro.proto"],
            &["proto"]
        )?;
    
    println!("cargo:rerun-if-changed=proto/");
    Ok(())
}