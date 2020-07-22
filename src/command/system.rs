use std::io::{Error};
use tokio::process::Command;

use super::utils::cmd_runner;

// create a new directory, return error on failure
pub async fn create_dir(dir: &str) -> Result<(), Error> {
    let mut cmd = Command::new("mkdir");
    cmd.arg("-p").arg(dir);
    cmd_runner(cmd).await?;

    Ok(())
}

// link /proc/pid/ns/net to /var/run/netns/pid, 
// return error on failure
pub async fn link_netns(pid: u64) -> Result<(), Error> {
    let src_dir = format!("/proc/{}/ns/net", pid);
    let dst_dir = format!("/var/run/netns/{}", pid);

    let mut cmd = Command::new("ln");
    cmd.arg("-s").arg(src_dir).arg(dst_dir);
    cmd_runner(cmd).await?;

    Ok(())
}