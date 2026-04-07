use std::path::Path;

fn main() {
    let kernel_path = std::env::var("CARGO_BIN_FILE_RINO_KERNEL_rino-kernel")
        .expect("Kernel binary not found — is the artifact dependency set up?");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let bios_image_path = Path::new(&out_dir).join("rino-bios.img");

    bootloader::BiosBoot::new(Path::new(&kernel_path))
        .create_disk_image(&bios_image_path)
        .expect("Failed to create BIOS disk image");

    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_image_path.display());
}
