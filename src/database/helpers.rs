use std::collections::HashMap;
use std::future::Future;

use indradb::Type;
use indradb::{SpecificVertexQuery, VertexQueryExt};
use indradb::{Vertex, VertexQuery};
use indradb_proto::{ClientError, Transaction};
use uuid::Uuid;

use crate::emunet::{self, ClusterInfo, Emunet, IdAllocator, User};

pub(crate) async fn create_vertex(tran: &mut Transaction, id: Uuid) -> Result<bool, ClientError> {
    let t = Type::new("t").unwrap();
    let v = Vertex::with_id(id, t);

    tran.create_vertex(&v).await
}

pub(crate) async fn get_vertex_json_value(
    tran: &mut Transaction,
    vid: Uuid,
    property_name: &str,
) -> Result<Option<serde_json::Value>, ClientError> {
    let q: VertexQuery = SpecificVertexQuery::single(vid.clone()).into();
    let vertex_list = tran.get_vertices(q).await?;
    if vertex_list.len() == 0 {
        return Ok(None);
    }

    let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v| v.id).collect())
        .property(property_name);
    let mut property_list = tran.get_vertex_properties(q).await?;
    if property_list.len() == 0 {
        return Ok(None);
    }

    Ok(Some(property_list.pop().unwrap().value))
}

pub(crate) async fn set_vertex_json_value(
    tran: &mut Transaction,
    vid: Uuid,
    property_name: &str,
    json: &serde_json::Value,
) -> Result<bool, ClientError> {
    let q: VertexQuery = SpecificVertexQuery::single(vid).into();
    let vertex_list = tran.get_vertices(q).await?;
    if vertex_list.len() == 0 {
        return Ok(false);
    }

    let q = SpecificVertexQuery::new(vertex_list.into_iter().map(|v| v.id).collect())
        .property(property_name);
    tran.set_vertex_properties(q, json).await?;
    Ok(true)
}

pub(crate) async fn delete_vertex(tran: &mut Transaction, vid: Uuid) -> Result<(), ClientError> {
    let q: VertexQuery = SpecificVertexQuery::single(vid).into();
    tran.delete_vertices(q).await
}

pub(crate) async fn get_cluster_info(tran: &mut Transaction) -> Result<ClusterInfo, ClientError> {
    let res = get_vertex_json_value(tran, super::CORE_INFO_ID.clone(), "cluster_info").await?;
    match res {
        Some(jv) => Ok(serde_json::from_value(jv).unwrap()),
        None => panic!("database is not correctly initialized"),
    }
}

pub(crate) async fn set_cluster_info(
    tran: &mut Transaction,
    cluster_info: ClusterInfo,
) -> Result<(), ClientError> {
    let jv = serde_json::to_value(cluster_info).unwrap();
    let res = set_vertex_json_value(tran, super::CORE_INFO_ID.clone(), "cluster_info", &jv).await?;
    if !res {
        panic!("database is not correctly initialized");
    }
    Ok(())
}

pub(crate) async fn get_user_map(
    tran: &mut Transaction,
) -> Result<HashMap<String, User>, ClientError> {
    let res = get_vertex_json_value(tran, super::CORE_INFO_ID.clone(), "user_map").await?;
    match res {
        Some(jv) => Ok(serde_json::from_value(jv).unwrap()),
        None => panic!("database is not correctly initialized"),
    }
}

pub(crate) async fn set_user_map(
    tran: &mut Transaction,
    user_map: HashMap<String, User>,
) -> Result<(), ClientError> {
    let jv = serde_json::to_value(user_map).unwrap();
    let res = set_vertex_json_value(tran, super::CORE_INFO_ID.clone(), "user_map", &jv).await?;
    if !res {
        panic!("database is not correctly initialized");
    }
    Ok(())
}

pub(crate) async fn get_emunet_id_allocator(
    tran: &mut Transaction,
) -> Result<IdAllocator, ClientError> {
    let res =
        get_vertex_json_value(tran, super::CORE_INFO_ID.clone(), "emunet_id_allocator").await?;
    match res {
        Some(jv) => Ok(serde_json::from_value(jv).unwrap()),
        None => panic!("database is not correctly initialized"),
    }
}

pub(crate) async fn set_emunet_id_allocator(
    tran: &mut Transaction,
    id_allocator: IdAllocator,
) -> Result<(), ClientError> {
    let jv = serde_json::to_value(id_allocator).unwrap();
    let res = set_vertex_json_value(
        tran,
        super::CORE_INFO_ID.clone(),
        "emunet_id_allocator",
        &jv,
    )
    .await?;
    if !res {
        panic!("database is not correctly initialized");
    }
    Ok(())
}

pub(crate) async fn get_emunet(
    tran: &mut Transaction,
    emunet_uuid: Uuid,
) -> Result<Option<Emunet>, ClientError> {
    let jv = match get_vertex_json_value(tran, emunet_uuid, emunet::EMUNET_NODE_PROPERTY).await? {
        None => return Ok(None),
        Some(jv) => jv,
    };

    let emunet: Emunet = serde_json::from_value(jv).expect("FATAL: this should not happen");
    Ok(Some(emunet))
}

pub(crate) fn set_emunet<'a>(
    tran: &'a mut Transaction,
    emunet: &Emunet,
) -> impl Future<Output = Result<bool, ClientError>> + Send + 'a {
    let jv = serde_json::to_value(emunet).unwrap();
    let emunet_uuid = emunet.emunet_uuid();

    async move {
        let res =
            set_vertex_json_value(tran, emunet_uuid, emunet::EMUNET_NODE_PROPERTY, &jv).await?;

        Ok(res)
    }
}
