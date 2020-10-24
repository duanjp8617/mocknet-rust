// mod indradb;
// mod message_queue;

// pub use self::indradb::IndradbClient;
// pub use self::indradb::IndradbClientError;
// pub use self::indradb::build_client_fut;

// mod resource;

mod new_message_queue;
mod new_indradb;

pub use self::new_indradb::IndradbClient;
pub use self::new_indradb::IndradbClientError;
pub use self::new_indradb::build_client_fut;