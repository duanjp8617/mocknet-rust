fn fuck() {
    println!("eee");
}

struct ResourcePool {

}

struct DevRecord {

}

struct DBConn {

}

struct ENetInner {
    rp: ResourcePool,
    dr: DevRecord,
    dc: DBConn,
}

struct ENetBuilder {
    inner: ENetInner,
}

struct ENet {
    inner: ENetInner,
}





