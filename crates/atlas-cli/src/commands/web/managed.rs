use std::{
    fs, io,
    net::{Ipv4Addr, SocketAddrV4},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};

use atlas_app_service::{AppServiceRetrievalMode, AtlasAppService, AtlasAppServiceOptions};
use atlas_runtime::{AtlasPathOverrides, AtlasRuntime, AtlasRuntimeOptions, Workspace};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Notify,
    task::JoinSet,
    time::{Instant, sleep, timeout},
};

use super::{WebArgs, bind_local_listener, open_browser};

const STARTUP: Duration = Duration::from_secs(15);
const DRAIN: Duration = Duration::from_secs(5);
const PROTOCOL: u32 = 1;
const HTTP_PROOF: u8 = 0;
const CONTROL_PROOF: u8 = 1;
const FOREGROUND_LEASE: u8 = 2;
const SEARCH_LEASE: u8 = 3;
const SEMANTIC_PUBLICATION: u8 = 4;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Settings {
    search_profile: Option<atlas_domain::SearchProfileId>,
    source_root: PathBuf,
    embedding_cache_root: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Endpoint {
    protocol: u32,
    build: String,
    workspace: PathBuf,
    settings: Settings,
    port: u16,
    control_port: u16,
    launch_id: String,
    secret: [u8; 32],
}

impl Endpoint {
    pub(crate) fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub(crate) fn supports_cli(&self, runtime: &AtlasRuntime) -> bool {
        self.build == env!("CARGO_PKG_VERSION")
            && (runtime.requested_search_profile().is_some()
                || self.settings.search_profile.is_none())
            && self.settings.embedding_cache_root == runtime.embedding_cache_root()
    }
}

fn endpoint_path(workspace: &Workspace, create: bool) -> Result<PathBuf, String> {
    let global =
        AtlasRuntime::resolve(AtlasRuntimeOptions::default()).map_err(|error| error.to_string())?;
    let directory = global.index_path().with_file_name("services");
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    if create {
        builder
            .create(&directory)
            .map_err(|error| error.to_string())?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::symlink_metadata(&directory).map_err(|error| error.to_string())?;
        if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
            return Err("Atlas service discovery directory must be private".into());
        }
    }
    let key = Sha256::digest(workspace.path().as_os_str().as_encoded_bytes());
    Ok(directory.join(format!("{key:x}.json")))
}

fn read_endpoint(path: &Path, workspace: &Workspace) -> Option<Endpoint> {
    let endpoint: Endpoint = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    (endpoint.workspace == workspace.path() && endpoint.protocol == PROTOCOL).then_some(endpoint)
}

pub(crate) fn discover(runtime: &AtlasRuntime) -> Option<Endpoint> {
    let workspace = Workspace::existing(runtime.index_path()).ok()?;
    read_endpoint(&endpoint_path(&workspace, false).ok()?, &workspace)
}

fn proof(secret: &[u8; 32], role: u8, challenge: &[u8]) -> String {
    let mut mac = hmac_sha256::HMAC::new(secret);
    mac.update(b"atlas-service-v1");
    mac.update([role]);
    mac.update(challenge);
    format!("{:x}", sha2::digest::Output::<Sha256>::from(mac.finalize()))
}

fn prove_http(endpoint: &Endpoint) -> Option<()> {
    let probe: ureq::Agent = ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_millis(500)))
        .build()
        .into();
    let challenge = format!("{:032x}", rand::random::<u128>());
    let response = probe
        .get(format!("{}/api/compatibility", endpoint.url()))
        .header("x-atlas-api-contract", atlas_web::API_CONTRACT)
        .header("x-atlas-service-challenge", &challenge)
        .call()
        .ok()?;
    (response.headers().get("x-atlas-api-contract")?.as_bytes()
        == atlas_web::API_CONTRACT.as_bytes()
        && response.headers().get("x-atlas-service-proof")?.as_bytes()
            == proof(&endpoint.secret, HTTP_PROOF, challenge.as_bytes()).as_bytes())
    .then_some(())
}

