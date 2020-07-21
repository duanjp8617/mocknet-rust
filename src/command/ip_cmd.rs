use tokio::process::Command;

pub struct VethPair {
    cmd : Command
}

impl VethPair {
    pub fn new(name: &str, peer: &str) -> Self {
        let mut cmd = Command::new("ip");

        // create a veth pair
        cmd.arg("link add").arg(name).arg("type veth peer name").arg(peer);

        VethPair {
            cmd : cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

pub struct AssignPortNs {
    cmd : Command
}

impl AssignPortNs {
    pub fn new(name: &str, pid: i32) -> Self {
        let mut cmd = Command::new("ip");

        // assign a port to a namespace
        cmd.arg("link set").arg(name).arg("netns").arg(pid.to_string());

        AssignPortNs {
            cmd: cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

pub struct AddNs {
    cmd : Command
}

impl AddNs {
    pub fn new(name: &str) -> Self {
        let mut cmd = Command::new("ip");

        // create a new network namespace
        cmd.arg("netns add").arg(name);

        AddNs {
            cmd: cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

pub struct AddBrInNs {
    cmd: Command,
}

impl AddBrInNs {
    pub fn new(netns: &str, br: &str) -> Self {
        let mut cmd = Command::new("ip");

        // create a bridge in a given network namespace
        cmd.arg("netns exec ").arg(netns).arg("ip link add name").arg(br).arg("type bridge");

        AddBrInNs {
            cmd: cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

pub struct UpPortInNs {
    cmd: Command
}

impl UpPortInNs {
    pub fn new(port: &str, netns: &str) -> Self {
        let mut cmd = Command::new("ip");

        // bring a port up in a given network namespace
        cmd.arg("netns exec").arg(netns).arg("ip link set dev").arg(port).arg("up");

        UpPortInNs {
            cmd: cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

pub struct AttachPortToBrInNs {
    cmd: Command
}

impl AttachPortToBrInNs {
    pub fn new(port: &str, br: &str, netns: &str) -> Self {
        let mut cmd = Command::new("ip");

        // attach a port to a bridge in a given network namespace
        cmd.arg("netns exec").arg(netns).arg("ip link set dev").arg(port).arg("master").arg(br);

        AttachPortToBrInNs {
            cmd: cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}

pub struct AddVxlanPort {
    cmd: Command
}

impl AddVxlanPort {
    pub fn new(port: &str, id: i32, remote_ip: &str, local_ip: &str) -> Self {
        let mut cmd = Command::new("ip");
        
        // by default, the destination port of a vxlan is 4789
        let dst_port = 4789;

        // create a vxlan port
        cmd.arg("link add").arg(port).arg("type vxlan id").arg(id.to_string()).arg(dst_port.to_string())
            .arg("remote").arg(remote_ip).arg("local").arg(local_ip);

        AddVxlanPort {
            cmd: cmd
        }
    }

    pub async fn run(mut self) -> Result<std::process::Output, std::io::Error> {
        self.cmd.output().await
    }
}