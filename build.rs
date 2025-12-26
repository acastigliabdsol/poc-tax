fn main() {
    // This build script will compile Cap'n Proto schemas if they exist.
    // For now, it's a placeholder to ensure the environment is ready for RT-001.
    if std::path::Path::new("schema.capnp").exists() {
        if std::process::Command::new("capnp").arg("--version").output().is_ok() {
            capnpc::CompilerCommand::new()
                .file("schema.capnp")
                .run()
                .expect("schema compilation failed");
        } else {
            println!("cargo:warning=capnp binary not found, skipping schema compilation");
        }
    }
}
