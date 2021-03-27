
// runs inside a task to do the lazy connection
struct ConnectorBackend {
    
}

// this is actually a sender that sends the request to the Connector Backend
// and listens for the returned message. The message is the connected client
struct Connector {

}

// this is actually a wrapper for indradb-proto::proto::Client, it can be used to deliver 
// a connection signal when it is dropped
struct Client {

}