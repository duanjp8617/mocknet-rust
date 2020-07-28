use std::io::Error;
use super::command::*;

pub async fn create_dev(dev: &str, img: &str, port_num: u32) -> Result<(), Error> {
    // create the phy container
    let phy_pid = docker::launch_container(dev, img).await?;
    
    // create a network namespace link for the phy container
    system::create_netns_link(phy_pid).await?;

    for port_id in 0..port_num {
        // name of the port that is attached to the netns of phy
        // naming convention: dev_a0, dev_a1, .. , dev_a32
        let attach_port = format!("{}_a{}", dev, port_id);
        
        // name of the port that is free for connection
        // naming convention: dev_p0, dev_p1, .. , dev_p32
        let free_port = format!("{}_p{}", dev, port_id);

        // create a veth pair using the port name
        ip::create_veth_pair(&attach_port, &free_port).await?;

        // attach the attach port to phy netns
        ip::netns_add_dev(&phy_pid.to_string(), &attach_port).await?;
    }
    
    
    Ok(())
}