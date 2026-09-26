//! Regenerates `src/ipc/bindings.ts` from the tauri-specta builder, with no window or event loop.
//! Run via `cargo run -p heimdall-desktop --bin generate_bindings` (also what
//! `scripts/check-bindings.mjs` shells out to).

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/ipc/bindings.ts");
    heimdall_desktop_lib::commands::builder()
        .export(specta_typescript::Typescript::default(), path)
        .expect("export typescript bindings");
    println!("wrote {path}");
}
