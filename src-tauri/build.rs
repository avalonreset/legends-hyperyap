#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn stage_cuda_provider_runtime() {
    let out_dir = std::path::PathBuf::from(
        std::env::var_os("OUT_DIR").expect("Cargo did not provide OUT_DIR"),
    );
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("Unable to resolve Cargo profile directory from OUT_DIR");
    let destination_dir = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo did not provide CARGO_MANIFEST_DIR"),
    )
    .join("runtime")
    .join("windows-x64");
    std::fs::create_dir_all(&destination_dir)
        .expect("Unable to create the staged Windows CUDA runtime directory");

    for file_name in [
        "onnxruntime_providers_cuda.dll",
        "onnxruntime_providers_shared.dll",
    ] {
        let source = profile_dir.join(file_name);
        let destination = destination_dir.join(file_name);
        let source_metadata = std::fs::metadata(&source).unwrap_or_else(|error| {
            panic!(
                "Required ONNX Runtime provider {} was not produced at {}: {}",
                file_name,
                source.display(),
                error
            )
        });
        let destination_matches = std::fs::metadata(&destination)
            .map(|metadata| metadata.len() == source_metadata.len())
            .unwrap_or(false);

        if !destination_matches {
            std::fs::copy(&source, &destination).unwrap_or_else(|error| {
                panic!(
                    "Unable to stage {} from {} to {}: {}",
                    file_name,
                    source.display(),
                    destination.display(),
                    error
                )
            });
        }
    }
}

fn main() {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    stage_cuda_provider_runtime();

    tauri_build::build()
}
