pub mod docker;

pub mod system;

mod ip_cmd;
pub use ip_cmd::VethPair;
pub use ip_cmd::AssignPortNs;
pub use ip_cmd::AddNs;
pub use ip_cmd::AddBrInNs;
pub use ip_cmd::UpPortInNs;
pub use ip_cmd::AttachPortToBrInNs;
pub use ip_cmd::AddVxlanPort;

mod utils;

pub fn fuck() {
    println!("fuck you");
}