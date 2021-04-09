use serde::Deserialize;

// LinkInfo represents an undirected edge connecting one node to another
// LinkInfo is deserialized from the incoming HTTP message
#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct InputLink<T> {
    edge_id: (u64, u64),
    description: T,
}

impl<T> InputLink<T> {
    pub(crate) fn link_id(&self) -> (u64, u64) {
        self.edge_id
    }

    pub(crate) fn _meta(&self) -> &T {
        &self.description
    }
}

// DeviceInfo is deserialized from the incoming HTTP message
#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct InputDevice<T> {
    id: u64,
    description: T,
}

impl<T> InputDevice<T> {
    pub(crate) fn id(&self) -> u64 {
        return self.id;
    }

    pub(crate) fn _meta(&self) -> &T {
        &self.description
    }
}
