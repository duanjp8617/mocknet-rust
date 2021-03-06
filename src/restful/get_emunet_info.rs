use std::collections::HashMap;

use indradb_proto::ClientError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::Filter;

use super::Response;
use crate::algo::UndirectedGraph;
use crate::database::{helpers, Client, Connector};
use crate::emunet::{EmunetAccessInfo, OutputDevice, OutputLink};

#[derive(Serialize, Deserialize)]
struct EmunetInfo {
    emunet_id: u8,
    emunet_name: String,
    emunet_uuid: Uuid,
    max_capacity: u64,
    user_name: String,
    access_info: EmunetAccessInfo,
    state: String,
    dev_count: u64,
}

#[derive(Serialize, Deserialize)]
struct ResponseData {
    emunet_info: EmunetInfo,
    devices: Vec<OutputDevice>,
    links: Vec<OutputLink>,
}

#[derive(Deserialize, Serialize)]
struct Request {
    emunet_uuid: Uuid,
}

async fn get_emunet_info(
    req: Request,
    client: &mut Client,
) -> Result<Response<ResponseData>, ClientError> {
    let mut tran = client.guarded_tran().await?;

    let emunet = match helpers::get_emunet(&mut tran, req.emunet_uuid.clone()).await? {
        None => {
            return Ok(Response::fail(format!(
                "emunet {} does not exist",
                req.emunet_uuid
            )))
        }
        Some(emunet) => emunet,
    };
    let (devices, links) = emunet.release_output_emunet();

    let access_info = emunet.access_info();
    let emunet_info = EmunetInfo {
        emunet_id: emunet.emunet_id(),
        emunet_name: emunet.emunet_name().to_string(),
        emunet_uuid: emunet.emunet_uuid().clone(),
        max_capacity: emunet.max_capacity(),
        user_name: emunet.emunet_user().to_string(),
        access_info: EmunetAccessInfo {
            login_server_addr: access_info.login_server_addr.clone(),
            login_server_user: access_info.login_server_user.clone(),
            login_server_pwd: access_info.login_server_pwd.clone(),
        },
        state: emunet.state().into(),
        dev_count: emunet.dev_count(),
    };

    Ok(Response::success(ResponseData {
        emunet_info,
        devices,
        links,
    }))
}

async fn guard(req: Request, mut client: Client) -> Result<warp::reply::Json, warp::Rejection> {
    let res = get_emunet_info(req, &mut client).await;
    match res {
        Ok(resp) => Ok(resp.into()),
        Err(e) => {
            client.notify_failure();
            let resp: Response<_> = e.into();
            Ok(resp.into())
        }
    }
}

