use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/proto/over18.proto"], &["src/proto"])?;
    println!("* * * * * * BUILDING * * * * * *\n");
    Ok(())
}
