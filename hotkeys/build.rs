fn main() {
    // Embed the application icon into the exe
    if std::path::Path::new("icon.rc").exists() {
        let _ = embed_resource::compile("icon.rc", embed_resource::NONE);
    }
}
