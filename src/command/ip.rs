use std::io::{Error};
use tokio::process::Command;

use super::utils::cmd_runner;

pub async fn create_veth_pair(name: &str, peer: &str) -> Result<(), Error> {
    let mut cmd = Command::new("ip");
    cmd.arg("link").arg("add").arg(name).arg("type")
        .arg("veth").arg("peer").arg("name").arg(peer);
    cmd_runner(cmd).await?;
    
    Ok(())
}

pub async fn delete_veth_dev(dev: &str) -> Result<(), Error> {
    let mut cmd = Command::new("ip");
    cmd.arg("link").arg("delete").arg(dev);
    cmd_runner(cmd).await?;
    
    Ok(())
}

pub async fn netns_add_dev(netns: &str, dev: &str) -> Result<(), Error> {
    let mut cmd = Command::new("ip");
    cmd.arg("link").arg("set").arg(dev).arg("netns").arg(netns);
    cmd_runner(cmd).await?;
    
    Ok(())
}

pub async fn create_netns(netns: &str) -> Result<(), Error> {
    let mut cmd = Command::new("ip");
    cmd.arg("netns").arg("add").arg(netns);
    cmd_runner(cmd).await?;
    
    Ok(())
}

pub async fn netns_create_br(netns: &str, br: &str) -> Result<(), Error> {
    let mut cmd = Command::new("ip");
    cmd.arg("netns").arg("exec").arg(netns).arg("ip").arg("link").arg("add")
        .arg("name").arg(br).arg("type").arg("bridge");
    cmd_runner(cmd).await?;
    
    Ok(())
}

pub async fn netns_up_dev(netns: &str, dev: &str) -> Result<(), Error> {
    let mut cmd = Command::new("ip");
    cmd.arg("netns").arg("exec").arg(netns).arg("ip").arg("link").arg("set")
        .arg("dev").arg(dev).arg("up");
    cmd_runner(cmd).await?;
    
    Ok(())
}

pub async fn netns_attach_dev_to_br(netns: &str, dev: &str, br: &str) -> Result<(), Error> {
    let mut cmd = Command::new("ip");
    cmd.arg("netns").arg("exec").arg(netns).arg("ip").arg("link").arg("set")
        .arg("dev").arg(dev).arg("master").arg(br);
    cmd_runner(cmd).await?;
    
    Ok(())
} 

pub async fn create_vxlan_dev(dev: &str, vxlan_id: i32, rip: &str, lip: &str) -> Result<(), Error> {
    // by default, the destination port of a vxlan is 4789
    let dst_port = 4789;

    let mut cmd = Command::new("ip");
    cmd.arg("link").arg("add").arg(dev).arg("type").arg("vxlan")
        .arg("id").arg(vxlan_id.to_string()).arg("dst port").arg(dst_port.to_string())
        .arg("remote").arg(rip).arg("local").arg(lip);
    cmd_runner(cmd).await?;

    Ok(())
}