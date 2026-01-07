use tonic_prost_build::configure;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=NULL");

    configure().compile_protos(&["./proto/auth/auth.proto"], &["proto"])?;
    Ok(())
}
