// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod elevate;

use anyhow::Context;
use bytes::Bytes;
#[cfg(target_os = "android")]
use easytier::instance::factory::subscribe_native_instance_event;
use easytier::proto::api::config::{
    ConfigPatchAction, ConfigRpc, ConfigRpcClientFactory, VpnPortalClientPatch,
};
use easytier::proto::api::instance::{
    GetVpnPortalInfoRequest, InstanceIdentifier, VpnPortalInfo, VpnPortalRpc,
    VpnPortalRpcClientFactory, instance_identifier,
};
use easytier::proto::api::manage::{
    CollectNetworkInfoResponse, ValidateConfigResponse, VpnPortalClientConfig, WebClientService,
    WebClientServiceClientFactory,
};
use easytier::proto::rpc_types::controller::BaseController;
use easytier::web_client::{self, WebClient};
use easytier::{
    common::config::{NetworkConfig, NetworkConfigExt},
    common::{
        config::{ConfigLoader, ConfigSource, FileLoggerConfig, LoggingConfig, TomlConfigLoader},
        log,
    },
    instance::factory::{NativeInstanceManager, native_instance_manager},
    proto::rpc::standalone::{runtime_rpc_dialer, runtime_rpc_listener},
    rpc_service::ApiRpcServer,
    utils::panic::setup_panic_handler,
};
use easytier_core::management::config_source_to_rpc;
use easytier_core::management::remote_client::{
    GetNetworkMetasResponse, ListNetworkInstanceIdsJsonResp, ListNetworkProps, RemoteClientManager,
    Storage,
};
use easytier_core::{
    connectivity::protocol::raw::TunnelDialer as _, process_runtime::CoreProcessRuntime,
    socket::SocketListener, tunnel::Tunnel,
};
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket};
use tokio::sync::{Mutex, RwLock, RwLockReadGuard};
use uuid::Uuid;

use tauri::{AppHandle, Emitter, Manager as _};

#[cfg(not(target_os = "android"))]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

static INSTANCE_MANAGER: once_cell::sync::Lazy<RwLock<Option<Arc<NativeInstanceManager>>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(None));

static RPC_RING_UUID: once_cell::sync::Lazy<uuid::Uuid> =
    once_cell::sync::Lazy::new(uuid::Uuid::new_v4);

static CLIENT_MANAGER: once_cell::sync::Lazy<RwLock<Option<manager::GUIClientManager>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(None));

type BoxedTunnelListener = Box<dyn SocketListener<Accepted = Box<dyn Tunnel>>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RpcServerKind {
    Ring,
    Tcp,
}

struct RpcServer {
    kind: RpcServerKind,
    _server: ApiRpcServer<BoxedTunnelListener>,
    bind_url: Option<url::Url>,
}
static RPC_SERVER: once_cell::sync::Lazy<Mutex<Option<RpcServer>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

static WEB_CLIENT: once_cell::sync::Lazy<RwLock<Option<WebClient>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(None));

macro_rules! get_client_manager {
    () => {{
        let guard = CLIENT_MANAGER
            .try_read()
            .map_err(|_| "Failed to acquire read lock for client manager")?;
        RwLockReadGuard::try_map(guard, |cm| cm.as_ref())
            .map_err(|_| "RPC connection not initialized".to_string())
    }};
}

#[tauri::command]
fn easytier_version() -> Result<String, String> {
    Ok(easytier::VERSION.to_string())
}

#[tauri::command]
fn set_dock_visibility(app: tauri::AppHandle, visible: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use tauri::ActivationPolicy;
        app.set_activation_policy(if visible {
            ActivationPolicy::Regular
        } else {
            ActivationPolicy::Accessory
        })
        .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
    Ok(())
}

#[tauri::command]
fn parse_network_config(cfg: NetworkConfig) -> Result<String, String> {
    let toml = cfg.gen_config().map_err(|e| e.to_string())?;
    Ok(toml.dump())
}

#[tauri::command]
fn generate_network_config(toml_config: String) -> Result<NetworkConfig, String> {
    let config = TomlConfigLoader::new_from_str(&toml_config).map_err(|e| e.to_string())?;
    let cfg = NetworkConfig::new_from_config(&config).map_err(|e| e.to_string())?;
    Ok(cfg)
}

#[tauri::command]
async fn run_network_instance(
    app: AppHandle,
    cfg: NetworkConfig,
    save: bool,
) -> Result<(), String> {
    let client_manager = get_client_manager!()?;
    let toml_config = cfg.gen_config().map_err(|e| e.to_string())?;
    let _mutation = client_manager.config_mutation.lock().await;
    client_manager
        .pre_run_network_instance_hook(&app, &toml_config, manager::PersistedConfigSource::User)
        .await?;
    client_manager
        .handle_run_network_instance(app.clone(), cfg, save)
        .await
        .map_err(|e| e.to_string())?;
    client_manager
        .post_run_network_instance_hook(&app, &toml_config.get_id())
        .await?;
    Ok(())
}

