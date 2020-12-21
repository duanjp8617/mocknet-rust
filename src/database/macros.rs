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

macro_rules! succeed {
    ($RequestType: ident,
     $($arg: expr,)+) => {
         Ok(Response::$RequestType(Ok($($arg,)+)))
     }
}

macro_rules! fail {
    ($RequestType: ident, $s: expr) => {
        Ok(Response::$RequestType(Err($s)))
    }
}