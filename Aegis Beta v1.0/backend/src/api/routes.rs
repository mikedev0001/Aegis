use std::sync::Arc;
use warp::Filter;

use crate::vm::manager::VMManager;
use super::handlers;

pub fn setup_routes(vm_manager: Arc<VMManager>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let vm_manager_filter = warp::any().map(move || vm_manager.clone());

    // API routes
    let api = warp::path("api");

    // Health check
    let health = api
        .and(warp::path("health"))
        .and(warp::get())
        .and_then(handlers::health_check);

    // VM management
    let list_vms = api
        .and(warp::path("vms"))
        .and(warp::get())
        .and(vm_manager_filter.clone())
        .and_then(handlers::list_vms);

    let get_vm = api
        .and(warp::path("vms"))
        .and(warp::path::param())
        .and(warp::get())
        .and(vm_manager_filter.clone())
        .and_then(handlers::get_vm);

    let create_vm = api
        .and(warp::path("vms"))
        .and(warp::post())
        .and(warp::body::json())
        .and(vm_manager_filter.clone())
        .and_then(handlers::create_vm);

    let start_vm = api
        .and(warp::path("vms"))
        .and(warp::path::param())
        .and(warp::path("start"))
        .and(warp::post())
        .and(vm_manager_filter.clone())
        .and_then(handlers::start_vm);

    let stop_vm = api
        .and(warp::path("vms"))
        .and(warp::path::param())
        .and(warp::path("stop"))
        .and(warp::post())
        .and(vm_manager_filter.clone())
        .and_then(handlers::stop_vm);

    let delete_vm = api
        .and(warp::path("vms"))
        .and(warp::path::param())
        .and(warp::delete())
        .and(vm_manager_filter.clone())
        .and_then(handlers::delete_vm);

    let get_vnc = api
        .and(warp::path("vms"))
        .and(warp::path::param())
        .and(warp::path("vnc"))
        .and(warp::get())
        .and(vm_manager_filter.clone())
        .and_then(handlers::get_vnc_url);

    // ISO management
    let upload_iso = api
        .and(warp::path("isos"))
        .and(warp::path("upload"))
        .and(warp::post())
        .and(vm_manager_filter.clone())
        .and(warp::body::bytes())
        .and_then(handlers::upload_iso);

    // Static files
    let static_files = warp::fs::dir("./frontend");

    // Combine all routes
    health
        .or(list_vms)
        .or(get_vm)
        .or(create_vm)
        .or(start_vm)
        .or(stop_vm)
        .or(delete_vm)
        .or(get_vnc)
        .or(upload_iso)
        .or(static_files)
        .with(warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "DELETE"])
            .allow_headers(vec!["Content-Type"]))
        .with(warp::log("vm_manager"))
}