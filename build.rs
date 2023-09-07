fn main() -> std::io::Result<()>
{
    if !std::process::Command::new("qemu-img")
        .args(["create", "-f", "qcow2", "drive0.qcow", "4G"])
        .status()?.success()
    {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "'qemu-img create' failed"));
    }

    Ok(())
}