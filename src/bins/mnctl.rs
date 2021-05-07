use std::error::Error as StdError;

use mocknet::cli::*;
use mocknet::restful::*;

// async fn user_history(username: &str, warp_addr: &str) -> Result<(), reqwest::Error> {
//     let req = Request { name: username.to_string() };
//     let response = reqwest::Client::new()
//         .post(format!("{}/v1/list_user_history", warp_addr))
//         .json(&req)
//         .send()
//         .await?;

//     Ok(())
// }

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
                match emunet_update::mnctl_user_update(
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
            _ => {}
        },
    }

    Ok(())
}
