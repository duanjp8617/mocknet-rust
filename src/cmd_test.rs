use tokio::process::Command;

struct DockerPs {
    cmd : Command
}

impl DockerPs {
    pub fn new(temp : String) -> Self {
        let mut cmd = Command::new("docker");
        cmd.arg(temp).arg("-a");
        
        DockerPs {
            cmd : cmd
        }
    }

    async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = DockerPs::new(String::from("ps"));
    
    // Make sure our child succeeded in spawning and process the result
    let output = cmd.run().await?;

    // Await until the future (and the command) completes
    println!("{}", String::from_utf8(output.stdout.clone()).unwrap());

    Ok(())
}