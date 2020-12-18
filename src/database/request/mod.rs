mod init;
pub use init::Init;

mod register_user;
pub use register_user::RegisterUser;

mod create_emu_net;
pub use create_emu_net::CreateEmuNet;

mod list_emu_net;
pub use list_emu_net::ListEmuNet;

mod get_emu_net;
pub use get_emu_net::GetEmuNet;

mod set_emu_net;
pub use set_emu_net::SetEmuNet;

mod init_emu_net;

use crate::database::message::{Response, DatabaseMessage, Request};
use crate::database::errors::BackendError;
pub fn build_request<T: DatabaseMessage<Response, BackendError> + Send + 'static>(t: T) -> Request {
    Box::new(t)
}