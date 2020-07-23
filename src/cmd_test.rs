use mocknet::command::docker;
use mocknet::command::system;
use mocknet::command::ip;
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

async fn test2() ->Result<(), std::io::Error> {
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

    ip::create_veth_pair("c1_sport", "c1_dport").await?;
    ip::create_veth_pair("c2_sport", "c2_dport").await?;

    ip::netns_add_dev(&pid_c1.to_string(), "c1_sport").await?;
    ip::netns_add_dev(&pid_c2.to_string(), "c2_sport").await?;

    let netns = "br_holder";
    ip::create_netns(netns).await?;
    ip::netns_add_dev(netns, "c1_dport").await?;
    ip::netns_add_dev(netns, "c2_dport").await?;

    ip::netns_create_br(netns, "br").await?;
    ip::netns_attach_dev_to_br(netns, "c1_dport", "br").await?;
    ip::netns_attach_dev_to_br(netns, "c2_dport", "br").await?;

    ip::netns_up_dev(netns, "c1_dport").await?;
    ip::netns_up_dev(netns, "c2_dport").await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test2().await?;
    Ok(())
}