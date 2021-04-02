// struct TopologyMeta {
//     name: String, // same as the pod name that this link belongs to
// }

// struct TopologyLink {
//     uid: u64 // id of the link, globally unique, non-repeatable
//     peer_pod: String,
//     local_intf: String, // interface name of this link in the pod
//     peer_intf: String,  // interface name of the peer link on the peer pod
//     local_ip: String,   // ip address of the local_intf belonging to the same subnet
//     peer_ip: String,    // ip address of the peer intf belonging to the same subnet
// }

// struct TopologyLinks {
//     links: Vec<TopologyLink>
// }

// struct Topology {
//     metadata: TopologyMeta,
//     spec: TopologyLinks,
// }

// struct PodMeta {
//     name: String // name of the pod
// }

// struct PodSpec {
//     nodeSelector: String // k8s node name
// }

// struct Pod {
//     metadata: PodMeta,
//     spec: PodSpec
// }

// enum Item {
//     T(Topology),
//     P(Pod),
// }

// struct Backend {
//     items: Vec<Item> 
// }

pub mod mocknet_proto {
    tonic::include_proto!("mocknet_proto"); // The string specified here must match the proto package name
}
