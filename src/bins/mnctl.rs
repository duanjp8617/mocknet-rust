use std::error::Error as StdError;

use mocknet::cli::*;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn StdError>> {
    let arg = parse_ctl_arg();
    println!("{:?}", arg);
    Ok(())
}
