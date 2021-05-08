use std::error::Error as StdError;

use mocknet::cli::*;
use mocknet::restful::*;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn StdError>> {
    let arg = match parse_ctl_arg() {
        Ok(arg) => arg,
        Err(msg) => {
            println!("{}", msg);
            return Ok(());
        }
    };

    match arg.subcmd {
        UserSubcmd::History => {
            match list_user_history::mnctl_user_history(&arg.user, &arg.warp_addr).await {
                Err(msg) => println!("{}", msg),
                _ => {}
            };
        }
        UserSubcmd::NetworkOp(emunet_name, subcmd) => match subcmd {
            NetworkSubcmd::Update(input_file) => {
                match emunet_update::mnctl_network_update(
                    &arg.user,
                    &emunet_name,
                    &input_file,
                    &arg.warp_addr,
                )
                .await
                {
                    Err(msg) => println!("{}", msg),
                    _ => {}
                };
            }
            NetworkSubcmd::Restore(history_index) => {
                match emunet_update::mnctl_network_restore(
                    &arg.user,
                    &emunet_name,
                    history_index as usize,
                    &arg.warp_addr,
                )
                .await
                {
                    Err(msg) => println!("{}", msg),
                    _ => {}
                }
            }
            NetworkSubcmd::Info => {
                match get_emunet_info::mnctl_network_info(&arg.user, &emunet_name, &arg.warp_addr)
                    .await
                {
                    Err(msg) => println!("{}", msg),
                    _ => {}
                }
            }
            NetworkSubcmd::Dev(dev_id) => {
                match get_emunet_info::mnctl_network_dev(
                    &arg.user,
                    &emunet_name,
                    dev_id as usize,
                    &arg.warp_addr,
                )
                .await
                {
                    Err(msg) => println!("{}", msg),
                    _ => {}
                }
            }
            NetworkSubcmd::Path(mut src_id, mut dst_id) => {
                if src_id > dst_id {
                    let temp = src_id;
                    src_id = dst_id;
                    dst_id = temp;
                }
                match get_emunet_info::mnctl_network_path(
                    &arg.user,
                    &emunet_name,
                    src_id,
                    dst_id,
                    &arg.warp_addr,
                )
                .await
                {
                    Err(msg) => println!("{}", msg),
                    Ok((path, _)) => match path {
                        None => println!("there is no path between {} and {}", src_id, dst_id),
                        Some(path) => {
                            for i in 0..path.len() {
                                if i < path.len() - 1 {
                                    print!("{}, ", path[i]);
                                } else {
                                    print!("{}\n", path[i]);
                                }
                            }
                        }
                    },
                }
            }
            _ => {}
        },
    }

    Ok(())
}