pub fn build_filter(
    connector: Connector,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + Send {
    super::filter_template("get_emunet_info".to_string(), connector, guard)
}

pub async fn mnctl_network_info(user: &str, emunet: &str, warp_addr: &str) -> Result<(), String> {
    // query emunet_uuid
    let req = super::list_emunet::Request {
        user: user.to_string(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/list_emunet", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<HashMap<String, Uuid>> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;
    let map = if response.success {
        response.data.unwrap()
    } else {
        return Err(response.message);
    };
    let emunet_uuid = map
        .get(emunet)
        .ok_or(format!("emunet {} does not exist", emunet))?;

    // get the response data from get_emunet_info
    let req = Request {
        emunet_uuid: emunet_uuid.clone(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/get_emunet_info", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<ResponseData> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;

    if response.success {
        let data = response.data.unwrap();

        println!("emunet uuid: {}", &data.emunet_info.emunet_uuid);
        println!("state: {}", &data.emunet_info.state);
        println!("max capacity: {}", data.emunet_info.max_capacity);
        println!("active devices: {}", data.emunet_info.dev_count);
        println!(
            "login server address: {}",
            &data.emunet_info.access_info.login_server_addr
        );
        println!(
            "login username: {}",
            &data.emunet_info.access_info.login_server_user
        );
        println!(
            "login password: {}",
            &data.emunet_info.access_info.login_server_pwd
        );

        print!("device list: ");
        for i in 0..data.devices.len() {
            let dev = &data.devices[i];
            if i < data.devices.len() - 1 {
                print!("{}, ", dev.id);
            } else {
                print!("{}\n", dev.id);
            }
        }

        Ok(())
    } else {
        Err(response.message)
    }
}

pub async fn mnctl_network_dev(
    user: &str,
    emunet: &str,
    dev_id: u64,
    warp_addr: &str,
) -> Result<(), String> {
    // query emunet_uuid
    let req = super::list_emunet::Request {
        user: user.to_string(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/list_emunet", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<HashMap<String, Uuid>> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;
    let map = if response.success {
        response.data.unwrap()
    } else {
        return Err(response.message);
    };
    let emunet_uuid = map
        .get(emunet)
        .ok_or(format!("emunet {} does not exist", emunet))?;

    // get the response data from get_emunet_info
    let req = Request {
        emunet_uuid: emunet_uuid.clone(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/get_emunet_info", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<ResponseData> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;

    if response.success {
        let data = response.data.unwrap();
        let mut dev_map = HashMap::new();
        let _: Vec<_> = data
            .devices
            .iter()
            .map(|odev| {
                assert!(dev_map.insert(odev.id, odev).is_none() == true);
            })
            .collect();

        let dev = *dev_map
            .get(&dev_id)
            .ok_or(format!("device {} does not exist", dev_id))?;

        println!(
            "login ip: {}",
            dev.pod_login_ip
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or("null".to_string())
        );
        println!(
            "login username: {}",
            dev.pod_login_user
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or("null".to_string())
        );
        println!(
            "login password: {}",
            dev.pod_login_pwd
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or("null".to_string())
        );

        println!("link list: ");
        for link in dev.links.iter() {
            println!(
                "pair device id: {}, intface name: {}, IP address: {}",
                link.dest_dev_id, link.intf_name, link.ip
            )
        }

        Ok(())
    } else {
        Err(response.message)
    }
}

pub async fn mnctl_network_path(
    user: &str,
    emunet: &str,
    src_id: u64,
    dst_id: u64,
    warp_addr: &str,
) -> Result<(Option<Vec<u64>>, uuid::Uuid), String> {
    // query emunet_uuid
    let req = super::list_emunet::Request {
        user: user.to_string(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/list_emunet", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<HashMap<String, Uuid>> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;
    let mut map = if response.success {
        response.data.unwrap()
    } else {
        return Err(response.message);
    };
    let emunet_uuid = map
        .remove(emunet)
        .ok_or(format!("emunet {} does not exist", emunet))?;

    // get the response data from get_emunet_info
    let req = Request {
        emunet_uuid: emunet_uuid.clone(),
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/get_emunet_info", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<ResponseData> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;

    if response.success {
        let data = response.data.unwrap();

        let nodes: Vec<(u64, ())> = data.devices.iter().map(|odev| (odev.id, ())).collect();
        let edges: Vec<((u64, u64), ())> =
            data.links.iter().map(|olink| (olink.link_id, ())).collect();
        let graph = UndirectedGraph::new(nodes, edges).unwrap();

        Ok((graph.shortest_path(src_id, dst_id), emunet_uuid))
    } else {
        Err(response.message)
    }
}

pub async fn mnctl_network_connect(
    user: &str,
    emunet: &str,
    src_id: u64,
    dst_id: u64,
    is_add: bool,
    warp_addr: &str,
) -> Result<(), String> {
    // compute the network path
    let (res, emunet_uuid) = mnctl_network_path(user, emunet, src_id, dst_id, warp_addr).await?;
    let path = res.ok_or(format!(
        "there is no path between {} and {}",
        src_id, dst_id
    ))?;

    // check length of the path
    if path.len() < 3 {
        return Err(format!("{} and {} have direct connection", src_id, dst_id));
    }

    // notify user
    let op = if is_add { "add" } else { "delete" };
    let mut path_string = String::new();
    for i in 0..path.len() {
        if i < path.len() - 1 {
            path_string = path_string + &format!("{}, ", path[i]);
        } else {
            path_string = path_string + &format!("{}", path[i]);
        }
    }
    println!("{} route for path: {}", op, path_string);

    // retrieve the route commands
    let req = super::route_command::Request {
        emunet_uuid: emunet_uuid.clone(),
        path,
        is_add,
    };
    let http_resp = reqwest::Client::new()
        .post(format!("http://{}/v1/route_command", warp_addr))
        .json(&req)
        .send()
        .await
        .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
    let response: Response<super::route_command::RespData> = http_resp
        .json()
        .await
        .map_err(|_| format!("can not parse JSON response"))?;

    let route_commands = if response.success {
        response.data.unwrap()
    } else {
        return Err(response.message);
    };

    // now apply the forward and backward commands
    for (dev_idx, cmd) in route_commands.forward_route_commands {
        println!("device {}: {}", dev_idx, &cmd);
        let req = super::execute_command::Request {
            emunet_uuid: emunet_uuid.clone(),
            dev_idx,
            cmd,
            api_server_addr: route_commands.api_server_addr.clone(),
        };
        let http_resp = reqwest::Client::new()
            .post(format!("http://{}/v1/execute_command", warp_addr))
            .json(&req)
            .send()
            .await
            .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
        let response: Response<String> = http_resp
            .json()
            .await
            .map_err(|_| format!("can not parse JSON response"))?;
        if response.success == false {
            return Err(response.message);
        }
        let err_msg = response.data.unwrap();
        if err_msg.len() == 0 {
            println!("done");
        } else {
            println!("conflict, cancelled");
        }
    }
    for (dev_idx, cmd) in route_commands.backward_route_commands {
        println!("device {}: {}", dev_idx, &cmd);
        let req = super::execute_command::Request {
            emunet_uuid: emunet_uuid.clone(),
            dev_idx,
            cmd,
            api_server_addr: route_commands.api_server_addr.clone(),
        };
        let http_resp = reqwest::Client::new()
            .post(format!("http://{}/v1/execute_command", warp_addr))
            .json(&req)
            .send()
            .await
            .map_err(|_| format!("can not send HTTP request to {}", warp_addr))?;
        let response: Response<String> = http_resp
            .json()
            .await
            .map_err(|_| format!("can not parse JSON response"))?;
        if response.success == false {
            return Err(response.message);
        }
        let err_msg = response.data.unwrap();
        if err_msg.len() == 0 {
            println!("done");
        } else {
            println!("conflict, cancelled");
        }
    }

    if req.is_add {
        println!(
            "device {}:{} <--> device{}:{}",
            route_commands.src_idx,
            route_commands.src_ip,
            route_commands.dest_idx,
            route_commands.dest_ip
        );
    } else {
        println!(
            "device {}:{} <-\\-> device{}:{}",
            route_commands.src_idx,
            route_commands.src_ip,
            route_commands.dest_idx,
            route_commands.dest_ip
        );
    }

    Ok(())
}
