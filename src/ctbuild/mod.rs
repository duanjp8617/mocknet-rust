use std::io::Error;

pub async fn create_dev(dev: &str, img: &str, port_num: u32) -> Result<(), Error> {
    Ok(())
}

pub async fn remove_dev(dev: &str) -> Result<(), Error> {
    Ok(())
}

pub async fn create_local_link(src_dev: &str, src_pid: u32, dst_dev: &str, dst_pid: u32) -> Result<(), Error> {
    Ok(())
}

pub async fn remove_local_link(src_dev: &str, src_pid: u32, dst_dev: &str, dst_pid: u32) -> Result<(), Error> {
    Ok(())
}

pub async fn create_remote_link(src_dev: &str, src_pid: u32, src_ip: &str, dst_ip: &str, vxlan_id: u32) -> Result<(), Error> {
    Ok(())
}

pub async fn remove_remote_link(src_dev: &str, src_pid: u32, src_ip: &str, dst_ip: &str, vxlan_id: u32) -> Result<(), Error> {
    Ok(())
}