pub(crate) fn search(
    runtime: &AtlasRuntime,
    request: &atlas_app_model::SearchResultsRequest,
) -> Option<atlas_app_service::AppServiceResult<atlas_app_model::SearchResultsPage>> {
    use atlas_app_model::{AppError, AppErrorCode};
    let endpoint = discover(runtime)?;
    if !endpoint.supports_cli(runtime) {
        return None;
    }
    // Keep the verified instance alive across HTTP discovery and dispatch.
    let control = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    let _hold = control.block_on(async {
        timeout(Duration::from_millis(500), attach(&endpoint, SEARCH_LEASE))
            .await
            .ok()?
            .ok()
    })?;
    prove_http(&endpoint)?;
    let challenge = format!("{:032x}", rand::random::<u128>());
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(60)))
        .build()
        .into();
    tracing::info!(target: "atlas_cli", "using the running Atlas service for search");
    Some((|| {
        let transport_error = |error: &dyn std::fmt::Display| {
            AppError::new(
                AppErrorCode::QueryFailed,
                format!("Atlas service search failed: {error}"),
            )
        };
        let body = serde_json::to_vec(request).map_err(|error| transport_error(&error))?;
        let mut response = agent
            .post(format!("{}/api/search/page", endpoint.url()))
            .header("x-atlas-api-contract", atlas_web::API_CONTRACT)
            .header("content-type", "application/json")
            .header("x-atlas-service-challenge", &challenge)
            .send(body)
            .map_err(|error| transport_error(&error))?;
        if response
            .headers()
            .get("x-atlas-service-proof")
            .is_none_or(|value| {
                value.as_bytes()
                    != proof(&endpoint.secret, HTTP_PROOF, challenge.as_bytes()).as_bytes()
            })
        {
            return Err(transport_error(&"service identity changed during search").into());
        }
        let bytes = response
            .body_mut()
            .with_config()
            .limit(32 * 1024 * 1024)
            .read_to_vec()
            .map_err(|error| transport_error(&error))?;
        if response.status().is_success() {
            serde_json::from_slice(&bytes).map_err(|error| transport_error(&error).into())
        } else {
            let error: AppError =
                serde_json::from_slice(&bytes).map_err(|error| transport_error(&error))?;
            Err(error.into())
        }
    })())
}

pub(crate) fn notify_semantic_publication(runtime: &AtlasRuntime) {
    let Some(endpoint) = discover(runtime) else {
        return;
    };
    let Ok(control) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return;
    };
    control.block_on(async {
        let _ = timeout(
            Duration::from_millis(500),
            attach(&endpoint, SEMANTIC_PUBLICATION),
        )
        .await;
    });
}

fn settings(runtime: &AtlasRuntime) -> Settings {
    Settings {
        search_profile: runtime.requested_search_profile().cloned(),
        source_root: runtime.source_root().to_owned(),
        embedding_cache_root: runtime.embedding_cache_root().to_owned(),
    }
}

pub(super) async fn run(args: WebArgs) -> Result<(), String> {
    let path_mode = crate::cli::runtime_path_mode(args.path_mode);
    let runtime = AtlasRuntime::resolve(AtlasRuntimeOptions {
        default_search_profile: args.search_profile.clone(),
        path_mode,
        overrides: AtlasPathOverrides {
            source_root: args.source.clone(),
            embedding_cache_root: args.embedding_cache_path.clone(),
            index_path: args.index.clone(),
        },
    })
    .map_err(|error| error.to_string())?;
    let workspace = runtime
        .workspace_location()
        .map_err(|error| error.to_string())?;
    let path = endpoint_path(&workspace, true)?;
    if let Some(launch_id) = args.managed_worker.clone() {
        let _lock = match workspace.service_guard() {
            Ok(lock) => lock,
            Err(atlas_runtime::StorageError::Busy) => return Ok(()),
            Err(error) => return Err(error.to_string()),
        };
        return serve(args, path, &workspace, settings(&runtime), launch_id).await;
    }

    runtime
        .check_workspace_if_present()
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + STARTUP;
    let requested = settings(&runtime);
    let mut child: Option<std::process::Child> = None;
    let launch_id = format!("{:032x}", rand::random::<u128>());
    loop {
        if let Some(endpoint) = read_endpoint(&path, &workspace)
            && let Ok(mut connection) = timeout(
                Duration::from_millis(400),
                attach(&endpoint, FOREGROUND_LEASE),
            )
            .await
            .unwrap_or_else(|_| Err(io::ErrorKind::TimedOut.into()))
        {
            if endpoint.build != env!("CARGO_PKG_VERSION")
                || endpoint.settings != requested
                || args
                    .port
                    .is_some_and(|port| port != 0 && port != endpoint.port)
            {
                return Err(format!(
                    "Atlas is already running at {} with a different version, port or configuration. Stop its launchers before changing the service.",
                    endpoint.url()
                ));
            }
            if endpoint.launch_id != launch_id
                && let Some(mut unused) = child.take()
            {
                let _ = unused.kill();
                let _ = unused.wait();
            }
            let url = endpoint.url();
            let browser_url = args.open.then(|| url.clone());
            if args.desktop_launcher {
                let ready = tokio::task::spawn_blocking(move || {
                    prove_http(&endpoint)?;
                    Some(atlas_desktop_protocol::LauncherReady {
                        version: atlas_desktop_protocol::VERSION,
                        workspace: endpoint.workspace,
                        endpoint: url,
                        instance: endpoint.launch_id,
                        api_contract: atlas_web::API_CONTRACT.to_owned(),
                    })
                })
                .await
                .map_err(|error| error.to_string())?
                .ok_or("The running Atlas backend failed the desktop compatibility check")?;
                println!("{}", serde_json::json!({"status": "ok", "data": ready}));
            } else {
                println!("Atlas web service listening at {url}");
            }
            if let Some(url) = browser_url {
                open_browser(&url);
            }
            let mut end = [0];
            return tokio::select! {
                result = tokio::signal::ctrl_c() => result.map_err(|error| error.to_string()),
                _ = connection.read(&mut end) => Err("Atlas web service disconnected".into()),
            };
        }
        if child.is_none() {
            child = Some(start_worker(&launch_id)?);
        }
        if let Some(process) = &mut child
            && let Some(status) = process.try_wait().map_err(|error| error.to_string())?
        {
            if !status.success() {
                return Err(format!("Atlas web service failed to start ({status})"));
            }
            child = None;
        }
        if Instant::now() >= deadline {
            return Err("timed out starting or attaching to the Atlas web service".into());
        }
        sleep(Duration::from_millis(50)).await;
    }
}

