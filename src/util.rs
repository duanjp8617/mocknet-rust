use uuid::Uuid;

pub fn new_uuid() -> Uuid {
    use indradb::util;
    util::generate_uuid_v1()
}