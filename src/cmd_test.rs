use mocknet::command::docker;
use mocknet::command::system;
use std::io::ErrorKind;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = system::create_dir("/var/run/netns").await;
    match res {
        Err(io_err) if io_err.kind() == ErrorKind::InvalidData => {},
        Err(io_err) => {
            return Err(io_err.into());
        }
        Ok(_) => {}
    };

    let pid_c1 = docker::launch_container("c1", "ubuntu").await?;
    println!("pid is {}", pid_c1);
    system::link_netns(pid_c1).await?;


    let pid_c2 = docker::launch_container("c2", "ubuntu").await?;
    println!("pid is {}", pid_c2);
    system::link_netns(pid_c2).await?;


    // docker::remove_container("c1").await?;
    // docker::remove_container("c2").await?;

    Ok(())
}