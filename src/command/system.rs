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
pub async fn create_netns_link(pid: u64) -> Result<(), Error> {
    let src_dir = format!("/proc/{}/ns/net", pid);
    let dst_dir = format!("/var/run/netns/{}", pid);

    let mut cmd = Command::new("ln");
    cmd.arg("-s").arg(src_dir).arg(dst_dir);
    cmd_runner(cmd).await?;

    Ok(())
}

pub async fn remove_netns_link(pid: u64) -> Result<(), Error> {
    let link_path = format!("/var/run/netns/{}", pid);

    let mut cmd = Command::new("ls");
    cmd.arg(&link_path);
    cmd_runner(cmd).await?;

    let mut cmd = Command::new("rm");
    cmd.arg("-f").arg(link_path);
    cmd_runner(cmd).await?;

    Ok(())
}