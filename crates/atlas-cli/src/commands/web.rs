use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::Command;

use atlas_app_service::{AtlasAppService, AtlasAppServiceOptions};
use tokio::net::TcpListener;
use tokio::runtime::Builder;

pub(crate) mod args;

use args::{DEFAULT_WEB_PORT, WebArgs};

pub(crate) fn run_web(args: WebArgs) -> Result<std::process::ExitCode, String> {
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to start async runtime: {error}"))?;

    runtime.block_on(run_web_async(args))
}

async fn run_web_async(args: WebArgs) -> Result<std::process::ExitCode, String> {
    let listener = bind_local_listener(args.port).await?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("failed to read web bind address: {error}"))?;
    let service = AtlasAppService::start(AtlasAppServiceOptions {
        path_mode: args.path_mode.into(),
        source_root: args.source,
        embedding_cache_root: args.embedding_cache_path,
        index_path: args.index,
    })
    .map_err(|error| {
        let app_error = error.into_app_error();
        format!(
            "atlas web startup failed: {}. Run `atlas setup` to prepare the full semantic-search runtime if setup is incomplete.",
            app_error.message
        )
    })?;
    let url = format!("http://{address}");

    println!("Atlas web service listening at {url}");
    if args.open {
        open_browser(&url);
    }

    axum::serve(listener, atlas_web::router(service))
        .await
        .map_err(|error| format!("atlas web server stopped with an error: {error}"))?;

    Ok(std::process::ExitCode::SUCCESS)
}

async fn bind_local_listener(port: Option<u16>) -> Result<TcpListener, String> {
    bind_local_listener_with(port, bind_port).await
}

async fn bind_local_listener_with<T, F, Fut>(port: Option<u16>, mut bind: F) -> Result<T, String>
where
    F: FnMut(u16) -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    if let Some(port) = port {
        return bind(port).await;
    }
    for port in DEFAULT_WEB_PORT..DEFAULT_WEB_PORT.saturating_add(100) {
        match bind(port).await {
            Ok(listener) => return Ok(listener),
            Err(_) => continue,
        }
    }
    Err(format!(
        "no available localhost port found in {}..{}",
        DEFAULT_WEB_PORT,
        DEFAULT_WEB_PORT.saturating_add(99)
    ))
}

async fn bind_port(port: u16) -> Result<TcpListener, String> {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    TcpListener::bind(address)
        .await
        .map_err(|error| format!("failed to bind {address}: {error}"))
}

fn open_browser(url: &str) {
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", url]).status()
    } else {
        Command::new("xdg-open").arg(url).status()
    };
    match result {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("warning: browser opener exited with status: {status}"),
        Err(error) => eprintln!("warning: failed to open browser: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn explicit_port_bind_reports_failure_when_port_is_occupied() {
        let port = DEFAULT_WEB_PORT + 20;

        let error = bind_local_listener_with(Some(port), |attempt| async move {
            Err::<u16, String>(format!("failed to bind simulated port {attempt}"))
        })
        .await
        .expect_err("explicit occupied port should fail");

        assert!(error.contains("failed to bind"));
        assert!(error.contains(&port.to_string()));
    }

    #[tokio::test]
    async fn default_port_binding_skips_occupied_default_port() {
        let selected = bind_local_listener_with(None, |attempt| async move {
            if attempt == DEFAULT_WEB_PORT {
                Err(format!("failed to bind simulated port {attempt}"))
            } else {
                Ok(attempt)
            }
        })
        .await
        .expect("auto binding should skip occupied default port");

        assert_ne!(selected, DEFAULT_WEB_PORT);
        assert!(selected > DEFAULT_WEB_PORT);
    }
}
