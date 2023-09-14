const DRIVE0: &'static str = "drive0.qcow";

fn main() -> std::io::Result<()>
{
    let drive0 = std::path::PathBuf::from(DRIVE0);
    if !drive0.exists()
    {
        println!("cargo:warning={} not found. Running qemu-img...", DRIVE0);
        if !std::process::Command::new("qemu-img")
            .args(["create", "-f", "qcow2", DRIVE0, "4G"])
            .status()?.success()
        {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "'qemu-img create' failed"));
        }
    }

    Ok(())
}