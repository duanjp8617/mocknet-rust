use std::io::{Error, ErrorKind};
use tokio::process::Command;

// execute a command, return the stdout if the command succeeds,
// otherwise build a std::io::Error using the stderr
pub async fn cmd_runner(mut cmd: Command) -> Result<Vec<u8>, Error> {
    let cmd_output = cmd.output().await?;
    if !cmd_output.status.success() {
        let err_msg = String::from_utf8(cmd_output.stderr).unwrap();
        return Err(Error::new(ErrorKind::InvalidData, err_msg));
    }
    Ok(cmd_output.stdout)
}
