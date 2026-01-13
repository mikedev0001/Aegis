use std::sync::Arc;
use warp::{Rejection, Reply};
use serde_json::json;

use crate::vm::manager::VMManager;
use crate::vm::config::{VMConfig, CreateVMRequest, UpdateVMRequest};
use crate::security::validation::validate_vm_config;

pub async fn list_vms(
    vm_manager: Arc<VMManager>
) -> Result<impl Reply, Rejection> {
    let vms = vm_manager.list_vms().await;
    Ok(warp::reply::json(&vms))
}

pub async fn get_vm(
    vm_id: String,
    vm_manager: Arc<VMManager>
) -> Result<impl Reply, Rejection> {
    match vm_manager.get_vm(&vm_id).await {
        Some(vm) => Ok(warp::reply::json(&vm)),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn create_vm(
    body: CreateVMRequest,
    vm_manager: Arc<VMManager>
) -> Result<impl Reply, Rejection> {
    // Validate input
    if let Err(err) = validate_vm_config(&body) {
        return Ok(warp::reply::json(&json!({
            "error": err.to_string()
        })));
    }

    match vm_manager.create_vm(body).await {
        Ok(vm) => Ok(warp::reply::json(&vm)),
        Err(err) => Ok(warp::reply::json(&json!({
            "error": err.to_string()
        }))),
    }
}

pub async fn start_vm(
    vm_id: String,
    vm_manager: Arc<VMManager>
) -> Result<impl Reply, Rejection> {
    match vm_manager.start_vm(&vm_id).await {
        Ok(_) => Ok(warp::reply::json(&json!({
            "success": true,
            "message": format!("VM {} started", vm_id)
        }))),
        Err(err) => Ok(warp::reply::json(&json!({
            "error": err.to_string()
        }))),
    }
}

pub async fn stop_vm(
    vm_id: String,
    vm_manager: Arc<VMManager>
) -> Result<impl Reply, Rejection> {
    match vm_manager.stop_vm(&vm_id).await {
        Ok(_) => Ok(warp::reply::json(&json!({
            "success": true,
            "message": format!("VM {} stopped", vm_id)
        }))),
        Err(err) => Ok(warp::reply::json(&json!({
            "error": err.to_string()
        }))),
    }
}

pub async fn delete_vm(
    vm_id: String,
    vm_manager: Arc<VMManager>
) -> Result<impl Reply, Rejection> {
    match vm_manager.delete_vm(&vm_id).await {
        Ok(_) => Ok(warp::reply::json(&json!({
            "success": true,
            "message": format!("VM {} deleted", vm_id)
        }))),
        Err(err) => Ok(warp::reply::json(&json!({
            "error": err.to_string()
        }))),
    }
}

pub async fn get_vnc_url(
    vm_id: String,
    vm_manager: Arc<VMManager>
) -> Result<impl Reply, Rejection> {
    match vm_manager.get_vnc_url(&vm_id).await {
        Some(url) => Ok(warp::reply::json(&json!({
            "url": url
        }))),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn upload_iso(
    vm_manager: Arc<VMManager>,
    body: bytes::Bytes,
) -> Result<impl Reply, Rejection> {
    // TODO: Implement ISO upload
    Ok(warp::reply::json(&json!({
        "error": "Not implemented"
    })))
}

pub async fn health_check() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}