fn start_worker(launch_id: &str) -> Result<std::process::Child, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new(executable);
    command
        .args(std::env::args_os().skip(1))
        .arg("--managed-worker")
        .arg(launch_id)
        .stdin(Stdio::null())
        .stdout(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command.spawn().map_err(|error| error.to_string())
}

async fn attach(endpoint: &Endpoint, role: u8) -> io::Result<TcpStream> {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, endpoint.control_port)).await?;
    let challenge: [u8; 32] = rand::random();
    stream.write_all(&challenge).await?;
    let mut reply = [0; 64];
    stream.read_exact(&mut reply).await?;
    if reply != proof(&endpoint.secret, CONTROL_PROOF, &challenge).as_bytes() {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    let mut challenge = [0; 32];
    stream.read_exact(&mut challenge).await?;
    stream.write_u8(role).await?;
    stream
        .write_all(proof(&endpoint.secret, role, &challenge).as_bytes())
        .await?;
    if stream.read_u8().await? != 1 {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    Ok(stream)
}

#[derive(Default)]
struct Owners {
    count: usize,
    foreground: usize,
    draining: bool,
    ever_attached: bool,
}

async fn serve(
    args: WebArgs,
    path: PathBuf,
    workspace: &Workspace,
    settings: Settings,
    launch_id: String,
) -> Result<(), String> {
    let listener = bind_local_listener(args.port).await?;
    let control = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|error| error.to_string())?;
    let endpoint = Endpoint {
        protocol: PROTOCOL,
        build: env!("CARGO_PKG_VERSION").into(),
        workspace: workspace.path().into(),
        settings,
        port: listener
            .local_addr()
            .map_err(|error| error.to_string())?
            .port(),
        control_port: control
            .local_addr()
            .map_err(|error| error.to_string())?
            .port(),
        secret: rand::random(),
        launch_id,
    };
    let service = AtlasAppService::start(AtlasAppServiceOptions {
        default_search_profile: args.search_profile,
        path_mode: crate::cli::runtime_path_mode(args.path_mode),
        source_root: args.source,
        embedding_cache_root: args.embedding_cache_path,
        index_path: Some(workspace.path().into()),
        retrieval_mode: AppServiceRetrievalMode::FullPool,
    })
    .map_err(|error| error.into_app_error().message)?;
    let model_ready = Arc::new(Notify::new());
    let warmup = service.clone();
    let warm_signal = model_ready.clone();
    let warm_task = tokio::spawn(async move {
        loop {
            warm_signal.notified().await;
            let service = warmup.clone();
            let _ = tokio::task::spawn_blocking(move || {
                match service.warm_default_search_profile() {
                    Ok(()) => tracing::info!(target: "atlas_cli", "Atlas search model is warm"),
                    Err(error) => tracing::info!(target: "atlas_cli", %error, "Atlas search model warmup skipped"),
                }
            }).await;
        }
    });
    let url = endpoint.url();
    let host = format!("127.0.0.1:{}", endpoint.port);
    let secret = endpoint.secret;
    let app = atlas_web::router(service).layer(axum::middleware::from_fn(
        move |request: axum::extract::Request, next: axum::middleware::Next| {
            let valid_host = request
                .headers()
                .get(axum::http::header::HOST)
                .is_some_and(|value| value.as_bytes() == host.as_bytes());
            let valid_origin = request
                .headers()
                .get(axum::http::header::ORIGIN)
                .is_none_or(|value| value.as_bytes() == url.as_bytes());
            let challenge = request
                .headers()
                .get("x-atlas-service-challenge")
                .filter(|value| {
                    matches!(
                        request.uri().path(),
                        "/api/compatibility" | "/api/search/page"
                    ) && value.as_bytes().len() == 32
                        && value.as_bytes().iter().all(u8::is_ascii_hexdigit)
                })
                .map(|value| proof(&secret, HTTP_PROOF, value.as_bytes()));
            async move {
                if valid_host && valid_origin {
                    let mut response = next.run(request).await;
                    if let Some(challenge) = challenge
                        && let Ok(value) = axum::http::HeaderValue::from_str(&challenge)
                    {
                        response
                            .headers_mut()
                            .insert("x-atlas-service-proof", value);
                    }
                    response
                } else {
                    axum::response::IntoResponse::into_response(axum::http::StatusCode::FORBIDDEN)
                }
            }
        },
    ));
    let mut file =
        tempfile::NamedTempFile::new_in(path.parent().ok_or("invalid discovery directory")?)
            .map_err(|error| error.to_string())?;
    serde_json::to_writer(&mut file, &endpoint).map_err(|error| error.to_string())?;
    file.persist(&path).map_err(|error| error.to_string())?;
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let mut server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
    });
    let control_result = own_service(control, endpoint.secret, model_ready).await;
    warm_task.abort();
    let _ = stop.send(());
    if timeout(DRAIN, &mut server).await.is_err() {
        server.abort();
    }
    let _ = fs::remove_file(path);
    control_result.map_err(|error| error.to_string())
}

