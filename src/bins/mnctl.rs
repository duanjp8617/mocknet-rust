use std::error::Error as StdError;

use mocknet::cli::*;
use mocknet::restful::list_user_history;

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
            let res = list_user_history::manual_request(&arg.user, &arg.warp_addr).await;
            match res {
                Err(msg) => println!("{}", msg),
                _ => {}
            };
        }
        _ => {
            println!("wtf?");
        }
    }

    Ok(())
}
