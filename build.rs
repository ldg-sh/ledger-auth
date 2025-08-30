fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/ledger.proto");
    tonic_prost_build::compile_protos("proto/ledger.proto")?;
    Ok(())
}
