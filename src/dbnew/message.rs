use std::future::Future;
use std::pin::Pin;

use uuid::Uuid;

use super::backend::IndradbClientBackend;
use super::errors::BackendError;

pub type QueryResult<T> = Result<T, String>;
pub use Result::Ok as Succeed;
pub use Result::Err as Fail;

pub type ResponseFuture<'a> = Pin<Box<dyn Future<Output = Result<Response, BackendError>> + 'a>>;
/// Every message sends to the indradb backend should implement this trait.
/// We use this trait to emulate polymorphism.
pub trait DatabaseMessage<Response, Error> {
    /// When implementing this trait, the implementor should wrap the actual query task
    /// in a boxed future and returns the boxed future.
    fn execute<'a>(&mut self, backend: &'a IndradbClientBackend) -> ResponseFuture<'a>;
}

/// The response that is delivered to the client.
/// 
/// Note: this response does not contain fatal errors generated by 
/// the backend.
pub enum Response {
    Init(QueryResult<()>),
    RegisterUser(QueryResult<()>),
    CreateEmuNet(QueryResult<Uuid>),
}

/// The request that is sent from the client to the indradb backend.
pub type Request = Box<dyn DatabaseMessage<Response, BackendError> + Send + 'static>;