async fn own_service(
    listener: TcpListener,
    secret: [u8; 32],
    model_ready: Arc<Notify>,
) -> io::Result<()> {
    let owners = Arc::new(Mutex::new(Owners::default()));
    let changed = Arc::new(Notify::new());
    let mut connections = JoinSet::new();
    let startup = sleep(STARTUP);
    tokio::pin!(startup);
    loop {
        {
            let mut state = owners
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.count == 0 && (state.ever_attached || startup.is_elapsed()) {
                state.draining = true;
                break;
            }
        }
        tokio::select! {
            _ = &mut startup, if !startup.is_elapsed() => {}
            _ = changed.notified() => {}
            _ = connections.join_next(), if !connections.is_empty() => {}
            result = listener.accept(), if connections.len() < 32 => {
                let (stream, _) = result?;
                connections.spawn(own_connection(stream, secret, owners.clone(), changed.clone(), model_ready.clone()));
            }
        }
    }
    Ok(())
}

async fn own_connection(
    mut stream: TcpStream,
    secret: [u8; 32],
    owners: Arc<Mutex<Owners>>,
    changed: Arc<Notify>,
    model_ready: Arc<Notify>,
) {
    let authenticated = timeout(Duration::from_secs(1), async {
        let mut client_challenge = [0; 32];
        stream.read_exact(&mut client_challenge).await?;
        stream
            .write_all(proof(&secret, CONTROL_PROOF, &client_challenge).as_bytes())
            .await?;
        let challenge: [u8; 32] = rand::random();
        stream.write_all(&challenge).await?;
        let role = stream.read_u8().await?;
        let mut response = [0; 64];
        stream.read_exact(&mut response).await?;
        Ok::<_, io::Error>(
            (matches!(role, FOREGROUND_LEASE | SEARCH_LEASE | SEMANTIC_PUBLICATION)
                && response == proof(&secret, role, &challenge).as_bytes())
            .then_some(role),
        )
    })
    .await;
    let Ok(Ok(Some(role))) = authenticated else {
        return;
    };
    if role == SEMANTIC_PUBLICATION {
        let active = {
            let state = owners
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.foreground > 0 && !state.draining
        };
        if active {
            model_ready.notify_one();
            let _ = stream.write_u8(1).await;
        }
        return;
    }
    {
        let mut state = owners
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.draining || (role == SEARCH_LEASE && state.foreground == 0) {
            return;
        }
        state.count += 1;
        if role == FOREGROUND_LEASE {
            if state.foreground == 0 {
                model_ready.notify_one();
            }
            state.foreground += 1;
            state.ever_attached = true;
        }
    }
    changed.notify_one();
    if stream.write_u8(1).await.is_ok() {
        if role == SEARCH_LEASE {
            let _ = timeout(DRAIN, stream.read_u8()).await;
        } else {
            let _ = stream.read_u8().await;
        }
    }
    {
        let mut state = owners
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.count -= 1;
        if role == FOREGROUND_LEASE {
            state.foreground -= 1;
        }
    }
    changed.notify_one();
}
