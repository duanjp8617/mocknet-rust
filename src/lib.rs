// first, build a library with detailed comments, following the style of https://github.com/nayuki/QR-Code-generator
// First, I will use the lagacy command-based approach to build up the virtual network and expose required APIs
// Then, I will use netlink and shiplift to implement an accelarated way for building the virtual network using the exact same set of APIs

mod enet;
pub mod container_backend;
pub mod backend;