#[tauri::command]
async fn collect_network_info(
    app: AppHandle,
    instance_id: String,
) -> Result<CollectNetworkInfoResponse, String> {
    let instance_id = instance_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    get_client_manager!()?
        .handle_collect_network_info(app, Some(vec![instance_id]))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_vpn_portal_info(instance_id: String) -> Result<Option<VpnPortalInfo>, String> {
    let instance_id = instance_id
        .parse::<uuid::Uuid>()
        .map_err(|e| e.to_string())?;
    let client_manager = get_client_manager!()?;
    let client = client_manager
        .rpc_manager
        .rpc_client()
        .scoped_client::<VpnPortalRpcClientFactory<BaseController>>(1, 1, "".to_string());
    let response = client
        .get_vpn_portal_info(
            BaseController::default(),
            GetVpnPortalInfoRequest {
                instance: Some(InstanceIdentifier {
                    selector: Some(instance_identifier::Selector::Id(instance_id.into())),
                }),
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(response.vpn_portal_info)
}

#[tauri::command]
async fn patch_vpn_portal_clients(
    app: AppHandle,
    instance_id: String,
    action: String,
    name: Option<String>,
    virtual_ip: Option<String>,
    groups: Option<Vec<String>>,
) -> Result<(), String> {
    let instance_id = instance_id
        .parse::<uuid::Uuid>()
        .map_err(|e| e.to_string())?;
    let action = match action.as_str() {
        "add" => ConfigPatchAction::Add,
        "remove" => ConfigPatchAction::Remove,
        "clear" => ConfigPatchAction::Clear,
        other => return Err(format!("invalid vpn portal client patch action: {other}")),
    };
    let client = if action == ConfigPatchAction::Clear {
        None
    } else {
        Some(VpnPortalClientConfig {
            name: name.unwrap_or_default(),
            virtual_ip: virtual_ip.unwrap_or_default(),
            groups: groups.unwrap_or_default(),
        })
    };

    let client_manager = get_client_manager!()?;
    let _mutation = client_manager.config_mutation.lock().await;
    client_manager
        .handle_patch_vpn_portal_clients(
            app,
            instance_id,
            vec![VpnPortalClientPatch {
                action: action as i32,
                client,
            }],
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_logging_level(level: String) -> Result<(), String> {
    get_client_manager!()?
        .set_logging_level(level.clone())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn set_tun_fd(fd: i32) -> Result<(), String> {
    let Some(instance_manager) = INSTANCE_MANAGER.read().await.clone() else {
        return Err("set_tun_fd is not supported in remote mode".to_string());
    };
    if let Some(uuid) = get_client_manager!()?
        .get_enabled_instances_with_tun_ids()
        .next()
    {
        instance_manager
            .attach_tun_fd(uuid, fd)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn list_network_instance_ids(
    app: AppHandle,
) -> Result<ListNetworkInstanceIdsJsonResp, String> {
    get_client_manager!()?
        .handle_list_network_instance_ids(app)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_network_instance(app: AppHandle, instance_id: String) -> Result<(), String> {
    let instance_id = instance_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let client_manager = get_client_manager!()?;
    let _mutation = client_manager.config_mutation.lock().await;
    client_manager
        .handle_remove_network_instances(app.clone(), vec![instance_id])
        .await
        .map_err(|e| e.to_string())?;
    client_manager
        .post_stop_network_instances_hook(&app)
        .await?;

    Ok(())
}

#[tauri::command]
async fn update_network_config_state(
    app: AppHandle,
    instance_id: String,
    disabled: bool,
) -> Result<(), String> {
    let instance_id = instance_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let client_manager = get_client_manager!()?;
    let _mutation = client_manager.config_mutation.lock().await;
    if !disabled {
        let (cfg, source) = client_manager
            .handle_get_network_config_with_source(app.clone(), instance_id)
            .await
            .map_err(|e| e.to_string())?;
        let toml_config = cfg.gen_config().map_err(|e| e.to_string())?;
        client_manager
            .pre_run_network_instance_hook(
                &app,
                &toml_config,
                manager::PersistedConfigSource::from_runtime_source(source),
            )
            .await?;
    }
    client_manager
        .handle_update_network_state(app.clone(), instance_id, disabled)
        .await
        .map_err(|e| e.to_string())?;

    if disabled {
        client_manager
            .post_stop_network_instances_hook(&app)
            .await?;
    } else {
        client_manager
            .post_run_network_instance_hook(&app, &instance_id)
            .await?;
    }

    Ok(())
}

#[tauri::command]
async fn save_network_config(app: AppHandle, cfg: NetworkConfig) -> Result<(), String> {
    let instance_id = cfg
        .instance_id()
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let client_manager = get_client_manager!()?;
    let _mutation = client_manager.config_mutation.lock().await;
    client_manager
        .handle_save_network_config(app, instance_id, cfg)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn validate_config(
    app: AppHandle,
    config: NetworkConfig,
) -> Result<ValidateConfigResponse, String> {
    get_client_manager!()?
        .handle_validate_config(app, config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config(app: AppHandle, instance_id: String) -> Result<NetworkConfig, String> {
    let instance_id = instance_id
        .parse()
        .map_err(|e: uuid::Error| e.to_string())?;
    let cfg = get_client_manager!()?
        .handle_get_network_config(app, instance_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(cfg)
}

#[tauri::command]
async fn load_configs(
    app: AppHandle,
    configs: Vec<manager::StoredGuiConfig>,
    enabled_networks: Vec<String>,
) -> Result<(), String> {
    let client_manager = get_client_manager!()?;
    let _mutation = client_manager.config_mutation.lock().await;
    client_manager
        .load_configs(app, configs, enabled_networks)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_network_metas(
    app: AppHandle,
    instance_ids: Vec<uuid::Uuid>,
) -> Result<GetNetworkMetasResponse, String> {
    get_client_manager!()?
        .handle_get_network_metas(app, instance_ids)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "android")]
#[tauri::command]
fn init_service() -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn init_service(opts: Option<service::ServiceOptions>) -> Result<(), String> {
    match opts {
        Some(args) => {
            let path = std::path::Path::new(&args.config_dir);
            if !path.exists() {
                std::fs::create_dir_all(&args.config_dir).map_err(|e| e.to_string())?;
            } else if !path.is_dir() {
                return Err("config_dir exists but is not a directory".to_string());
            }
            let path = std::path::Path::new(&args.file_log_dir);
            if !path.exists() {
                std::fs::create_dir_all(&args.file_log_dir).map_err(|e| e.to_string())?;
            } else if !path.is_dir() {
                return Err("file_log_dir exists but is not a directory".to_string());
            }

            service::install(args).map_err(|e| format!("{:#}", e))?;
        }
        None => {
            service::uninstall().map_err(|e| format!("{:#}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
fn set_service_status(_enable: bool) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    {
        service::set_status(_enable).map_err(|e| format!("{:#}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn get_service_status() -> Result<&'static str, String> {
    #[cfg(not(target_os = "android"))]
    {
        use easytier::service_manager::ServiceStatus;
        let status = service::status().map_err(|e| format!("{:#}", e))?;
        match status {
            ServiceStatus::NotInstalled => Ok("NotInstalled"),
            ServiceStatus::Stopped(_) => Ok("Stopped"),
            ServiceStatus::Running => Ok("Running"),
        }
    }
    #[cfg(target_os = "android")]
    {
        Ok("NotInstalled")
    }
}

fn normalize_normal_mode_rpc_portal(portal: &str) -> Result<(url::Url, url::Url), String> {
    let portal_url: url::Url = portal
        .parse()
        .map_err(|e| format!("invalid rpc portal: {:#}", e))?;
    let bind_url = portal_url.clone();
    let mut connect_url = portal_url.clone();
    // if bind addr is 0.0.0.0, should convert to 127.0.0.1
    if connect_url.host_str() == Some("0.0.0.0") {
        connect_url.set_host(Some("127.0.0.1")).unwrap();
    }
    Ok((bind_url, connect_url))
}

async fn resolve_rpc_bind_url(url: &url::Url) -> Result<std::net::SocketAddr, String> {
    if url.scheme() != "tcp" {
        return Err(format!("RPC portal requires tcp URL: {url}"));
    }
    let host = url
        .host_str()
        .ok_or_else(|| format!("RPC portal has no host: {url}"))?;
    let port = url.port().unwrap_or(11010);
    tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| format!("failed to resolve RPC portal {url}: {error}"))?
        .next()
        .ok_or_else(|| format!("RPC portal has no resolved address: {url}"))
}

#[tauri::command]
async fn init_rpc_connection(
    _app: AppHandle,
    is_normal_mode: bool,
    url: Option<String>,
) -> Result<(), String> {
    let mut client_manager_guard =
        tokio::time::timeout(std::time::Duration::from_secs(5), CLIENT_MANAGER.write())
            .await
            .map_err(|_| "Failed to acquire write lock for client manager")?;
    let mut instance_manager_guard = INSTANCE_MANAGER
        .try_write()
        .map_err(|_| "Failed to acquire write lock for instance manager")?;
    let mut rpc_server_guard = RPC_SERVER
        .try_lock()
        .map_err(|_| "Failed to acquire lock for rpc server")?;

    let mut client_url = url.clone();
    let mut local_process_runtime = None;
    if is_normal_mode {
        let instance_manager = if let Some(im) = instance_manager_guard.take() {
            im
        } else {
            Arc::new(native_instance_manager())
        };

        let portal = url.and_then(|s| {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        let (desired_kind, bind_url, connect_url) = if let Some(portal) = portal {
            let (bind_url, connect_url) = normalize_normal_mode_rpc_portal(&portal)?;
            (RpcServerKind::Tcp, Some(bind_url), Some(connect_url))
        } else {
            (RpcServerKind::Ring, None, None)
        };

        let need_restart = rpc_server_guard
            .as_ref()
            .map(|x| x.kind != desired_kind || x.bind_url != bind_url)
            .unwrap_or(true);

        if need_restart {
            *rpc_server_guard = None;

            let tunnel: BoxedTunnelListener = match desired_kind {
                RpcServerKind::Ring => instance_manager
                    .process_runtime()
                    .bind_ring_tunnel(*RPC_RING_UUID.deref())
                    .map_err(|error| error.to_string())?,
                RpcServerKind::Tcp => {
                    let bind_url = bind_url.as_ref().expect("tcp rpc must have bind url");
                    Box::new(runtime_rpc_listener(resolve_rpc_bind_url(bind_url).await?))
                }
            };

            let rpc_server = ApiRpcServer::from_tunnel(tunnel, instance_manager.clone())
                .with_rx_timeout(None)
                .serve()
                .await
                .map_err(|e| e.to_string())?;
            *rpc_server_guard = Some(RpcServer {
                kind: desired_kind,
                _server: rpc_server,
                bind_url,
            });
        }

        local_process_runtime = Some(instance_manager.process_runtime());
        *instance_manager_guard = Some(instance_manager);
        client_url = connect_url.map(|u| u.to_string());
    } else {
        *rpc_server_guard = None;
    }

    let client_manager = tokio::time::timeout(
        std::time::Duration::from_millis(1000),
        manager::GUIClientManager::new(client_url, local_process_runtime),
    )
    .await
    .map_err(|_| "connect remote rpc timed out".to_string())?
    .with_context(|| "Failed to connect remote rpc")
    .map_err(|e| format!("{:#}", e))?;
    *client_manager_guard = Some(client_manager);

    if !is_normal_mode {
        drop(WEB_CLIENT.write().await.take());
        if let Some(instance_manager) = instance_manager_guard.take() {
            instance_manager
                .retain_network_instances(&[])
                .await
                .map_err(|e| e.to_string())?;
            drop(instance_manager);
        }
    }

    Ok(())
}

#[tauri::command]
async fn is_client_running() -> Result<bool, String> {
    Ok(get_client_manager!()?.rpc_manager.is_running())
}

#[tauri::command]
async fn init_web_client(app: AppHandle, url: Option<String>) -> Result<(), String> {
    let mut web_client_guard = WEB_CLIENT.write().await;
    let Some(url) = url else {
        *web_client_guard = None;
        return Ok(());
    };
    let instance_manager = INSTANCE_MANAGER
        .try_read()
        .map_err(|_| "Failed to acquire read lock for instance manager")?
        .clone()
        .ok_or_else(|| "Instance manager is not available".to_string())?;

    let hooks = Arc::new(manager::GuiHooks { app: app.clone() });
    let machine_id_state_dir = app
        .path()
        .app_data_dir()
        .with_context(|| "Failed to resolve machine id state directory")
        .map_err(|e| format!("{:#}", e))?;

    let web_client = web_client::run_web_client(
        url.as_str(),
        easytier::common::MachineIdOptions {
            explicit_machine_id: None,
            state_dir: Some(machine_id_state_dir),
        },
        None,
        false,
        instance_manager,
        Some(hooks),
    )
    .await
    .with_context(|| "Failed to initialize web client")
    .map_err(|e| format!("{:#}", e))?;
    *web_client_guard = Some(web_client);
    Ok(())
}

#[tauri::command]
async fn is_web_client_connected() -> Result<bool, String> {
    let web_client_guard = WEB_CLIENT.read().await;
    if let Some(web_client) = web_client_guard.as_ref() {
        Ok(web_client.is_connected())
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn http_get(
    url: String,
    proxy: Option<String>,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs.unwrap_or(3)));
    if let Some(ref p) = proxy {
        builder =
            builder.proxy(reqwest::Proxy::all(p).map_err(|e| format!("Invalid proxy: {}", e))?);
    }
    let client = builder
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;
    client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))
}

#[tauri::command]
async fn http_post(
    url: String,
    proxy: Option<String>,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs.unwrap_or(3)));
    if let Some(ref p) = proxy {
        builder =
            builder.proxy(reqwest::Proxy::all(p).map_err(|e| format!("Invalid proxy: {}", e))?);
    }
    let client = builder
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;
    client
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))
}

#[derive(serde::Serialize)]
struct HttpRequestResponse {
    status: u16,
    headers: std::collections::HashMap<String, String>,
    body: String,
}

#[tauri::command]
async fn http_request(
    method: String,
    url: String,
    proxy: Option<String>,
    timeout_secs: Option<u64>,
    headers: Option<std::collections::HashMap<String, String>>,
    body: Option<String>,
) -> Result<HttpRequestResponse, String> {
    let method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|e| format!("Invalid HTTP method: {}", e))?;
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs.unwrap_or(10)));
    if let Some(ref p) = proxy {
        builder =
            builder.proxy(reqwest::Proxy::all(p).map_err(|e| format!("Invalid proxy: {}", e))?);
    }
    let client = builder
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;

    let mut request = client.request(method, &url);
    if let Some(headers) = headers {
        for (name, value) in headers {
            let name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| format!("Invalid header name: {}", e))?;
            let value = reqwest::header::HeaderValue::from_str(&value)
                .map_err(|e| format!("Invalid header value for {}: {}", name, e))?;
            request = request.header(name, value);
        }
    }
    if let Some(body) = body {
        request = request.body(body);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    let status = response.status().as_u16();
    let response_headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                value.to_str().unwrap_or_default().to_string(),
            )
        })
        .collect();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    Ok(HttpRequestResponse {
        status,
        headers: response_headers,
        body,
    })
}

#[tauri::command]
async fn tcp_ping(host: String, port: u16) -> Result<String, String> {
    let addr = format!("{}:{}", host, port);
    let start = std::time::Instant::now();
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::net::TcpStream::connect(&addr),
    )
    .await
    {
        Ok(Ok(_stream)) => {
            let ms = start.elapsed().as_millis();
            Ok(format!("OK {}ms", ms))
        }
        Ok(Err(e)) => Err(format!("TCP connect failed: {}", e)),
        Err(_) => Err("TCP connect timeout (5s)".to_string()),
    }
}

// ============================================================
// LuCI HTTP Reverse Proxy (local iframe → router)
// ============================================================

type ProxyBodyError = Box<dyn std::error::Error + Send + Sync>;
type ProxyBody = http_body_util::combinators::UnsyncBoxBody<Bytes, ProxyBodyError>;

const TRANSFER_PROGRESS_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);
const TRANSFER_PROGRESS_BYTES: u64 = 1024 * 1024;
const TRANSFER_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

struct ProxyHandle {
    #[allow(dead_code)]
    port: u16,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    state: std::sync::Arc<ProxyState>,
}

struct SharedState {
    last_path: std::sync::Mutex<String>,
    #[allow(dead_code)]
    last_path_type: std::sync::Mutex<String>, // "tunnel" or "lan"
}

struct ProxyState {
    routers: std::sync::RwLock<HashMap<String, std::sync::Arc<ProxyCtx>>>,
    active_router_id: std::sync::Mutex<String>,
}

struct ProxyCtx {
    app: AppHandle,
    client: reqwest::Client,
    cookie: Mutex<String>,
    router_ip: String,
    username: String,
    password: String,
    shared: std::sync::Arc<SharedState>,
    download_cache: std::sync::Mutex<HashMap<String, CachedDownload>>,
    last_download: std::sync::Mutex<Option<CachedDownload>>,
    download_dir: PathBuf,
}

#[derive(Clone)]
struct CachedDownload {
    path: PathBuf,
    content_type: Option<String>,
    content_disposition: Option<String>,
    content_length: u64,
}

impl ProxyState {
    fn router(&self, router_id: &str) -> Option<std::sync::Arc<ProxyCtx>> {
        self.routers
            .read()
            .unwrap()
            .get(router_id)
            .map(std::sync::Arc::clone)
    }

    fn active_router_id(&self) -> String {
        self.active_router_id.lock().unwrap().clone()
    }
}

fn internal_cookie_value(headers: &reqwest::header::HeaderMap, name: &str) -> Option<String> {
    header_value(headers, "cookie")?
        .split(';')
        .filter_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name).then(|| value.trim().to_string())
        })
        .next()
}

fn request_query_value(path: &str, name: &str) -> Option<String> {
    let request_url = url::Url::parse("http://127.0.0.1").ok()?.join(path).ok()?;
    request_url
        .query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.to_string())
}

fn request_router_id(
    request_path: &str,
    headers: &reqwest::header::HeaderMap,
    state: &ProxyState,
) -> String {
    if let Some(router_id) = request_query_value(request_path, "__et_router") {
        return router_id;
    }
    if let Some(router_id) = internal_cookie_value(headers, "__et_router") {
        return router_id;
    }
    if let Some(referer) = header_value(headers, "referer")
        && let Ok(referer) = url::Url::parse(&referer)
        && let Some(router_id) = referer
            .query_pairs()
            .find(|(key, _)| key == "__et_router")
            .map(|(_, value)| value.to_string())
    {
        return router_id;
    }
    state.active_router_id()
}

fn upstream_browser_cookie(headers: &reqwest::header::HeaderMap) -> Option<String> {
    let cookies: Vec<_> = header_value(headers, "cookie")?
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .filter(|part| !part.starts_with("__et_router="))
        .map(str::to_owned)
        .collect();
    (!cookies.is_empty()).then(|| cookies.join("; "))
}

#[derive(Clone, serde::Serialize)]
struct RouterTransferEvent {
    event: String,
    direction: String,
    path: String,
    filename: Option<String>,
    content_type: Option<String>,
    content_length: Option<u64>,
    transferred: u64,
    duration_ms: u64,
    status: Option<u16>,
    error: Option<String>,
}

#[allow(clippy::too_many_arguments)]
fn router_transfer_event(
    event: &str,
    direction: &str,
    path: &str,
    filename: Option<String>,
    content_type: Option<String>,
    content_length: Option<u64>,
    transferred: u64,
    duration_ms: u64,
    status: Option<u16>,
    error: Option<String>,
) -> RouterTransferEvent {
    RouterTransferEvent {
        event: event.to_string(),
        direction: direction.to_string(),
        path: path.to_string(),
        filename,
        content_type,
        content_length,
        transferred,
        duration_ms,
        status,
        error,
    }
}

fn emit_router_transfer(app: &AppHandle, event: RouterTransferEvent) {
    let _ = app.emit("router-transfer", event);
}

fn header_value(headers: &reqwest::header::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn content_length(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    header_value(headers, "content-length").and_then(|value| value.parse::<u64>().ok())
}

fn content_disposition_filename(value: &str) -> Option<String> {
    for part in value.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("filename*=") {
            let value = rest.trim().trim_matches('"');
            if let Some((_, filename)) = value.split_once("''") {
                return Some(filename.trim_matches('"').to_string());
            }
        }
        if let Some(rest) = part.strip_prefix("filename=") {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn strip_internal_router_query(path: &str) -> String {
    let Some((base, query)) = path.split_once('?') else {
        return path.to_string();
    };
    let mut kept = Vec::new();
    for part in query.split('&') {
        if !part.starts_with("__et_router=") && !part.starts_with("__et_nonce=") {
            kept.push(part);
        }
    }
    if kept.is_empty() {
        base.to_string()
    } else {
        format!("{base}?{}", kept.join("&"))
    }
}

fn download_cache_key(target_url: &str) -> Option<String> {
    let target = url::Url::parse(target_url).ok()?;
    let mut path = target.path().to_string();
    if let Some(query) = target.query() {
        path.push('?');
        path.push_str(query);
    }
    Some(strip_internal_router_query(&path))
}

fn cached_download_key_from_request(path: &str) -> Option<String> {
    let request_url = url::Url::parse("http://127.0.0.1").ok()?.join(path).ok()?;
    if request_url.path() != "/__easytier_download__" {
        return None;
    }
    let target = request_url
        .query_pairs()
        .find(|(key, _)| key == "url")?
        .1
        .to_string();
    download_cache_key(&target)
}

async fn file_proxy_body(path: PathBuf) -> Result<ProxyBody, String> {
    use futures_util::TryStreamExt as _;

    let file = tokio::fs::File::open(path)
        .await
        .map_err(|e| format!("Failed to open cached download: {e}"))?;
    let stream = futures_util::stream::unfold(file, |mut file| async move {
        let mut buffer = vec![0_u8; 64 * 1024];
        match file.read(&mut buffer).await {
            Ok(0) => None,
            Ok(size) => {
                buffer.truncate(size);
                Some((Ok(Bytes::from(buffer)), file))
            }
            Err(error) => Some((Err(error), file)),
        }
    });
    Ok(StreamBody::new(
        stream
            .map_ok(Frame::data)
            .map_err(|error| -> ProxyBodyError { error.into() }),
    )
    .boxed_unsync())
}

async fn cached_download_response(cached: &CachedDownload) -> Result<Response<ProxyBody>, String> {
    let body = file_proxy_body(cached.path.clone()).await?;
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("content-length", cached.content_length.to_string())
        .header("cross-origin-resource-policy", "cross-origin");
    if let Some(content_type) = &cached.content_type {
        builder = builder.header("content-type", content_type);
    }
    if let Some(content_disposition) = &cached.content_disposition {
        builder = builder.header("content-disposition", content_disposition);
    }
    Ok(builder.body(body).unwrap())
}

#[allow(clippy::too_many_arguments)]
async fn cache_download_response(
    resp: reqwest::Response,
    ctx: &std::sync::Arc<ProxyCtx>,
    path: &str,
    filename: Option<String>,
    content_type: Option<String>,
    content_disposition: Option<String>,
    content_length: Option<u64>,
    status: u16,
) -> Result<Response<ProxyBody>, String> {
    use futures_util::StreamExt as _;

    let cache_key = download_cache_key(&format!("http://{}{}", ctx.router_ip, path))
        .unwrap_or_else(|| path.to_string());
    let cache_path = ctx
        .download_dir
        .join(format!("{}.download", Uuid::new_v4().simple()));
    let mut output = tokio::fs::File::create(&cache_path)
        .await
        .map_err(|e| format!("Failed to create download cache: {e}"))?;
    let mut stream = Box::pin(tracked_transfer_stream(
        resp.bytes_stream(),
        ctx.app.clone(),
        "download",
        path.to_string(),
        filename,
        content_type.clone(),
        content_length,
        Some(status),
        true,
    ));
    let mut total = 0_u64;
    loop {
        match stream.next().await {
            Some(Ok(chunk)) => {
                output
                    .write_all(&chunk)
                    .await
                    .map_err(|e| format!("Failed to write download cache: {e}"))?;
                total += chunk.len() as u64;
            }
            None => break,
            Some(Err(error)) => {
                drop(output);
                let _ = tokio::fs::remove_file(&cache_path).await;
                return Err(error.to_string());
            }
        }
    }
    output
        .flush()
        .await
        .map_err(|e| format!("Failed to flush download cache: {e}"))?;

    let cached = CachedDownload {
        path: cache_path,
        content_type,
        content_disposition,
        content_length: total,
    };
    let previous = ctx
        .download_cache
        .lock()
        .unwrap()
        .insert(cache_key, cached.clone());
    *ctx.last_download.lock().unwrap() = Some(cached.clone());
    if let Some(previous) = previous {
        tokio::spawn(async move {
            let _ = tokio::fs::remove_file(previous.path).await;
        });
    }
    cached_download_response(&cached).await
}

fn rewrite_location(value: &str, router_ip: &str) -> String {
    let Ok(target) = url::Url::parse(value) else {
        return value.to_string();
    };
    if !matches!(target.scheme(), "http" | "https") || target.host_str() != Some(router_ip) {
        return value.to_string();
    }
    let mut path = target.path().to_string();
    if let Some(query) = target.query() {
        path.push('?');
        path.push_str(query);
    }
    path
}

fn is_download_response(resp: &reqwest::Response) -> bool {
    if header_value(resp.headers(), "content-disposition")
        .map(|value| value.to_ascii_lowercase().contains("attachment"))
        .unwrap_or(false)
    {
        return true;
    }

    let content_type = header_value(resp.headers(), "content-type")
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(
        content_type.split(';').next().unwrap_or_default().trim(),
        "application/octet-stream"
            | "application/zip"
            | "application/gzip"
            | "application/x-gzip"
            | "application/x-tar"
            | "application/x-7z-compressed"
            | "application/vnd.android.package-archive"
    )
}

fn is_upload_request(
    method: &reqwest::Method,
    content_type: Option<&str>,
    length: Option<u64>,
) -> bool {
    if method == reqwest::Method::GET || method == reqwest::Method::HEAD {
        return false;
    }
    content_type
        .map(|value| value.to_ascii_lowercase().contains("multipart/form-data"))
        .unwrap_or(false)
        || length.unwrap_or(0) >= 1024 * 1024
}

fn full_proxy_body(data: Bytes) -> ProxyBody {
    Full::new(data)
        .map_err(|never| -> ProxyBodyError { match never {} })
        .boxed_unsync()
}

#[allow(clippy::too_many_arguments)]
fn tracked_transfer_stream<S, E>(
    stream: S,
    app: AppHandle,
    direction: &'static str,
    path: String,
    filename: Option<String>,
    content_type: Option<String>,
    content_length: Option<u64>,
    status: Option<u16>,
    report: bool,
) -> impl futures_util::Stream<Item = Result<Bytes, ProxyBodyError>> + Send + 'static
where
    S: futures_util::Stream<Item = Result<Bytes, E>> + Send + 'static,
    E: Into<ProxyBodyError> + Send + Sync + 'static,
{
    use futures_util::StreamExt as _;

    let started_at = std::time::Instant::now();
    if report {
        emit_router_transfer(
            &app,
            router_transfer_event(
                "started",
                direction,
                &path,
                filename.clone(),
                content_type.clone(),
                content_length,
                0,
                0,
                status,
                None,
            ),
        );
    }

    let stream = Box::pin(stream);
    futures_util::stream::unfold(
        (
            stream,
            0_u64,
            started_at,
            std::time::Instant::now(),
            0_u64,
            false,
        ),
        move |(mut stream, mut transferred, started_at, mut last_emit, mut last_reported, done)| {
            let app = app.clone();
            let path = path.clone();
            let filename = filename.clone();
            let content_type = content_type.clone();
            async move {
                if done {
                    return None;
                }

                match tokio::time::timeout(TRANSFER_IDLE_TIMEOUT, stream.next()).await {
                    Err(_) => {
                        let error = "Transfer timed out while waiting for more data".to_string();
                        if report {
                            emit_router_transfer(
                                &app,
                                router_transfer_event(
                                    "failed",
                                    direction,
                                    &path,
                                    filename,
                                    content_type,
                                    content_length,
                                    transferred,
                                    started_at.elapsed().as_millis() as u64,
                                    status,
                                    Some(error),
                                ),
                            );
                        }
                        Some((
                            Err("Transfer timed out while waiting for more data".into()),
                            (
                                stream,
                                transferred,
                                started_at,
                                last_emit,
                                last_reported,
                                true,
                            ),
                        ))
                    }
                    Ok(Some(Ok(bytes))) => {
                        transferred += bytes.len() as u64;
                        if report
                            && (last_emit.elapsed() >= TRANSFER_PROGRESS_INTERVAL
                                || transferred.saturating_sub(last_reported)
                                    >= TRANSFER_PROGRESS_BYTES)
                        {
                            emit_router_transfer(
                                &app,
                                router_transfer_event(
                                    "progress",
                                    direction,
                                    &path,
                                    filename,
                                    content_type,
                                    content_length,
                                    transferred,
                                    started_at.elapsed().as_millis() as u64,
                                    status,
                                    None,
                                ),
                            );
                            last_emit = std::time::Instant::now();
                            last_reported = transferred;
                        }
                        Some((
                            Ok(bytes),
                            (
                                stream,
                                transferred,
                                started_at,
                                last_emit,
                                last_reported,
                                false,
                            ),
                        ))
                    }
                    Ok(Some(Err(error))) => {
                        let error: ProxyBodyError = error.into();
                        if report {
                            emit_router_transfer(
                                &app,
                                router_transfer_event(
                                    "failed",
                                    direction,
                                    &path,
                                    filename,
                                    content_type,
                                    content_length,
                                    transferred,
                                    started_at.elapsed().as_millis() as u64,
                                    status,
                                    Some(error.to_string()),
                                ),
                            );
                        }
                        Some((
                            Err(error),
                            (
                                stream,
                                transferred,
                                started_at,
                                last_emit,
                                last_reported,
                                true,
                            ),
                        ))
                    }
                    Ok(None) => {
                        if report {
                            emit_router_transfer(
                                &app,
                                router_transfer_event(
                                    "finished",
                                    direction,
                                    &path,
                                    filename,
                                    content_type,
                                    content_length,
                                    transferred,
                                    started_at.elapsed().as_millis() as u64,
                                    status,
                                    None,
                                ),
                            );
                        }
                        None
                    }
                }
            }
        },
    )
}

static PROXY: once_cell::sync::Lazy<std::sync::Mutex<Option<ProxyHandle>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

fn build_proxy_client(socks5_proxy: &Option<String>) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .tcp_keepalive(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none());
    if let Some(p) = socks5_proxy {
        builder =
            builder.proxy(reqwest::Proxy::all(p).map_err(|e| format!("Invalid proxy: {}", e))?);
    }
    builder
        .build()
        .map_err(|e| format!("Failed to create proxy client: {}", e))
}

async fn luci_login(
    client: &reqwest::Client,
    router_ip: &str,
    username: &str,
    password: &str,
) -> Result<String, String> {
    let login_url = format!("http://{}/cgi-bin/luci/", router_ip);
    let resp = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        client
            .post(&login_url)
            .form(&[("luci_username", username), ("luci_password", password)])
            .send(),
    )
    .await
    .map_err(|_| "Login request timed out".to_string())?
    .map_err(|e| format!("Login request failed: {}", e))?;
    let status = resp.status().as_u16();
    let mut found_cookie = String::new();
    for cookie in resp.headers().get_all("set-cookie") {
        let s = cookie.to_str().unwrap_or("");
        if s.starts_with("sysauth=") || s.starts_with("sysauth_http=") {
            found_cookie = s.split(';').next().unwrap_or("").to_string();
            break;
        }
    }
    if !found_cookie.is_empty() {
        return Ok(found_cookie);
    }
    // Collect first 3 cookies for debug
    let debug_cookies: Vec<_> = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|c| c.to_str().ok())
        .take(3)
        .collect();
    Err(format!(
        "No sysauth cookie in login response (HTTP {status}, cookies: {:?})",
        debug_cookies
    ))
}

fn is_html(resp: &reqwest::Response) -> bool {
    resp.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/html"))
        .unwrap_or(false)
}

async fn proxy_request(
    req: Request<Incoming>,
    state: &std::sync::Arc<ProxyState>,
) -> Result<Response<ProxyBody>, hyper::Error> {
    let request_path = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string();
    let path = strip_internal_router_query(&request_path);
    let router_id = request_router_id(&request_path, req.headers(), state);
    let Some(ctx) = state.router(&router_id) else {
        return build_error_response("Router session is not available");
    };
    let target_url = format!("http://{}{}", ctx.router_ip, path);
    if request_path.starts_with("/__easytier_download__") {
        let cached = cached_download_key_from_request(&request_path)
            .and_then(|cache_key| ctx.download_cache.lock().unwrap().get(&cache_key).cloned())
            .or_else(|| ctx.last_download.lock().unwrap().clone());
        return match cached {
            Some(cached) => cached_download_response(&cached)
                .await
                .or_else(|error| build_error_response(&error)),
            None => build_error_response("Download is no longer available"),
        };
    }
    let method = req.method().clone();
    let request_content_type = header_value(req.headers(), "content-type");
    let request_length = content_length(req.headers());
    let report_upload = is_upload_request(&method, request_content_type.as_deref(), request_length);

    let cookie = ctx.cookie.lock().await.clone();
    let browser_cookie = upstream_browser_cookie(req.headers());
    let cookie_header = match browser_cookie {
        Some(browser_cookie) if !cookie.is_empty() => format!("{cookie}; {browser_cookie}"),
        Some(browser_cookie) => browser_cookie,
        None => cookie,
    };
    let mut proxy_req = ctx
        .client
        .request(method.clone(), &target_url)
        .header("Cookie", &cookie_header);

    for (name, value) in req.headers() {
        let n = name.as_str().to_lowercase();
        if n != "host"
            && n != "cookie"
            && n != "origin"
            && n != "referer"
            && n != "transfer-encoding"
            && let Ok(v) = value.to_str()
        {
            proxy_req = proxy_req.header(name.as_str(), v);
        }
    }

    let (_parts, body) = req.into_parts();
    let has_body = method != reqwest::Method::GET && method != reqwest::Method::HEAD;
    if has_body {
        let body_stream = tracked_transfer_stream(
            body.into_data_stream(),
            ctx.app.clone(),
            "upload",
            path.clone(),
            None,
            request_content_type.clone(),
            request_length,
            None,
            report_upload,
        );
        proxy_req = proxy_req.body(reqwest::Body::wrap_stream(body_stream));
    }

    let resp =
        match tokio::time::timeout(std::time::Duration::from_secs(30), proxy_req.send()).await {
            Err(_) => {
                if report_upload {
                    emit_router_transfer(
                        &ctx.app,
                        router_transfer_event(
                            "failed",
                            "upload",
                            &path,
                            None,
                            request_content_type.clone(),
                            request_length,
                            0,
                            0,
                            None,
                            Some("Router request timed out".to_string()),
                        ),
                    );
                }
                return build_error_response("Router request timed out");
            }
            Ok(Err(e)) => {
                if report_upload {
                    emit_router_transfer(
                        &ctx.app,
                        router_transfer_event(
                            "failed",
                            "upload",
                            &path,
                            None,
                            request_content_type.clone(),
                            request_length,
                            0,
                            0,
                            None,
                            Some(e.to_string()),
                        ),
                    );
                }
                return build_error_response(&e.to_string());
            }
            Ok(Ok(resp)) => resp,
        };

    // Only track page navigations (HTML), not resource loads (CSS/JS/images)
    if is_html(&resp) {
        *ctx.shared.last_path.lock().unwrap() = path.to_string();
    }

    // If session expired (redirect to login), re-login and retry once
    let is_login_redirect = resp.status() == reqwest::StatusCode::FOUND
        && resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|loc| loc.contains("login"))
            .unwrap_or(false);

    if is_login_redirect {
        drop(resp);
        match luci_login(&ctx.client, &ctx.router_ip, &ctx.username, &ctx.password).await {
            Ok(new_cookie) => {
                *ctx.cookie.lock().await = new_cookie.clone();
                if has_body {
                    return build_error_response("Session expired during upload, please retry");
                }
                let retry_req = ctx
                    .client
                    .request(method.clone(), &target_url)
                    .header("Cookie", &new_cookie);
                let retry_resp = match retry_req.send().await {
                    Ok(r) => r,
                    Err(e) => return build_error_response(&e.to_string()),
                };
                if is_html(&retry_resp) {
                    *ctx.shared.last_path.lock().unwrap() = path.to_string();
                }
                return build_proxy_response(retry_resp, &ctx, &router_id, &path, &method).await;
            }
            Err(e) => return build_error_response(&e.to_string()),
        }
    }

    build_proxy_response(resp, &ctx, &router_id, &path, &method).await
}

async fn build_proxy_response(
    resp: reqwest::Response,
    ctx: &std::sync::Arc<ProxyCtx>,
    router_id: &str,
    path: &str,
    method: &reqwest::Method,
) -> Result<Response<ProxyBody>, hyper::Error> {
    let status =
        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let response_content_type = header_value(resp.headers(), "content-type");
    let response_length = content_length(resp.headers());
    let response_content_disposition = header_value(resp.headers(), "content-disposition");
    let filename = response_content_disposition
        .as_deref()
        .and_then(content_disposition_filename);
    let report_download = is_download_response(&resp);

    let response_is_html = is_html(&resp);
    let headers: Vec<_> = resp
        .headers()
        .iter()
        .filter(|(name, _)| {
            let n = name.as_str().to_lowercase();
            !(matches!(
                n.as_str(),
                "transfer-encoding"
                    | "connection"
                    | "keep-alive"
                    | "proxy-authenticate"
                    | "proxy-authorization"
                    | "te"
                    | "trailer"
                    | "upgrade"
                    | "content-security-policy"
                    | "content-security-policy-report-only"
                    | "x-frame-options"
                    | "x-content-type-options"
                    | "cross-origin-embedder-policy"
                    | "cross-origin-opener-policy"
                    | "cross-origin-resource-policy"
            ) || response_is_html && n == "content-disposition")
        })
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();

    if report_download
        && method != reqwest::Method::HEAD
        && status != StatusCode::NO_CONTENT
        && status != StatusCode::NOT_MODIFIED
    {
        return cache_download_response(
            resp,
            ctx,
            path,
            filename,
            response_content_type,
            response_content_disposition,
            response_length,
            status.as_u16(),
        )
        .await
        .or_else(|error| build_error_response(&error));
    }

    let body = if method == reqwest::Method::HEAD
        || status == StatusCode::NO_CONTENT
        || status == StatusCode::NOT_MODIFIED
    {
        if report_download {
            emit_router_transfer(
                &ctx.app,
                router_transfer_event(
                    "finished",
                    "download",
                    path,
                    filename,
                    response_content_type,
                    response_length,
                    0,
                    0,
                    Some(status.as_u16()),
                    None,
                ),
            );
        }
        full_proxy_body(Bytes::new())
    } else {
        use futures_util::TryStreamExt as _;
        let body_stream = tracked_transfer_stream(
            resp.bytes_stream(),
            ctx.app.clone(),
            "download",
            path.to_string(),
            filename,
            response_content_type,
            response_length,
            Some(status.as_u16()),
            report_download,
        );
        StreamBody::new(body_stream.map_ok(Frame::data)).boxed_unsync()
    };

    let mut builder = Response::builder().status(status);
    for (name, value) in headers {
        if let Ok(v) = value.to_str() {
            let value = if name.as_str().eq_ignore_ascii_case("location") {
                rewrite_location(v, &ctx.router_ip)
            } else {
                v.to_string()
            };
            builder = builder.header(name.as_str(), value);
        }
    }
    builder = builder.header("cross-origin-resource-policy", "cross-origin");
    builder = builder.header(
        "set-cookie",
        format!("__et_router={router_id}; Path=/; SameSite=Lax"),
    );
    if response_is_html {
        builder = builder.header("x-content-type-options", "nosniff");
    }
    Ok(builder.body(body).unwrap())
}

fn build_error_response(msg: &str) -> Result<Response<ProxyBody>, hyper::Error> {
    let builder = Response::builder()
        .status(StatusCode::BAD_GATEWAY)
        .header("content-type", "text/plain; charset=utf-8");
    Ok(builder
        .body(full_proxy_body(Bytes::from(msg.to_string())))
        .unwrap())
}

async fn run_proxy(
    listener: TcpListener,
    state: std::sync::Arc<ProxyState>,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = &mut shutdown_rx => { break; }
            result = listener.accept() => {
                match result {
                    Ok((stream, _addr)) => {
                        let io = TokioIo::new(stream);
                        let state2 = state.clone();
                        tokio::spawn(async move {
                            let svc = service_fn(move |req| {
                                let state3 = state2.clone();
                                async move { proxy_request(req, &state3).await }
                            });
                            let _ = http1::Builder::new()
                                .half_close(true)
                                .serve_connection(io, svc)
                                .await;
                        });
                    }
                    Err(_) => { /* accept error, continue */ }
                }
            }
        }
    }
}

