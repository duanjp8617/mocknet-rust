/// https://danielkeep.github.io/quick-intro-to-macros.html#some-more-gotchas
/// can be a good source for learning macro
macro_rules! generate_request {
    ($who: ident,
     $RequestType: ident,
     $($arg: expr,)+) => {
         match $who.sender.send(Box::new(request::$RequestType::new($($arg,)+))).await? {
             Response::$RequestType(inner) => Ok(inner),
             _ => panic!("invalid response")
         }
     }
}