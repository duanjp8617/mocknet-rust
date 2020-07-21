use tokio::process::Command;

pub struct CreateNetNSDir {
    cmd : Command
}

impl CreateNetNSDir {
    pub fn new() -> Self {
        let mut cmd = Command::new("mkdir");
        
        // create /var/run/netns directory
        cmd.arg("-p /var/run/netns");

        CreateNetNSDir {
            cmd: cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

pub struct LinkNS {
    cmd : Command
}

impl LinkNS {
    pub fn new(container_pid: i32) -> Self {
        let mut cmd = Command::new("ln");

        let src_dir = format!("/proc/{}/ns/net", container_pid);
        let dst_dir = format!("/var/run/netns/{}", container_pid);

        cmd.arg("-s").arg(src_dir).arg(dst_dir);

        LinkNS {
            cmd : cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

