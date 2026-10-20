fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rerun-if-changed=src/calendar/eventkit_macos.m");
        let mut native = cc::Build::new();
        native
            .file("src/calendar/eventkit_macos.m")
            .flag("-fobjc-arc")
            .flag("-fblocks");
        native.compile("todosidebar_calendar");
        // @available benötigt __isPlatformVersionAtLeast aus compiler-rt.
        // Rust linkt mit -nodefaultlibs; Clang fügt seine Runtime hier nicht
        // automatisch hinzu. Den Pfad vom tatsächlich verwendeten Compiler holen.
        let runtime = native
            .get_compiler()
            .to_command()
            .arg("--print-runtime-dir")
            .output()
            .expect("Clang runtime directory could not be queried");
        assert!(
            runtime.status.success(),
            "Clang runtime directory query failed"
        );
        let runtime_dir =
            String::from_utf8(runtime.stdout).expect("Clang runtime directory is not UTF-8");
        let runtime_dir = runtime_dir.trim();
        assert!(
            std::path::Path::new(runtime_dir)
                .join("libclang_rt.osx.a")
                .is_file(),
            "macOS Clang runtime not found in {runtime_dir}"
        );
        println!("cargo:rustc-link-search=native={runtime_dir}");
        println!("cargo:rustc-link-lib=static=clang_rt.osx");
        println!("cargo:rustc-link-lib=framework=EventKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }
    tauri_build::build();
}
