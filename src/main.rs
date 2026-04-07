use std::process::Command;

fn main() {
    let bios_image = env!("BIOS_IMAGE");

    // Launch QEMU with the disk image
    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args([
        "-nographic", // no GUI window
        "-drive",
        &format!("format=raw,file={bios_image}"), // boot from our disk image
    ]);

    let status = cmd
        .status()
        .expect("Failed to launch QEMU. Is it installed? Run: brew install qemu");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}
