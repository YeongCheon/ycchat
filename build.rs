fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["protobuf/chatService.proto"], &["protobuf/**"])?;
    // tonic_build::compile_protos("protobuf/chatService.proto")?;
    Ok(())
}
