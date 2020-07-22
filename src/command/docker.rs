use std::io::{Error};
use tokio::process::Command;

use super::utils::cmd_runner;

// launch a new container with a certain name for a given image
// return the pid of the container
pub async fn launch_container(name: &str, image: &str) -> Result<u64, Error> {
    // launch a container
    let mut cmd = Command::new("docker");
    cmd.arg("run").arg("--net=none").arg("--privileged").arg("-itd")
        .arg("--entrypoint=/bin/bash").arg("--name").arg(name).arg(image);
    cmd_runner(cmd).await?;

    // if successful, get the pid of the container
    let mut cmd = Command::new("docker");
    cmd.arg("inspect").arg("-f").arg("{{.State.Pid}}").arg(name);
    let stdout = cmd_runner(cmd).await?;

    // convert the stdout into a string
    let pid_string = String::from_utf8(stdout).unwrap();
    
    // return the pid back, trim the ending "\n" to avoid parse errors
    Ok(pid_string.trim().parse::<u64>().unwrap())
}

// stop and remove the container with a certain name
pub async fn remove_container(name: &str) -> Result<(), Error> {
    // stop the container
    let mut cmd = Command::new("docker");
    cmd.arg("stop").arg(name);
    // stoping the container may return non-zero error code, 
    // simply ignore it
    cmd.output().await?;
    
    // remove the container
    let mut cmd = Command::new("docker");
    cmd.arg("rm").arg(name);
    cmd_runner(cmd).await?;

    Ok(())
}
