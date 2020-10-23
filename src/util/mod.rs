// the following mode are taken from indradb codebase and 
// adapted for mocknet
mod uuid_related;
pub use uuid_related::new_uuid;
pub(crate) mod converters;
mod client_datastore;