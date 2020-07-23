use mocknet::command::docker;
use mocknet::command::system;
use std::io::ErrorKind;

async fn test1() -> Result<(), std::io::Error> {
    let res = system::create_dir("/var/run/netns").await;
    match res {
        Err(io_err) if io_err.kind() == ErrorKind::InvalidData => {},
        Err(io_err) => {
            return Err(io_err.into());
        }
        Ok(_) => {}
    };

    let pid_c1 = docker::launch_container("c1", "ubuntu").await?;
    system::create_netns_link(pid_c1).await?;
    println!("creating netns link for container {}", pid_c1);


    let pid_c2 = docker::launch_container("c2", "ubuntu").await?;
    system::create_netns_link(pid_c2).await?;
    println!("creating netns link for container {}", pid_c2);


    docker::remove_container("c1").await?;
    docker::remove_container("c2").await?;
    println!("removing both the two containers");

    system::remove_netns_link(pid_c1).await?;
    system::remove_netns_link(pid_c2).await?;
    println!("removing netns link");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test1().await?;
    Ok(())
}