fn stop_proxy_inner() {
    let mut guard = PROXY.lock().unwrap();
    if let Some(handle) = guard.take() {
        let _ = handle.shutdown_tx.send(());
    }
}

async fn bind_proxy_listener(preferred_port: Option<u16>) -> Result<TcpListener, String> {
    for port in preferred_port.into_iter().chain(std::iter::once(0)) {
        for _ in 0..20 {
            let socket =
                TcpSocket::new_v4().map_err(|e| format!("Failed to create proxy socket: {e}"))?;
            socket
                .set_reuseaddr(true)
                .map_err(|e| format!("Failed to configure proxy socket: {e}"))?;
            if let Err(e) = socket.bind(std::net::SocketAddr::from(([127, 0, 0, 1], port))) {
                if port != 0 {
                    let _ = e;
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                    continue;
                }
                return Err(format!("Failed to bind local port {port}: {e}"));
            }
            match socket.listen(1024) {
                Ok(listener) => return Ok(listener),
                Err(e) if port != 0 => {
                    let _ = e;
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
                Err(e) => return Err(format!("Failed to listen on local port {port}: {e}")),
            }
        }
    }
    Err("Failed to bind local proxy port".to_string())
}

#[tauri::command]
async fn luci_proxy_start(
    app: AppHandle,
    router_id: String,
    router_ip: String,
    username: String,
    password: String,
    socks5_proxy: Option<String>,
) -> Result<String, String> {
    if router_id.trim().is_empty() {
        return Err("router_id is required".to_string());
    }

    let path_type = if socks5_proxy.is_some() {
        "tunnel"
    } else {
        "lan"
    };
    if let Some(handle) = PROXY.lock().unwrap().as_ref()
        && let Some(ctx) = handle.state.router(&router_id)
        && ctx.router_ip == router_ip
        && *ctx.shared.last_path_type.lock().unwrap() == path_type
    {
        *handle.state.active_router_id.lock().unwrap() = router_id;
        return Ok(format!("http://127.0.0.1:{}", handle.port));
    }

    let client = build_proxy_client(&socks5_proxy)?;
    let cookie = luci_login(&client, &router_ip, &username, &password).await?;
    let download_dir = app
        .path()
        .cache_dir()
        .map(|path| path.join("router-downloads"))
        .unwrap_or_else(|_| std::env::temp_dir().join("easytier-router-downloads"));
    tokio::fs::create_dir_all(&download_dir)
        .await
        .map_err(|e| format!("Failed to create download cache directory: {e}"))?;

    let shared = std::sync::Arc::new(SharedState {
        last_path: std::sync::Mutex::new("/cgi-bin/luci/admin/".to_string()),
        last_path_type: std::sync::Mutex::new(path_type.to_string()),
    });
    let ctx = std::sync::Arc::new(ProxyCtx {
        app,
        client,
        cookie: Mutex::new(cookie),
        router_ip,
        username,
        password,
        shared,
        download_cache: std::sync::Mutex::new(HashMap::new()),
        last_download: std::sync::Mutex::new(None),
        download_dir: download_dir.clone(),
    });

    if let Some(handle) = PROXY.lock().unwrap().as_ref() {
        handle
            .state
            .routers
            .write()
            .unwrap()
            .insert(router_id.clone(), ctx);
        *handle.state.active_router_id.lock().unwrap() = router_id;
        return Ok(format!("http://127.0.0.1:{}", handle.port));
    }

    let listener = bind_proxy_listener(Some(32260)).await?;
    let port = listener.local_addr().unwrap().port();
    let state = std::sync::Arc::new(ProxyState {
        routers: std::sync::RwLock::new(HashMap::from([(router_id.clone(), ctx)])),
        active_router_id: std::sync::Mutex::new(router_id),
    });

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let state_for_server = state.clone();
    tokio::spawn(async move { run_proxy(listener, state_for_server, shutdown_rx).await });
    *PROXY.lock().unwrap() = Some(ProxyHandle {
        port,
        shutdown_tx,
        state,
    });

    Ok(format!("http://127.0.0.1:{}", port))
}

#[tauri::command]
async fn luci_proxy_stop() -> Result<(), String> {
    stop_proxy_inner();
    Ok(())
}

#[tauri::command]
async fn luci_get_last_path(router_id: Option<String>) -> Result<String, String> {
    let guard = PROXY.lock().unwrap();
    let Some(handle) = guard.as_ref() else {
        return Ok("/cgi-bin/luci/admin/".to_string());
    };
    let router_id = router_id.unwrap_or_else(|| handle.state.active_router_id());
    match handle.state.router(&router_id) {
        Some(ctx) => Ok(ctx.shared.last_path.lock().unwrap().clone()),
        None => Ok("/cgi-bin/luci/admin/".to_string()),
    }
}

// 获取日志目录的辅助函数
fn get_log_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, tauri::Error> {
    if cfg!(target_os = "android") {
        // Android: cache_dir + logs 子目录
        app.path().cache_dir().map(|p| p.join("logs"))
    } else {
        // 其他平台: 默认日志目录
        app.path().app_log_dir()
    }
}

#[tauri::command]
async fn get_log_dir_path(app: tauri::AppHandle) -> Result<String, String> {
    match get_log_dir(&app) {
        Ok(log_dir) => {
            std::fs::create_dir_all(&log_dir).ok();
            Ok(log_dir.to_string_lossy().to_string())
        }
        Err(e) => Err(format!("Failed to get log directory: {}", e)),
    }
}

#[cfg(not(target_os = "android"))]
fn toggle_window_visibility(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or_default();
        let minimized = window.is_minimized().unwrap_or_default();
        let focused = window.is_focused().unwrap_or_default();

        let should_show = !visible || minimized || !focused;
        if should_show {
            if !visible {
                let _ = window.show();
            }
            if minimized {
                let _ = window.unminimize();
            }
            if !focused {
                let _ = window.set_focus();
            }
            let _ = set_dock_visibility(app.clone(), true);
        } else {
            let _ = window.hide();
            let _ = set_dock_visibility(app.clone(), false);
        }
    }
}

