use std::collections::HashMap;

use indradb::BulkInsertItem;
use indradb::Type;
use indradb::{RangeVertexQuery, SpecificVertexQuery, VertexQueryExt};
use indradb::{Vertex, VertexQuery};
use indradb_proto::{Client, ClientError, Transaction};
use uuid::Uuid;

use crate::new_emunet::cluster::ClusterInfo;
use crate::new_emunet::user::User;

pub(crate) async fn create_vertex(
    tran: &mut Transaction,
    id: Option<Uuid>,
) -> Result<Option<Uuid>, ClientError> {
    let t = Type::new("t").unwrap();
    let v = match id {
        Some(id) => Vertex::with_id(id, t),
        None => Vertex::with_id(indradb::util::generate_uuid_v1(), t),
    };

    let succeed = tran.create_vertex(&v).await?;
    if succeed {
        Ok(Some(v.id))
    } else {
        Ok(None)
    }
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

// This is not really required, I will delete it when necessary
pub(crate) async fn bulk_insert(
    client: &mut Client,
    qs: Vec<BulkInsertItem>,
) -> Result<(), ClientError> {
    client.bulk_insert(qs.into_iter()).await
}

// This is not needed as well
pub(crate) async fn get_vertex_properties(
    tran: &mut Transaction,
    q: RangeVertexQuery,
) -> Result<Vec<serde_json::Value>, ClientError> {
    let q = q.property("default".to_string());
    tran.get_vertex_properties(q).await.map(|vp| {
        vp.into_iter().fold(Vec::new(), |mut vec, vp| {
            vec.push(vp.value);
            vec
        })
    })
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