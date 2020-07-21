use tokio::process::Command;

pub struct PhyContainer {
    cmd : Command
}

impl PhyContainer {
    pub fn new(name: &str, image: &str) -> Self {
        let mut cmd = Command::new("docker");
        
        cmd.arg("run")                     // run a container with command
            .arg("--net=none")             // with no default network
            .arg("--privileged")           // with root priviledge            
            .arg("-itd")                   // in interactive and detached mode
            .arg("--entrypoint=/bin/bash") // set entrypoint program to /bin/bash
            .arg("--name")                 // set container name
            .arg(name)
            .arg(image);                   // set container image

        PhyContainer {
            cmd : cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

pub struct PIDChecker {
    cmd : Command
}

impl PIDChecker {
    pub fn new(name: &str) -> Self {
        let mut cmd = Command::new("docker");
        
        // inspect the process ID of the container
        cmd.arg("docker inspect -f {{.State.Pid}}").arg(name);
        
        PIDChecker {
            cmd : cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}