fn get_exe_path() -> String {
    if let Ok(appimage_path) = std::env::var("APPIMAGE")
        && !appimage_path.is_empty()
    {
        return appimage_path;
    }
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

#[cfg(not(target_os = "android"))]
fn check_sudo() -> bool {
    let is_elevated = elevate::Command::is_elevated();
    if !is_elevated {
        let exe_path = get_exe_path();
        let stdcmd = std::process::Command::new(&exe_path);
        elevate::Command::new(stdcmd)
            .output()
            .expect("Failed to run elevated command");
    }
    is_elevated
}

mod manager {
    use super::*;
    use async_trait::async_trait;
    use dashmap::{DashMap, DashSet};
    use easytier::common::config::{NetworkConfig, NetworkConfigExt};
    use easytier::proto::api::logger::{LoggerRpc, LoggerRpcClientFactory, SetLoggerConfigRequest};
    use easytier::proto::api::manage::RunNetworkInstanceRequest;
    use easytier::proto::rpc::bidirect::BidirectRpcManager;
    use easytier::proto::rpc_types::controller::BaseController;
    use easytier::web_client::WebClientHooks;
    use easytier_core::management::remote_client::PersistentConfig;

    pub(super) struct GuiHooks {
        pub(super) app: AppHandle,
    }

    #[async_trait]
    impl WebClientHooks for GuiHooks {
        async fn pre_run_network_instance(
            &self,
            cfg: &easytier::common::config::TomlConfigLoader,
        ) -> Result<(), String> {
            let client_manager = get_client_manager!()?;
            client_manager
                .pre_run_network_instance_hook(
                    &self.app,
                    cfg,
                    PersistedConfigSource::from_runtime_source(cfg.get_network_config_source()),
                )
                .await
        }

        async fn post_run_network_instance(&self, instance_id: &uuid::Uuid) -> Result<(), String> {
            let client_manager = get_client_manager!()?;
            client_manager
                .post_run_network_instance_hook(&self.app, instance_id)
                .await
        }

        async fn post_remove_network_instances(&self, ids: &[uuid::Uuid]) -> Result<(), String> {
            let client_manager = get_client_manager!()?;
            client_manager
                .post_remote_remove_network_instances_hook(&self.app, ids)
                .await
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    #[derive(Default)]
    pub(super) enum PersistedConfigSource {
        User,
        #[serde(alias = "webhook")]
        Web,
        #[serde(other)]
        #[default]
        Legacy,
    }

    impl PersistedConfigSource {
        pub(super) fn from_runtime_source(source: ConfigSource) -> Self {
            match source {
                ConfigSource::User => Self::User,
                ConfigSource::Web => Self::Web,
            }
        }

        fn merge_persisted(self, incoming: Self) -> Self {
            match (self, incoming) {
                // Older runtimes report missing source as `user`. Keep the stronger persisted
                // ownership until web sync or an explicit user save repairs it.
                (Self::Web, Self::User) | (Self::Legacy, Self::User) => self,
                (_, next) => next,
            }
        }

        fn to_runtime_source(self) -> ConfigSource {
            match self {
                Self::User | Self::Legacy => ConfigSource::User,
                Self::Web => ConfigSource::Web,
            }
        }

        #[cfg(any(test, target_os = "android"))]
        fn is_web_like(self) -> bool {
            matches!(self, Self::Web)
        }
    }

    #[derive(Clone)]
    pub(super) struct GUIConfig {
        inst_id: String,
        pub(crate) config: NetworkConfig,
        source: PersistedConfigSource,
    }

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    pub(super) struct StoredGuiConfig {
        config: NetworkConfig,
        #[serde(default)]
        source: PersistedConfigSource,
    }

    impl GUIConfig {
        fn new(inst_id: String, config: NetworkConfig, source: PersistedConfigSource) -> Self {
            Self {
                inst_id,
                config,
                source,
            }
        }

        fn into_stored(self) -> StoredGuiConfig {
            StoredGuiConfig {
                config: self.config,
                source: self.source,
            }
        }
    }

    impl PersistentConfig<anyhow::Error> for GUIConfig {
        fn get_network_inst_id(&self) -> &str {
            &self.inst_id
        }
        fn get_network_config(&self) -> Result<NetworkConfig, anyhow::Error> {
            Ok(self.config.clone())
        }
        fn get_network_config_source(&self) -> ConfigSource {
            self.source.to_runtime_source()
        }
    }

    pub(super) struct GUIStorage {
        network_configs: DashMap<Uuid, GUIConfig>,
        enabled_networks: DashSet<Uuid>,
    }
    impl GUIStorage {
        fn new() -> Self {
            Self {
                network_configs: DashMap::new(),
                enabled_networks: DashSet::new(),
            }
        }

        fn save_configs(&self, app: &AppHandle) -> anyhow::Result<()> {
            let configs = self
                .network_configs
                .iter()
                .map(|entry| entry.value().clone().into_stored())
                .collect::<Vec<_>>();
            app.emit("save_configs", configs)?;
            Ok(())
        }

        fn save_enabled_networks(&self, app: &AppHandle) -> anyhow::Result<()> {
            let payload: Vec<String> = self
                .enabled_networks
                .iter()
                .map(|entry| entry.key().to_string())
                .collect();
            app.emit("save_enabled_networks", payload)?;
            Ok(())
        }

        fn save_config(
            &self,
            app: &AppHandle,
            inst_id: Uuid,
            cfg: NetworkConfig,
            source: PersistedConfigSource,
        ) -> anyhow::Result<()> {
            let source = self
                .network_configs
                .get(&inst_id)
                .map(|existing| existing.source.merge_persisted(source))
                .unwrap_or(source);
            let config = GUIConfig::new(inst_id.to_string(), cfg, source);
            self.network_configs.insert(inst_id, config);
            self.save_configs(app)
        }
    }
    #[async_trait]
    impl Storage<AppHandle, GUIConfig, anyhow::Error> for GUIStorage {
        async fn insert_or_update_user_network_config(
            &self,
            app: AppHandle,
            network_inst_id: Uuid,
            network_config: NetworkConfig,
            source: ConfigSource,
        ) -> Result<(), anyhow::Error> {
            self.save_config(
                &app,
                network_inst_id,
                network_config,
                PersistedConfigSource::from_runtime_source(source),
            )?;
            self.enabled_networks.insert(network_inst_id);
            self.save_enabled_networks(&app)?;
            Ok(())
        }

        async fn delete_network_configs(
            &self,
            app: AppHandle,
            network_inst_ids: &[Uuid],
        ) -> Result<(), anyhow::Error> {
            for network_inst_id in network_inst_ids {
                self.network_configs.remove(network_inst_id);
                self.enabled_networks.remove(network_inst_id);
            }
            self.save_configs(&app)?;
            self.save_enabled_networks(&app)?;
            Ok(())
        }

        async fn update_network_config_state(
            &self,
            app: AppHandle,
            network_inst_id: Uuid,
            disabled: bool,
        ) -> Result<(), anyhow::Error> {
            if disabled {
                self.enabled_networks.remove(&network_inst_id);
            } else {
                self.enabled_networks.insert(network_inst_id);
            }
            self.save_enabled_networks(&app)?;
            Ok(())
        }

        async fn list_network_configs(
            &self,
            _: AppHandle,
            props: ListNetworkProps,
        ) -> Result<Vec<GUIConfig>, anyhow::Error> {
            let mut ret = Vec::new();
            for entry in self.network_configs.iter() {
                let id: Uuid = entry.key().to_owned();
                match props {
                    ListNetworkProps::All => {
                        ret.push(entry.value().clone());
                    }
                    ListNetworkProps::EnabledOnly => {
                        if self.enabled_networks.contains(&id) {
                            ret.push(entry.value().clone());
                        }
                    }
                    ListNetworkProps::DisabledOnly => {
                        if !self.enabled_networks.contains(&id) {
                            ret.push(entry.value().clone());
                        }
                    }
                }
            }
            Ok(ret)
        }

        async fn get_network_config(
            &self,
            _: AppHandle,
            network_inst_id: &str,
        ) -> Result<Option<GUIConfig>, anyhow::Error> {
            let uuid = Uuid::parse_str(network_inst_id)?;
            Ok(self
                .network_configs
                .get(&uuid)
                .map(|entry| entry.value().clone()))
        }
    }

    pub(super) struct GUIClientManager {
        pub(super) storage: GUIStorage,
        pub(super) rpc_manager: BidirectRpcManager,
        pub(super) config_mutation: Mutex<()>,
    }
    impl GUIClientManager {
        pub async fn new(
            rpc_url: Option<String>,
            local_process_runtime: Option<Arc<CoreProcessRuntime>>,
        ) -> Result<Self, anyhow::Error> {
            let tunnel = if let Some(url) = rpc_url {
                runtime_rpc_dialer(url.parse()?).connect().await?
            } else {
                local_process_runtime
                    .context("local RPC requires a core process runtime")?
                    .connect_ring_tunnel(*RPC_RING_UUID.deref())?
            };

            let rpc_manager = BidirectRpcManager::new();
            rpc_manager.run_with_tunnel(tunnel);

            Ok(Self {
                storage: GUIStorage::new(),
                rpc_manager,
                config_mutation: Mutex::new(()),
            })
        }

        pub fn get_enabled_instances_with_tun_ids(&self) -> impl Iterator<Item = uuid::Uuid> + '_ {
            self.storage
                .network_configs
                .iter()
                .filter(|v| self.storage.enabled_networks.contains(v.key()))
                .filter(|v| !v.config.no_tun())
                .filter_map(|c| c.config.instance_id().parse::<uuid::Uuid>().ok())
        }

        #[cfg(target_os = "android")]
        pub fn get_enabled_instances_with_web_like_tun_ids(
            &self,
        ) -> impl Iterator<Item = uuid::Uuid> + '_ {
            self.storage
                .network_configs
                .iter()
                .filter(|v| self.storage.enabled_networks.contains(v.key()))
                .filter(|v| !v.config.no_tun())
                .filter(|v| v.source.is_web_like())
                .filter_map(|c| c.config.instance_id().parse::<uuid::Uuid>().ok())
        }

        #[cfg(target_os = "android")]
        pub(super) async fn disable_instances_with_tun(
            &self,
            app: &AppHandle,
            web_only: bool,
        ) -> Result<(), easytier_core::management::remote_client::RemoteClientError<anyhow::Error>>
        {
            let inst_ids: Vec<uuid::Uuid> = if web_only {
                self.get_enabled_instances_with_web_like_tun_ids().collect()
            } else {
                self.get_enabled_instances_with_tun_ids().collect()
            };
            for inst_id in inst_ids {
                self.handle_update_network_state(app.clone(), inst_id, true)
                    .await?;
            }
            Ok(())
        }

        pub(super) fn notify_vpn_stop_if_no_tun(&self, app: &AppHandle) -> Result<(), String> {
            let has_tun = self.get_enabled_instances_with_tun_ids().any(|_| true);
            if !has_tun {
                app.emit("vpn_service_stop", "")
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        }

        pub(super) async fn pre_run_network_instance_hook(
            &self,
            app: &AppHandle,
            cfg: &easytier::common::config::TomlConfigLoader,
            source: PersistedConfigSource,
        ) -> Result<(), String> {
            let instance_id = cfg.get_id();
            app.emit("pre_run_network_instance", instance_id.to_string())
                .map_err(|e| e.to_string())?;

            #[cfg(target_os = "android")]
            if !cfg.get_flags().no_tun {
                match source {
                    PersistedConfigSource::User | PersistedConfigSource::Legacy => {
                        self.disable_instances_with_tun(app, false)
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                    PersistedConfigSource::Web => {
                        self.disable_instances_with_tun(app, true)
                            .await
                            .map_err(|e| e.to_string())?;
                        if self.get_enabled_instances_with_tun_ids().next().is_some() {
                            return Err(
                                "Android only supports one active TUN network; user-managed VPN remains active"
                                    .to_string(),
                            );
                        }
                    }
                }
            }

            self.storage
                .save_config(
                    app,
                    instance_id,
                    NetworkConfig::new_from_config(cfg).map_err(|e| e.to_string())?,
                    source,
                )
                .map_err(|e| e.to_string())?;

            Ok(())
        }

        pub(super) async fn post_run_network_instance_hook(
            &self,
            app: &AppHandle,
            instance_id: &uuid::Uuid,
        ) -> Result<(), String> {
            #[cfg(target_os = "android")]
            if let Some(instance_manager) = super::INSTANCE_MANAGER.read().await.as_ref() {
                let instance_uuid = *instance_id;
                if let Some(instance) = instance_manager.instance(instance_uuid) {
                    if let Some(mut event_receiver) = subscribe_native_instance_event(&instance) {
                        let app_clone = app.clone();
                        let instance_id_clone = *instance_id;
                        tokio::spawn(async move {
                            let instance_id_str = instance_id_clone.to_string();
                            loop {
                                match event_receiver.recv().await {
                                    Ok(easytier::common::global_ctx::GlobalCtxEvent::DhcpIpv4Changed(_, _)) => {
                                        let _ = app_clone.emit("dhcp_ip_changed", &instance_id_str);
                                    }
                                    Ok(easytier::common::global_ctx::GlobalCtxEvent::ProxyCidrsUpdated(_, _)) => {
                                        let _ = app_clone.emit("proxy_cidrs_updated", &instance_id_str);
                                    }
                                    Ok(_) => {}
                                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                        break;
                                    }
                                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                        let _ = app_clone.emit("event_lagged", &instance_id_str);
                                        event_receiver = event_receiver.resubscribe();
                                    }
                                }
                            }
                        });
                    }
                }
            }

            self.storage.enabled_networks.insert(*instance_id);

            app.emit("post_run_network_instance", instance_id.to_string())
                .map_err(|e| e.to_string())?;

            Ok(())
        }

        pub(super) async fn post_remote_remove_network_instances_hook(
            &self,
            app: &AppHandle,
            ids: &[uuid::Uuid],
        ) -> Result<(), String> {
            self.storage
                .delete_network_configs(app.clone(), ids)
                .await
                .map_err(|e| e.to_string())?;
            self.notify_vpn_stop_if_no_tun(app)?;
            Ok(())
        }

        pub(super) async fn post_stop_network_instances_hook(
            &self,
            app: &AppHandle,
        ) -> Result<(), String> {
            self.notify_vpn_stop_if_no_tun(app)?;
            Ok(())
        }

        fn get_logger_rpc_client(
            &self,
        ) -> Option<Box<dyn LoggerRpc<Controller = BaseController> + Send>> {
            Some(
                self.rpc_manager
                    .rpc_client()
                    .scoped_client::<LoggerRpcClientFactory<BaseController>>(1, 1, "".to_string()),
            )
        }

        pub(super) async fn set_logging_level(&self, level: String) -> Result<(), anyhow::Error> {
            let logger_rpc = self
                .get_logger_rpc_client()
                .ok_or_else(|| anyhow::anyhow!("Logger RPC client not available"))?;
            logger_rpc
                .set_logger_config(
                    BaseController::default(),
                    SetLoggerConfigRequest {
                        level: easytier_core::management::parse_log_level(&level).into(),
                    },
                )
                .await?;
            Ok(())
        }

        pub(super) async fn load_configs(
            &self,
            app: AppHandle,
            configs: Vec<StoredGuiConfig>,
            enabled_networks: Vec<String>,
        ) -> anyhow::Result<()> {
            self.storage.network_configs.clear();
            for stored in configs {
                let instance_id = stored.config.instance_id();
                self.storage.network_configs.insert(
                    instance_id.parse()?,
                    GUIConfig::new(instance_id.to_string(), stored.config, stored.source),
                );
            }

            self.storage.enabled_networks.clear();
            let client = self
                .get_rpc_client(app.clone())
                .ok_or_else(|| anyhow::anyhow!("RPC client not found"))?;
            for id in enabled_networks {
                if let Ok(uuid) = id.parse()
                    && !self.storage.enabled_networks.contains(&uuid)
                {
                    let config = self
                        .storage
                        .network_configs
                        .get(&uuid)
                        .map(|i| (i.value().config.clone(), i.value().source));
                    let Some((config, source)) = config else {
                        continue;
                    };
                    let toml_config = config.gen_config()?;
                    self.pre_run_network_instance_hook(&app, &toml_config, source)
                        .await
                        .map_err(|e| anyhow::anyhow!(e))?;
                    client
                        .run_network_instance(
                            BaseController::default(),
                            RunNetworkInstanceRequest {
                                inst_id: None,
                                config: Some(config),
                                overwrite: false,
                                source: config_source_to_rpc(source.to_runtime_source()),
                            },
                        )
                        .await?;
                    self.post_run_network_instance_hook(&app, &uuid)
                        .await
                        .map_err(|e| anyhow::anyhow!(e))?;
                }
            }
            Ok(())
        }
    }
    impl RemoteClientManager<AppHandle, GUIConfig, anyhow::Error> for GUIClientManager {
        fn get_config_rpc_client(
            &self,
            _: AppHandle,
        ) -> Option<Box<dyn ConfigRpc<Controller = BaseController> + Send>> {
            Some(
                self.rpc_manager
                    .rpc_client()
                    .scoped_client::<ConfigRpcClientFactory<BaseController>>(1, 1, String::new()),
            )
        }

        fn get_rpc_client(
            &self,
            _: AppHandle,
        ) -> Option<Box<dyn WebClientService<Controller = BaseController> + Send>> {
            Some(
                self.rpc_manager
                    .rpc_client()
                    .scoped_client::<WebClientServiceClientFactory<BaseController>>(
                        1,
                        1,
                        "".to_string(),
                    ),
            )
        }

        fn get_storage(&self) -> &impl Storage<AppHandle, GUIConfig, anyhow::Error> {
            &self.storage
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{PersistedConfigSource, StoredGuiConfig};
        use easytier::proto::api::manage::NetworkConfig;

        #[test]
        fn stored_gui_config_defaults_missing_source_to_legacy() {
            let stored: StoredGuiConfig = serde_json::from_value(serde_json::json!({
                "config": NetworkConfig::default(),
            }))
            .unwrap();
            assert_eq!(stored.source, PersistedConfigSource::Legacy);
        }

        #[test]
        fn stored_gui_config_deserializes_webhook_source_as_web() {
            let stored: StoredGuiConfig = serde_json::from_value(serde_json::json!({
                "config": NetworkConfig::default(),
                "source": "webhook",
            }))
            .unwrap();
            assert_eq!(stored.source, PersistedConfigSource::Web);
        }

        #[test]
        fn stored_gui_config_defaults_unknown_source_to_legacy() {
            let stored: StoredGuiConfig = serde_json::from_value(serde_json::json!({
                "config": NetworkConfig::default(),
                "source": "unknown",
            }))
            .unwrap();
            assert_eq!(stored.source, PersistedConfigSource::Legacy);
        }

        #[test]
        fn persisted_source_merge_keeps_legacy_and_web_over_ambiguous_user() {
            assert_eq!(
                PersistedConfigSource::Legacy.merge_persisted(PersistedConfigSource::User),
                PersistedConfigSource::Legacy
            );
            assert_eq!(
                PersistedConfigSource::Web.merge_persisted(PersistedConfigSource::User),
                PersistedConfigSource::Web
            );
            assert_eq!(
                PersistedConfigSource::Legacy.merge_persisted(PersistedConfigSource::Web),
                PersistedConfigSource::Web
            );
        }

        #[test]
        fn only_web_configs_are_web_like() {
            assert!(!PersistedConfigSource::Legacy.is_web_like());
            assert!(!PersistedConfigSource::User.is_web_like());
            assert!(PersistedConfigSource::Web.is_web_like());
        }
    }
}

#[cfg(not(target_os = "android"))]
mod service {
    use anyhow::Context;

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    pub struct ServiceOptions {
        pub(super) config_dir: String,
        pub(super) rpc_portal: String,
        pub(super) file_log_level: String,
        pub(super) file_log_dir: String,
        pub(super) config_server: Option<String>,
    }
    impl ServiceOptions {
        fn to_args_vec(&self) -> Vec<std::ffi::OsString> {
            let mut args = vec![
                "--config-dir".into(),
                self.config_dir.clone().into(),
                "--rpc-portal".into(),
                self.rpc_portal.clone().into(),
                "--file-log-level".into(),
                self.file_log_level.clone().into(),
                "--file-log-dir".into(),
                self.file_log_dir.clone().into(),
                "--daemon".into(),
            ];

            if let Some(config_server) = &self.config_server {
                args.push("--config-server".into());
                args.push(config_server.clone().into());
            }

            args
        }
    }

    #[cfg(target_os = "macos")]
    fn service_environment() -> Option<Vec<(String, String)>> {
        // System LaunchDaemons run as root but launchd does not provide HOME.
        Some(vec![("HOME".to_string(), "/var/root".to_string())])
    }

    #[cfg(not(target_os = "macos"))]
    fn service_environment() -> Option<Vec<(String, String)>> {
        None
    }

    pub fn install(opts: ServiceOptions) -> anyhow::Result<()> {
        let service = easytier::service_manager::Service::new(env!("CARGO_PKG_NAME").to_string())?;
        let options = easytier::service_manager::ServiceInstallOptions {
            program: super::get_exe_path().into(),
            args: opts.to_args_vec(),
            work_directory: std::env::current_dir()?,
            environment: service_environment(),
            disable_autostart: false,
            description: Some("EasyTier Gui Service".to_string()),
            display_name: Some("EasyTier Gui Service".to_string()),
            disable_restart_on_failure: false,
        };
        service
            .install(&options)
            .with_context(|| "Failed to install service")?;
        Ok(())
    }

    pub fn uninstall() -> anyhow::Result<()> {
        let service = easytier::service_manager::Service::new(env!("CARGO_PKG_NAME").to_string())?;
        service.uninstall()?;
        Ok(())
    }

    pub fn set_status(enable: bool) -> anyhow::Result<()> {
        use easytier::service_manager::*;
        let service = Service::new(env!("CARGO_PKG_NAME").to_string())?;
        let status = service.status()?;
        if enable && status != ServiceStatus::Running {
            service.start().with_context(|| "Failed to start service")?;
        } else if !enable && status == ServiceStatus::Running {
            service.stop().with_context(|| "Failed to stop service")?;
        } else if status == ServiceStatus::NotInstalled {
            return Err(anyhow::anyhow!("Service not installed"));
        }
        Ok(())
    }

    pub fn status() -> anyhow::Result<easytier::service_manager::ServiceStatus> {
        let service = easytier::service_manager::Service::new(env!("CARGO_PKG_NAME").to_string())?;
        service.status()
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn service_environment_matches_platform() {
            #[cfg(target_os = "macos")]
            assert_eq!(
                super::service_environment(),
                Some(vec![("HOME".to_string(), "/var/root".to_string())])
            );

            #[cfg(not(target_os = "macos"))]
            assert_eq!(super::service_environment(), None);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_gui() -> std::process::ExitCode {
    #[cfg(not(target_os = "android"))]
    if !check_sudo() {
        use std::process;
        process::exit(0);
    }

    setup_panic_handler();

    let mut builder = tauri::Builder::default();

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            app.webview_windows()
                .values()
                .next()
                .expect("Sorry, no window found")
                .set_focus()
                .expect("Can't Bring Window to Focus");
        }));
    }

    builder = builder
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_vpnservice::init());

    let app = builder
        .setup(|app| {
            // for logging config
            let Ok(log_dir) = get_log_dir(app.app_handle()) else {
                return Ok(());
            };
            let config = LoggingConfig::builder()
                .file_logger(FileLoggerConfig {
                    dir: Some(log_dir.to_string_lossy().to_string()),
                    level: None,
                    file: None,
                    size_mb: None,
                    count: None,
                })
                .build();
            let Ok(_) = log::init(&config, true) else {
                return Ok(());
            };

            // for tray icon, menu need to be built in js
            #[cfg(not(target_os = "android"))]
            let _tray_menu = TrayIconBuilder::with_id("main")
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        toggle_window_visibility(app);
                    }
                })
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/icon.png"
                ))?)
                .icon_as_template(true)
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            parse_network_config,
            generate_network_config,
            run_network_instance,
            collect_network_info,
            get_vpn_portal_info,
            patch_vpn_portal_clients,
            set_logging_level,
            set_tun_fd,
            easytier_version,
            set_dock_visibility,
            list_network_instance_ids,
            remove_network_instance,
            update_network_config_state,
            save_network_config,
            validate_config,
            get_config,
            load_configs,
            get_network_metas,
            init_service,
            set_service_status,
            get_service_status,
            init_rpc_connection,
            is_client_running,
            init_web_client,
            is_web_client_connected,
            http_get,
            http_post,
            http_request,
            tcp_ping,
            luci_proxy_start,
            luci_proxy_stop,
            luci_get_last_path,
            get_log_dir_path,
        ])
        .on_window_event(|_win, event| match event {
            #[cfg(not(target_os = "android"))]
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = _win.hide();
                let _ = set_dock_visibility(_win.app_handle().clone(), false);
                api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .unwrap();

    app.run(|_app, _event| {});

    std::process::ExitCode::SUCCESS
}

pub fn run_cli() -> std::process::ExitCode {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { easytier::core::main().await })
}
