//! Release discovery and user-authorized installation belong to the desktop host.
use std::{
    sync::{Mutex, atomic::Ordering},
    time::Duration,
};

use atlas_desktop_protocol::workspace_selection::{
    SelectionSnapshot, TransitionFailure, TransitionFailureKind,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use minisign_verify::{PublicKey, Signature};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_updater::{Update, UpdaterExt};
use url::Url;

use crate::workspace_switch::{self, Admission, Build, Role, Transition, Waiting};

use crate::{Desktop, current_product, display, lock, opening, process, staged_setup};

const RELEASES: &str = "https://github.com/e-kosten/pf2e-atlas/releases";
const MANIFEST: &str = "atlas-desktop-update.json";

fn release_request_error(error: reqwest::Error) -> String {
    eprintln!("Atlas release request failed: {error}");
    if error.is_connect() || error.is_timeout() || error.is_body() {
        "Atlas couldn’t reach the update service. Check your connection before checking again."
            .into()
    } else {
        "Atlas couldn’t retrieve this release’s update information.".into()
    }
}

fn release_metadata_error(error: impl std::fmt::Display) -> String {
    eprintln!("Atlas release verification failed: {error}");
    "Atlas couldn’t verify this release’s update information. This release can’t be installed through Atlas.".into()
}

fn updater_check_error(error: tauri_plugin_updater::Error) -> String {
    match error {
        tauri_plugin_updater::Error::Reqwest(error) if error.is_decode() => {
            release_metadata_error(error)
        }
        tauri_plugin_updater::Error::Reqwest(error) => release_request_error(error),
        tauri_plugin_updater::Error::ReleaseNotFound => {
            "Atlas couldn’t retrieve this release’s update information.".into()
        }
        error => release_metadata_error(error),
    }
}

fn download_error(error: tauri_plugin_updater::Error) -> String {
    use tauri_plugin_updater::Error;
    eprintln!("Atlas update download failed: {error}");
    match error {
        Error::Reqwest(_) | Error::Network(_) => {
            "Atlas couldn’t download the update. Check your connection before trying again.".into()
        }
        Error::Minisign(_)
        | Error::Base64(_)
        | Error::SignatureUtf8(_)
        | Error::SignedVersionMismatch { .. }
        | Error::MissingSignedVersion => {
            "Atlas couldn’t verify the downloaded update. It won’t be installed.".into()
        }
        _ => "Atlas couldn’t prepare the downloaded update for installation.".into(),
    }
}
#[cfg(target_os = "macos")]
const PACKAGE: &str = "atlas-desktop-aarch64-apple-darwin.app.tar.gz";
#[cfg(windows)]
const PACKAGE: &str = "atlas-desktop-x86_64-pc-windows-msvc-setup.exe";

#[derive(Clone, PartialEq, Eq)]
enum Channel {
    Stable,
    Nightly { timestamp: String, commit: String },
}

impl Channel {
    fn from_version(version: &Version) -> Option<Self> {
        if !version.build.is_empty() {
            return None;
        }
        if version.pre.is_empty() {
            return Some(Self::Stable);
        }
        let mut fields = version.pre.as_str().split('.');
        if fields.next()? != "nightly" {
            return None;
        }
        let timestamp = fields.next()?;
        let commit = fields.next()?;
        if fields.next().is_some()
            || timestamp.len() != 14
            || !timestamp.bytes().all(|b| b.is_ascii_digit())
            || commit.len() != 13
            || !commit
                .strip_prefix('g')?
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return None;
        }
        Some(Self::Nightly {
            timestamp: timestamp.into(),
            commit: commit.into(),
        })
    }

    fn tag(&self, version: &Version) -> String {
        match self {
            Self::Stable => format!("release/v{version}"),
            Self::Nightly { timestamp, commit } => format!("nightly/{timestamp}.{commit}"),
        }
    }
}

fn newer(installed: &Version, candidate: &Version) -> bool {
    match (
        Channel::from_version(installed),
        Channel::from_version(candidate),
    ) {
        (Some(Channel::Stable), Some(Channel::Stable)) => candidate > installed,
        (
            Some(Channel::Nightly {
                timestamp: before, ..
            }),
            Some(Channel::Nightly {
                timestamp: after, ..
            }),
        ) => after > before,
        _ => false,
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Compatibility {
    workspace_schema: u32,
    foundry_commit: String,
    foundry_version: String,
}

#[derive(Clone)]
struct Offer {
    update: Update,
    compatibility: Compatibility,
    descriptor: Option<Descriptor>,
    switch: Option<Switch>,
}

#[derive(Clone)]
struct Switch {
    descriptor: Descriptor,
    admission: Admission,
    seed_source: Option<Admission>,
    config: workspace_switch::Config,
    returning_to_stable: bool,
}

impl Switch {
    fn validate_destination(&self) -> Result<(), String> {
        let initializing_pending = self.config.pending.as_ref().is_some_and(|pending| {
            pending.initialization_key.is_some()
                && pending.target == self.descriptor.build()
                && pending.admission == self.admission
                && pending.returning_to_stable == self.returning_to_stable
        });
        if self.admission.key.is_some() || self.seed_source.is_some() || initializing_pending {
            Ok(())
        } else {
            Err(
                "The selected data is missing. Choose an available copy before switching releases."
                    .into(),
            )
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Descriptor {
    protocol: u8,
    channel: String,
    version: String,
    tag: String,
    commit: String,
    asset: String,
    sha256: String,
    migration_ledger: serde_json::Value,
    #[serde(flatten)]
    compatibility: Compatibility,
}

impl Descriptor {
    fn admission_error(&self, error: TransitionFailure, data: &str) -> ChannelError {
        let incompatible = matches!(error.kind, TransitionFailureKind::NewerSchema);
        let message = process::transition_error(
            error,
            if self.channel == "stable" {
                "Stable"
            } else {
                "Nightly"
            },
            data,
        );
        if incompatible {
            ChannelError::Unavailable(message)
        } else {
            ChannelError::Failed(message)
        }
    }

    fn build(&self) -> Build {
        Build {
            version: self.version.clone(),
            tag: self.tag.clone(),
            commit: self.commit.clone(),
        }
    }

    fn verify_bytes(&self, bytes: &[u8]) -> Result<(), String> {
        if format!("{:x}", Sha256::digest(bytes)) != self.sha256 {
            return Err(
                "The verified update does not match the reviewed workspace compatibility.".into(),
            );
        }
        Ok(())
    }
}

pub(crate) fn installed_build() -> Option<Build> {
    let version = Version::parse(option_env!("ATLAS_DESKTOP_RELEASE")?).ok()?;
    Some(Build {
        tag: Channel::from_version(&version)?.tag(&version),
        version: version.to_string(),
        commit: option_env!("ATLAS_DESKTOP_SOURCE_COMMIT")?.to_owned(),
    })
}

fn decode_signature(text: &[u8]) -> Result<String, String> {
    let text = std::str::from_utf8(text).map_err(display)?;
    String::from_utf8(STANDARD.decode(text.trim()).map_err(display)?).map_err(display)
}

fn verify_descriptor(
    bytes: &[u8],
    signature: &[u8],
    key: &str,
    version: &Version,
    tag: &str,
) -> Result<Descriptor, String> {
    let key = PublicKey::decode(&decode_signature(key.as_bytes())?).map_err(display)?;
    let signature = Signature::decode(&decode_signature(signature)?).map_err(display)?;
    key.verify(bytes, &signature, true).map_err(display)?;
    let signed_version = format!("version:{version}");
    let descriptor: Descriptor = serde_json::from_slice(bytes).map_err(display)?;
    let nightly = tag.starts_with("nightly/");
    if !signature
        .trusted_comment()
        .split('\t')
        .any(|part| part == signed_version)
        || descriptor.protocol != 2
        || descriptor.version != version.to_string()
        || descriptor.tag != tag
        || descriptor.asset != PACKAGE
        || descriptor.channel != if nightly { "nightly" } else { "stable" }
        || !hex(&descriptor.commit, 40)
        || !hex(&descriptor.sha256, 64)
        || (nightly && !tag.ends_with(&format!(".g{}", &descriptor.commit[..12])))
        || descriptor.migration_ledger["entries"]
            .as_array()
            .and_then(|entries| entries.last())
            .and_then(|entry| entry["version"].as_u64())
            != Some(u64::from(descriptor.compatibility.workspace_schema))
    {
        return Err("The signed Atlas compatibility does not match the selected release.".into());
    }
    Ok(descriptor)
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

async fn bounded_download(
    client: &reqwest::Client,
    url: &str,
    maximum: usize,
) -> Result<Option<Vec<u8>>, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(release_request_error)?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let mut response = response.error_for_status().map_err(release_request_error)?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(release_request_error)? {
        if bytes.len() + chunk.len() > maximum {
            return Err(release_metadata_error(
                "Atlas update compatibility exceeded the size limit.",
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(Some(bytes))
}

async fn read_descriptor(
    source: &ReleaseSource,
    base: &str,
    key: &str,
    version: &Version,
    tag: &str,
) -> Result<Option<Descriptor>, String> {
    let client = source
        .client(
            reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::custom(release_redirect))
                .timeout(Duration::from_secs(10)),
        )
        .build()
        .map_err(display)?;
    let url = format!("{base}/{PACKAGE}.transition.json");
    let descriptor = bounded_download(&client, &url, 65_536).await?;
    let signature = bounded_download(&client, &format!("{url}.sig"), 4096).await?;
    match (descriptor, signature) {
        (None, None) => Ok(None),
        (Some(bytes), Some(signature)) => verify_descriptor(&bytes, &signature, key, version, tag)
            .map(Some)
            .map_err(release_metadata_error),
        _ => Err(release_metadata_error(
            "Atlas found an incomplete workspace compatibility signature.",
        )),
    }
}

#[derive(Default)]
pub(crate) struct Updates {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    offer: Option<Offer>,
    channel_offer: Option<Offer>,
    return_version: Option<String>,
    attempt: Option<Snapshot>,
}

impl State {
    fn active(&self) -> Option<&Snapshot> {
        self.attempt
            .as_ref()
            .filter(|attempt| attempt.phase != Phase::Available)
    }

    fn update_offer(&self) -> Option<Snapshot> {
        let version = offer_version(&self.offer)?;
        let error = self
            .attempt
            .as_ref()
            .filter(|attempt| !attempt.switching && attempt.version == version)
            .and_then(|attempt| attempt.error.clone());
        Some(Snapshot {
            version: version.into(),
            switching: false,
            phase: Phase::Available,
            error,
        })
    }
}

#[derive(Clone, Copy, Default, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Phase {
    #[default]
    Available,
    Downloading {
        percent: Option<u8>,
    },
    Installing,
}

#[derive(Clone, Serialize)]
pub(crate) struct Snapshot {
    version: String,
    switching: bool,
    phase: Phase,
    error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ChannelSnapshot {
    installed: Build,
    active: Role,
    workspace_path: Option<std::path::PathBuf>,
    other_exists: bool,
    returning: Option<workspace_switch::ReturnChoice>,
    ready: bool,
    pending_version: Option<String>,
    busy: bool,
    error: Option<String>,
}

#[derive(Default)]
pub(crate) struct BroadcastUpdates {
    pub update: Option<Snapshot>,
    pub channel: Option<ChannelSnapshot>,
}

pub(crate) fn broadcast_updates(
    app: &AppHandle,
    read_selection: impl FnOnce() -> Result<SelectionSnapshot, String>,
) -> BroadcastUpdates {
    let desktop = app.state::<Desktop>();
    let selection = desktop
        .workspace_route
        .index()
        .ok()
        .flatten()
        .and_then(|pinned| read_selection().ok().map(|selection| (pinned, selection)));
    let state = lock(&desktop.updates.state);
    let Some((pinned, selection)) = selection else {
        return BroadcastUpdates {
            update: state.active().cloned(),
            channel: None,
        };
    };
    selection_views(
        &state,
        pinned,
        selection,
        installed_build(),
        state.update_offer(),
    )
}

fn selection_views(
    state: &State,
    pinned: std::path::PathBuf,
    selection: SelectionSnapshot,
    installed: Option<Build>,
    offer: Option<Snapshot>,
) -> BroadcastUpdates {
    let Ok(active) = workspace_switch::running_role(&pinned, &selection) else {
        return BroadcastUpdates {
            update: state.active().cloned(),
            channel: None,
        };
    };
    let config = selection.config;
    let changed = active != config.active;
    let update = state.active().cloned().or_else(|| {
        offer.filter(|_| {
            !changed
                && !config
                    .returning
                    .as_ref()
                    .is_some_and(|choice| choice.updates == Waiting::Hold)
        })
    });
    let channel = installed.map(|installed| ChannelSnapshot {
        active,
        workspace_path: Some(pinned),
        other_exists: selection
            .workspaces
            .iter()
            .any(|workspace| workspace.role == active.other() && workspace.exists),
        returning: config.returning,
        ready: state.return_version.is_some(),
        pending_version: config
            .pending
            .filter(|pending| pending.previous_resumed && pending.before == installed)
            .map(|pending| pending.target.version),
        installed,
        busy: changed || state.active().is_some(),
        error: if changed {
            Some(workspace_switch::CHANGED_WORKSPACE.into())
        } else {
            state
                .attempt
                .as_ref()
                .filter(|attempt| attempt.switching)
                .and_then(|attempt| attempt.error.clone())
        },
    });
    BroadcastUpdates { update, channel }
}

#[derive(Clone)]
struct ReleaseSource {
    root: &'static str,
    rehearsal_ca: Option<reqwest::Certificate>,
}

impl ReleaseSource {
    fn bundled() -> Result<Self, String> {
        let Some(root) = option_env!("ATLAS_UPDATER_REHEARSAL_ROOT") else {
            return Ok(Self {
                root: RELEASES,
                rehearsal_ca: None,
            });
        };
        let url = Url::parse(root).map_err(display)?;
        if url.scheme() != "https"
            || url.host_str() != Some("127.0.0.1")
            || url.port().is_none()
            || url.path() != "/releases"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("The rehearsal feed must use HTTPS on the local loopback address.".into());
        }
        let pem = option_env!("ATLAS_UPDATER_REHEARSAL_CA")
            .ok_or("The rehearsal feed requires a disposable TLS certificate.")?;
        let certificate = reqwest::Certificate::from_pem(pem.as_bytes()).map_err(display)?;
        Ok(Self {
            root,
            rehearsal_ca: Some(certificate),
        })
    }

    fn client(&self, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
        if let Some(certificate) = &self.rehearsal_ca {
            builder
                .tls_certs_only([certificate.clone()])
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
        } else {
            builder.redirect(reqwest::redirect::Policy::custom(release_redirect))
        }
    }
}

pub(crate) fn start(app: &AppHandle) {
    let Some(installed) =
        option_env!("ATLAS_DESKTOP_RELEASE").and_then(|value| Version::parse(value).ok())
    else {
        return;
    };
    let Some(channel) = Channel::from_version(&installed) else {
        return;
    };
    let Some(key) = option_env!("ATLAS_UPDATER_PUBLIC_KEY").filter(|key| !key.is_empty()) else {
        return;
    };
    let source = match ReleaseSource::bundled() {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Could not configure Atlas update feed: {error}");
            return;
        }
    };
    let _ = rustls::crypto::ring::default_provider().install_default();
    let app = app.clone();
    std::thread::spawn(move || {
        let mut first = true;
        loop {
            if !first {
                std::thread::sleep(Duration::from_secs(4 * 60 * 60));
            }
            first = false;
            if app.state::<Desktop>().quitting.load(Ordering::Acquire) {
                return;
            }
            match tauri::async_runtime::block_on(poll(&app, &installed, &channel, key, &source)) {
                Ok((normal, returning)) => {
                    let desktop = app.state::<Desktop>();
                    let mut state = lock(&desktop.updates.state);
                    if state.active().is_none() {
                        let changed = offer_version(&state.offer) != offer_version(&normal)
                            || state.return_version.as_deref() != offer_version(&returning);
                        state.offer = normal;
                        state.return_version = offer_version(&returning).map(str::to_owned);
                        if changed {
                            drop(state);
                            staged_setup::broadcast(&app);
                        }
                    }
                }
                Err(error) => eprintln!("Atlas update check failed: {error}"),
            }
        }
    });
}

fn offer_version(offer: &Option<Offer>) -> Option<&str> {
    offer.as_ref().map(|entry| entry.update.version.as_str())
}

async fn poll(
    app: &AppHandle,
    installed: &Version,
    channel: &Channel,
    key: &str,
    source: &ReleaseSource,
) -> Result<(Option<Offer>, Option<Offer>), String> {
    let config = workspace_switch::load_current(app)?;
    if config.pending.is_some() {
        return Ok((None, None));
    }
    let normal = if config
        .returning
        .as_ref()
        .is_some_and(|choice| choice.updates == Waiting::Hold)
    {
        None
    } else {
        let prove = config.active != Role::Original || config.returning.is_some();
        match check(app, installed, channel, key, source, false, prove).await {
            Ok(Some(offer)) if offer.descriptor.is_some() => {
                match authorize(offer, config.clone(), config.active, false, false).await {
                    Ok(offer) => Some(offer),
                    Err(error) => {
                        eprintln!(
                            "Atlas could not verify the workspace for its next update: {error:?}"
                        );
                        None
                    }
                }
            }
            Ok(offer) => offer,
            Err(error) => {
                eprintln!("Atlas could not verify its next update: {error}");
                None
            }
        }
    };
    let returning = if let (Channel::Nightly { .. }, Some(choice)) = (channel, &config.returning) {
        match check(app, installed, &Channel::Stable, key, source, true, true).await {
            Ok(Some(offer)) => authorize(offer, config.clone(), choice.workspace, true, false)
                .await
                .ok(),
            _ => None,
        }
    } else {
        None
    };
    // A choice can change while the feed is being fetched.
    if workspace_switch::load_current(app)? != config {
        return Err("The update preference changed.".into());
    }
    Ok((normal, returning))
}

fn release_redirect(attempt: reqwest::redirect::Attempt<'_>) -> reqwest::redirect::Action {
    let destination = attempt.url();
    if attempt.previous().len() < 5
        && destination.scheme() == "https"
        && destination.port().is_none()
        && destination.username().is_empty()
        && destination.password().is_none()
        && matches!(
            destination.host_str(),
            Some("github.com" | "release-assets.githubusercontent.com")
        )
    {
        attempt.follow()
    } else {
        attempt.error("Untrusted Atlas release redirect")
    }
}

async fn release_tag(channel: &Channel, source: &ReleaseSource) -> Result<Option<String>, String> {
    let root = source.root;
    let builder = source.client(reqwest::Client::builder().timeout(Duration::from_secs(10)));
    match channel {
        Channel::Stable => {
            let response = builder
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(release_metadata_error)?
                .head(format!("{root}/latest"))
                .send()
                .await
                .map_err(release_request_error)?;
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            let response = response.error_for_status().map_err(release_request_error)?;
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    release_metadata_error("Stable release did not resolve to an Atlas tag.")
                })?;
            if location == root {
                return Ok(None);
            }
            let tag = location
                .strip_prefix(&format!("{root}/tag/"))
                .ok_or_else(|| {
                    release_metadata_error("Stable release resolved outside Atlas releases.")
                })?;
            let version = Version::parse(
                tag.strip_prefix("release/v")
                    .ok_or_else(|| release_metadata_error("Invalid stable tag."))?,
            )
            .map_err(release_metadata_error)?;
            if Channel::from_version(&version) != Some(Channel::Stable)
                || Channel::Stable.tag(&version) != tag
            {
                return Err(release_metadata_error("Invalid stable tag."));
            }
            Ok(Some(tag.into()))
        }
        Channel::Nightly { .. } => {
            let response = builder
                .build()
                .map_err(release_metadata_error)?
                .get(format!(
                    "{root}/download/nightly/latest/nightly-version.txt"
                ))
                .send()
                .await
                .map_err(release_request_error)?;
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            let mut response = response.error_for_status().map_err(release_request_error)?;
            let mut pointer = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(release_request_error)? {
                if pointer.len() + chunk.len() > 64 {
                    return Err(release_metadata_error("Invalid nightly release pointer."));
                }
                pointer.extend_from_slice(&chunk);
            }
            let tag = std::str::from_utf8(&pointer)
                .map_err(release_metadata_error)?
                .trim();
            let identity = tag
                .strip_prefix("nightly/")
                .ok_or_else(|| release_metadata_error("Invalid nightly release pointer."))?;
            let version = Version::parse(&format!("0.0.0-nightly.{identity}"))
                .map_err(release_metadata_error)?;
            if !matches!(Channel::from_version(&version), Some(channel @ Channel::Nightly { .. }) if channel.tag(&version) == tag)
            {
                return Err(release_metadata_error("Invalid nightly release pointer."));
            }
            Ok(Some(tag.into()))
        }
    }
}

async fn check(
    app: &AppHandle,
    installed: &Version,
    channel: &Channel,
    key: &str,
    source: &ReleaseSource,
    switching: bool,
    require_descriptor: bool,
) -> Result<Option<Offer>, String> {
    let Some(tag) = release_tag(channel, source).await? else {
        return Ok(None);
    };
    if tag == channel.tag(installed) {
        return Ok(None);
    }
    check_tag(
        app,
        installed,
        tag,
        key,
        source,
        switching,
        require_descriptor,
    )
    .await
}

async fn check_tag(
    app: &AppHandle,
    installed: &Version,
    tag: String,
    key: &str,
    source: &ReleaseSource,
    switching: bool,
    require_descriptor: bool,
) -> Result<Option<Offer>, String> {
    let base = format!("{}/download/{tag}", source.root);
    let http_source = source.clone();
    let current = installed.clone();
    let expected = tag.clone();
    let updater = app
        .updater_builder()
        .pubkey(key)
        .timeout(Duration::from_secs(15))
        .endpoints(vec![
            Url::parse(&format!("{base}/{MANIFEST}")).map_err(display)?,
        ])
        .map_err(display)?
        .configure_client(move |builder| http_source.client(builder))
        .version_comparator(move |_, release| {
            if switching {
                Channel::from_version(&release.version)
                    .is_some_and(|channel| channel.tag(&release.version) == expected)
            } else {
                newer(&current, &release.version)
            }
        })
        .build()
        .map_err(display)?;
    let found = match updater.check().await {
        Ok(found) => found,
        Err(error) => return Err(updater_check_error(error)),
    };
    let Some(found) = found else {
        if switching {
            return Err(release_metadata_error(
                "No matching update for the selected release tag.",
            ));
        }
        return Ok(None);
    };
    let version = Version::parse(&found.version).map_err(display)?;
    if Channel::from_version(&version).is_none_or(|channel| channel.tag(&version) != tag)
        || found.download_url.as_str() != format!("{base}/{PACKAGE}")
    {
        return Err(release_metadata_error(
            "Update metadata does not match the Atlas release.",
        ));
    }
    let compatibility = serde_json::from_value(found.raw_json["atlasCompatibility"].clone())
        .map_err(release_metadata_error)?;
    // Restoration guarantees that install and download require a signature bound to
    // the full Atlas version, including the nightly timestamp and commit.
    let mut update = updater
        .restore_update(found.raw_json)
        .map_err(updater_check_error)?
        .ok_or("The Atlas update version changed.")?;
    update.timeout = Some(Duration::from_secs(15 * 60));
    let descriptor = read_descriptor(source, &base, key, &version, &tag).await?;
    if require_descriptor && descriptor.is_none() {
        return Err("This release doesn’t support compatibility checks. Atlas can’t switch to it automatically.".into());
    }
    Ok(Some(Offer {
        update,
        compatibility: descriptor
            .as_ref()
            .map_or(compatibility, |value| value.compatibility.clone()),
        descriptor,
        switch: None,
    }))
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum UpdateImpact {
    Restart,
    Upgrade,
    Reference { version: String },
    Unknown,
}

fn setup_impact(
    target: &Compatibility,
    installed_schema: Option<u32>,
    read_reference: impl FnOnce() -> Option<atlas_desktop_protocol::ReferenceStatus>,
) -> UpdateImpact {
    let Some(installed_schema) = installed_schema else {
        return UpdateImpact::Unknown;
    };
    if target.workspace_schema > installed_schema {
        return UpdateImpact::Upgrade;
    }
    let Some(report) = read_reference() else {
        return UpdateImpact::Unknown;
    };
    if report.managed {
        match report.revision.as_deref() {
            None => return UpdateImpact::Unknown,
            Some(revision) if revision != target.foundry_commit => {
                return UpdateImpact::Reference {
                    version: target.foundry_version.clone(),
                };
            }
            _ => {}
        }
    }
    UpdateImpact::Restart
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Action {
    Details,
    Install,
}

#[tauri::command]
pub(crate) async fn desktop_update_action(
    window: WebviewWindow,
    token: String,
    version: String,
    action: Action,
) -> Result<Option<UpdateImpact>, String> {
    let app = window.app_handle().clone();
    let desktop = app.state::<Desktop>();
    let (_, instance) = current_product(&app)?;
    if !desktop
        .staged_setup
        .authorize_document(&window, &instance, &token)
    {
        return Err("Open the update again in the current Atlas window.".into());
    }
    let workspace = lock(&desktop.launcher)
        .as_ref()
        .filter(|launcher| launcher.ready.instance == instance)
        .map(|launcher| launcher.ready.workspace.clone())
        .ok_or("The Atlas service stopped.")?;
    let offer = lock(&desktop.updates.state)
        .offer
        .as_ref()
        .filter(|offer| offer.update.version == version)
        .cloned()
        .ok_or("A different update is available. Open Update again.")?;
    match action {
        Action::Details => tauri::async_runtime::spawn_blocking(move || {
            let installed_schema =
                option_env!("ATLAS_WORKSPACE_SCHEMA").and_then(|schema| schema.parse::<u32>().ok());
            Some(setup_impact(&offer.compatibility, installed_schema, || {
                process::reference_status(&app)
                    .ok()
                    .filter(|report| report.workspace == workspace)
            }))
        })
        .await
        .map_err(display),
        Action::Install => {
            begin_install(&app, offer, instance, false)?;
            Ok(None)
        }
    }
}

fn retries_pending(offer: &Offer, pending: &Transition) -> bool {
    pending.previous_resumed
        && installed_build().as_ref() == Some(&pending.before)
        && offer.switch.as_ref().is_some_and(|switch| {
            switch.descriptor.build() == pending.target
                && switch.admission == pending.admission
                && switch.seed_source.is_none()
        })
}

fn begin_install(
    app: &AppHandle,
    offer: Offer,
    instance: String,
    switching: bool,
) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    {
        let _operation = desktop
            .operation
            .try_lock()
            .map_err(|_| "Wait for Atlas setup to finish and try again.")?;
        let mut state = lock(&desktop.updates.state);
        if state.active().is_some() {
            return Err("Atlas is already installing an update.".into());
        }
        let current = if switching {
            &state.channel_offer
        } else {
            &state.offer
        };
        if offer_version(current) != Some(&offer.update.version) {
            return Err("A different update is available. Review the update again.".into());
        }
        if desktop.staged_setup.active() || lock(&desktop.ordinary_setup).is_some() {
            return Err("Wait for Atlas setup to finish and try again.".into());
        }
        let config = workspace_switch::load_current(app)?;
        if config
            .pending
            .as_ref()
            .is_some_and(|pending| !retries_pending(&offer, pending))
            || (offer.switch.is_none()
                && (config.active != Role::Original || config.returning.is_some()))
        {
            return Err(
                "The workspace or update preference changed. Review the update again.".into(),
            );
        }
        state.attempt = Some(Snapshot {
            version: offer.update.version.clone(),
            switching,
            phase: Phase::Downloading { percent: None },
            error: None,
        });
    }
    staged_setup::broadcast(app);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = download_and_install(&app, offer, instance).await {
            let desktop = app.state::<Desktop>();
            let mut state = lock(&desktop.updates.state);
            if let Some(attempt) = &mut state.attempt {
                attempt.phase = Phase::Available;
                attempt.error = Some(error);
            }
            drop(state);
            staged_setup::broadcast(&app);
        }
    });
    Ok(())
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub(crate) enum ChannelError {
    Unavailable(String),
    Failed(String),
}

impl From<String> for ChannelError {
    fn from(message: String) -> Self {
        Self::Failed(message)
    }
}

impl From<&str> for ChannelError {
    fn from(message: &str) -> Self {
        Self::Failed(message.into())
    }
}

async fn authorize(
    mut offer: Offer,
    config: workspace_switch::Config,
    role: Role,
    returning_to_stable: bool,
    seed_if_missing: bool,
) -> Result<Offer, ChannelError> {
    let descriptor = offer
        .descriptor
        .clone()
        .ok_or("This build has no signed workspace compatibility information.")?;
    let transition = tauri::async_runtime::spawn_blocking(move || {
        let data = if role == config.active {
            "your current data"
        } else {
            "this earlier copy of your data"
        };
        let admission = process::inspect_transition(role, &descriptor.migration_ledger, None)
            .map_err(|error| descriptor.admission_error(error, data))?;
        let seed_source = if seed_if_missing && role != config.active && admission.key.is_none() {
            let source =
                process::inspect_transition(config.active, &descriptor.migration_ledger, None)
                    .map_err(|error| descriptor.admission_error(error, "your current data"))?;
            if source.key.is_none() {
                return Err("Set up your current workspace before making a separate copy.".into());
            }
            Some(source)
        } else {
            None
        };
        Ok::<_, ChannelError>(Switch {
            descriptor,
            admission,
            seed_source,
            config,
            returning_to_stable,
        })
    })
    .await
    .map_err(display)??;
    transition.validate_destination()?;
    offer.switch = Some(transition);
    Ok(offer)
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ChannelAction {
    Review {
        workspace: Role,
        #[serde(default)]
        seed_if_missing: bool,
    },
    Wait {
        workspace: Role,
        updates: Waiting,
    },
    CancelWait,
    RetryPending,
    Install {
        token: String,
    },
}

#[derive(Serialize)]
pub(crate) struct ChannelReview {
    id: String,
    version: String,
    workspace: Role,
    will_seed: bool,
    needs_setup: bool,
    will_upgrade: bool,
}

impl ChannelReview {
    fn from_switch(transition: &Switch) -> Result<Self, String> {
        Ok(Self {
            id: review_id(transition)?,
            version: transition.descriptor.version.clone(),
            workspace: transition.admission.role,
            will_seed: transition.seed_source.is_some(),
            needs_setup: transition.admission.key.is_none() && transition.seed_source.is_none(),
            will_upgrade: transition
                .seed_source
                .as_ref()
                .unwrap_or(&transition.admission)
                .schema
                .is_some_and(|schema| {
                    schema < transition.descriptor.compatibility.workspace_schema
                }),
        })
    }
}

fn review_id(transition: &Switch) -> Result<String, String> {
    let payload = serde_json::to_vec(&(
        transition.descriptor.build(),
        &transition.admission,
        &transition.seed_source,
        &transition.config,
        &transition.descriptor.sha256,
    ))
    .map_err(display)?;
    Ok(format!("{:x}", Sha256::digest(payload)))
}

#[tauri::command]
pub(crate) async fn desktop_channel_action(
    window: WebviewWindow,
    token: String,
    action: ChannelAction,
) -> Result<Option<ChannelReview>, ChannelError> {
    let app = window.app_handle().clone();
    let desktop = app.state::<Desktop>();
    let (_, instance) = current_product(&app)?;
    if !desktop
        .staged_setup
        .authorize_document(&window, &instance, &token)
    {
        return Err("Open Updates again in the current Atlas window.".into());
    }
    let installed = installed_build().ok_or("This build does not support channel switching.")?;
    let installed_version = Version::parse(&installed.version).map_err(display)?;
    let nightly = installed.tag.starts_with("nightly/");
    let mut config = workspace_switch::load_current(&app)?;
    if lock(&desktop.updates.state).active().is_some() {
        return Err("Atlas is already installing an update.".into());
    }
    if let Some(pending) = &config.pending {
        if !matches!(action, ChannelAction::RetryPending)
            || !pending.previous_resumed
            || pending.before != installed
        {
            return Err("Finish the pending Atlas installation before changing channels.".into());
        }
        let version = Version::parse(&pending.target.version).map_err(display)?;
        if Channel::from_version(&version)
            .is_none_or(|channel| channel.tag(&version) != pending.target.tag)
        {
            return Err("The pending release identity is invalid.".into());
        }
        let key = option_env!("ATLAS_UPDATER_PUBLIC_KEY")
            .ok_or("This build does not support signed updates.")?;
        let source = ReleaseSource::bundled()?;
        let offer = check_tag(
            &app,
            &installed_version,
            pending.target.tag.clone(),
            key,
            &source,
            true,
            true,
        )
        .await?
        .ok_or("The pending Atlas release is unavailable.")?;
        let offer = authorize(
            offer,
            config.clone(),
            pending.admission.role,
            pending.returning_to_stable,
            false,
        )
        .await?;
        if workspace_switch::load_current(&app)? != config
            || !retries_pending(&offer, pending)
            || current_product(&app)?.1 != instance
        {
            return Err("The previously confirmed release or workspace changed. Atlas kept both workspaces.".into());
        }
        lock(&desktop.updates.state).channel_offer = Some(offer.clone());
        begin_install(&app, offer, instance, true)?;
        return Ok(None);
    }
    match action {
        ChannelAction::Review {
            workspace,
            seed_if_missing,
        } => {
            let key = option_env!("ATLAS_UPDATER_PUBLIC_KEY")
                .filter(|key| !key.is_empty())
                .ok_or("This build does not support signed updates.")?;
            let source = ReleaseSource::bundled()?;
            // Only the channel kind matters for lookup. The release pointer supplies the actual identity.
            let target = if nightly {
                Channel::Stable
            } else {
                Channel::Nightly {
                    timestamp: String::new(),
                    commit: String::new(),
                }
            };
            let offer = check(&app, &installed_version, &target, key, &source, true, true)
                .await?
                .ok_or_else(|| {
                    if nightly {
                        ChannelError::Unavailable(
                            "There is no Stable release available to review.".into(),
                        )
                    } else {
                        ChannelError::Failed(
                            "There is no Nightly release available to review.".into(),
                        )
                    }
                })?;
            let offer = authorize(
                offer,
                config.clone(),
                workspace,
                nightly,
                seed_if_missing && !nightly,
            )
            .await?;
            if workspace_switch::load_current(&app)? != config
                || current_product(&app)?.1 != instance
            {
                return Err(
                    "The workspace or update preference changed. Review the switch again.".into(),
                );
            }
            let transition = offer
                .switch
                .as_ref()
                .ok_or("Missing reviewed channel switch.")?;
            let review = ChannelReview::from_switch(transition)?;
            let mut state = lock(&desktop.updates.state);
            if state.active().is_some() {
                return Err("Atlas is already installing an update.".into());
            }
            state.channel_offer = Some(offer);
            if state
                .attempt
                .as_ref()
                .is_some_and(|attempt| attempt.switching)
            {
                state.attempt = None;
                drop(state);
                staged_setup::broadcast(&app);
            }
            Ok(Some(review))
        }
        ChannelAction::Install { token } => {
            let offer = lock(&desktop.updates.state)
                .channel_offer
                .clone()
                .ok_or("Review the channel switch again.")?;
            let transition = offer
                .switch
                .as_ref()
                .ok_or("Review the channel switch again.")?;
            if config != transition.config || token != review_id(transition)? {
                return Err(
                    "The reviewed channel or workspace changed. Review the switch again.".into(),
                );
            }
            begin_install(&app, offer, instance, true)?;
            Ok(None)
        }
        ChannelAction::Wait { workspace, updates } if nightly => {
            {
                let _operation = desktop
                    .operation
                    .try_lock()
                    .map_err(|_| "Atlas is already working.")?;
                let mut current = lock(&desktop.updates.state);
                if current.active().is_some() {
                    return Err("Atlas is already installing an update.".into());
                }
                config.returning = Some(workspace_switch::ReturnChoice { workspace, updates });
                workspace_switch::save(&app, &mut config)?;
                current.offer = None;
                current.channel_offer = None;
                current.return_version = None;
                current.attempt = None;
            }
            refresh(&app);
            Ok(None)
        }
        ChannelAction::CancelWait if nightly => {
            {
                let _operation = desktop
                    .operation
                    .try_lock()
                    .map_err(|_| "Atlas is already working.")?;
                let mut current = lock(&desktop.updates.state);
                if current.active().is_some() {
                    return Err("Atlas is already installing an update.".into());
                }
                config.returning = None;
                workspace_switch::save(&app, &mut config)?;
                current.channel_offer = None;
                current.return_version = None;
                current.attempt = None;
            }
            refresh(&app);
            Ok(None)
        }
        _ => Err("This update choice is only available on Nightly.".into()),
    }
}

pub(crate) fn refresh(app: &AppHandle) {
    staged_setup::broadcast(app);
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let Some(build) = installed_build() else {
            return;
        };
        let Ok(version) = Version::parse(&build.version) else {
            return;
        };
        let Some(channel) = Channel::from_version(&version) else {
            return;
        };
        let Some(key) = option_env!("ATLAS_UPDATER_PUBLIC_KEY") else {
            return;
        };
        let Ok(source) = ReleaseSource::bundled() else {
            return;
        };
        if let Ok((normal, returning)) = poll(&app, &version, &channel, key, &source).await {
            let desktop = app.state::<Desktop>();
            let mut state = lock(&desktop.updates.state);
            if state.active().is_none() {
                state.offer = normal;
                state.return_version = offer_version(&returning).map(str::to_owned);
                drop(state);
                staged_setup::broadcast(&app);
            }
        }
    });
}

async fn download_and_install(
    app: &AppHandle,
    offer: Offer,
    instance: String,
) -> Result<(), String> {
    let mut downloaded = 0u64;
    let bytes = offer
        .update
        .download(
            |count, total| {
                downloaded += count as u64;
                let percent = total
                    .filter(|size| *size > 0)
                    .map(|size| ((downloaded.saturating_mul(100) / size).min(100)) as u8);
                let desktop = app.state::<Desktop>();
                let mut state = lock(&desktop.updates.state);
                let phase = Phase::Downloading { percent };
                if let Some(attempt) = &mut state.attempt
                    && attempt.phase != phase
                {
                    attempt.phase = phase;
                    drop(state);
                    staged_setup::broadcast(app);
                }
            },
            || {},
        )
        .await
        .map_err(download_error)?;
    if let Some(transition) = &offer.switch {
        transition.descriptor.verify_bytes(&bytes)?;
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || install(&app, &offer, &bytes, &instance))
        .await
        .map_err(display)?
}

fn install(app: &AppHandle, offer: &Offer, bytes: &[u8], instance: &str) -> Result<(), String> {
    let desktop = app.state::<Desktop>();
    let _operation = desktop
        .operation
        .try_lock()
        .map_err(|_| "Wait for Atlas setup to finish and try again.")?;
    if desktop.staged_setup.active() || lock(&desktop.ordinary_setup).is_some() {
        return Err("Wait for Atlas setup to finish and try again.".into());
    }
    let (old_base, current_instance) = current_product(app)?;
    if instance != current_instance {
        return Err("The workspace connection changed. Try the update again.".into());
    }
    let mut config = workspace_switch::load_current(app)?;
    let retry = config.pending.is_some();
    if config
        .pending
        .as_ref()
        .is_some_and(|pending| !retries_pending(offer, pending))
    {
        return Err("A different Atlas install is already pending.".into());
    }
    #[cfg(windows)]
    let windows = {
        let (launcher, worker) = lock(&desktop.launcher)
            .as_ref()
            .map(|launcher| (launcher.pid, launcher.ready.worker_pid))
            .ok_or("The Atlas service stopped.")?;
        crate::window_host::check_cli_sessions(launcher, worker)?;
        crate::windows_tabs::WindowHandoff::for_update(app)?
    };
    let reservation = if let Some(transition) = &offer.switch {
        if transition.config != config {
            return Err(
                "The workspace or update preference changed. Review the update again.".into(),
            );
        }
        let role = transition.admission.role;
        let ledger = &transition.descriptor.migration_ledger;
        let expected = if let Some(seed) = &transition.seed_source {
            if process::inspect_transition(role, ledger, None)
                .map_err(|error| process::transition_error(error, "Atlas", "the selected data"))?
                != transition.admission
            {
                return Err("The separate workspace appeared. Review the switch again.".into());
            }
            process::inspect_transition(role, ledger, Some(seed))
                .map_err(|error| process::transition_error(error, "Atlas", "the selected data"))?
        } else {
            transition.admission.clone()
        };
        let reservation = process::reserve_transition(role, ledger)?;
        if reservation.admission != expected {
            return Err("The destination workspace changed. Review the update again.".into());
        }
        if !retry {
            config.pending = Some(workspace_switch::new_transition(
                installed_build().ok_or("Cannot identify this Atlas build.")?,
                transition.descriptor.build(),
                expected,
                transition.returning_to_stable,
            )?);
        }
        Some(reservation)
    } else {
        if config.active != Role::Original || config.returning.is_some() {
            return Err(
                "The workspace or update preference changed. Review the update again.".into(),
            );
        }
        None
    };
    #[cfg(windows)]
    windows.validate()?;
    let launcher = {
        let mut slot = lock(&desktop.launcher);
        if slot
            .as_ref()
            .is_none_or(|launcher| launcher.ready.instance != current_instance)
        {
            return Err("The workspace connection changed. Try the update again.".into());
        }
        // Keep an unexpected service exit from separating the pending switch
        // from the handoff that owns it.
        if !retry && config.pending.is_some() {
            workspace_switch::save(app, &mut config)?;
        }
        slot.take().ok_or("The Atlas service stopped.")?
    };
    if let Some(attempt) = &mut lock(&desktop.updates.state).attempt {
        attempt.phase = Phase::Installing;
    }
    staged_setup::broadcast(app);
    let install = launcher.release().and_then(|()| {
        #[cfg(windows)]
        {
            drop(reservation);
            crate::window_host::wait_for_cli_release()?;
        }
        offer.update.install(bytes).map_err(|error| {
            eprintln!("Could not install Atlas update: {error}");
            if matches!(error, tauri_plugin_updater::Error::Io(ref io)
                if io.kind() == std::io::ErrorKind::PermissionDenied)
            {
                "Atlas couldn’t write to its installation folder. Check the folder permissions and try again.".to_owned()
            } else {
                "Atlas couldn’t install the update. Try again.".to_owned()
            }
        })
    });
    #[cfg(target_os = "macos")]
    drop(reservation);
    if let Err(error) = install {
        if !retry
            && config.pending.take().is_some()
            && let Err(error) = workspace_switch::save(app, &mut config)
        {
            eprintln!("Atlas could not clear its interrupted transition: {error}");
        }
        #[cfg(windows)]
        drop(windows);
        recover(app, old_base, &current_instance);
        return Err(error);
    }
    #[cfg(target_os = "macos")]
    app.restart();
    #[cfg(windows)]
    Ok(())
}

fn recover(app: &AppHandle, old_base: Url, old_instance: &str) {
    let recovery = crate::start_launcher(app).and_then(|()| current_product(app));
    if recovery
        .as_ref()
        .is_ok_and(|(base, instance)| base == &old_base && instance == old_instance)
    {
        // An independent lease kept the same service and its documents alive.
        return;
    }
    app.state::<Desktop>().staged_setup.clear_documents();
    let restore = move |app: &AppHandle| {
        let desktop = app.state::<Desktop>();
        let views = app
            .webview_windows()
            .into_values()
            .filter_map(|window| {
                let url = window.url().ok()?;
                if url.origin() != old_base.origin() {
                    return None;
                }
                let route = &url[url::Position::BeforePath..];
                Some((window, route.to_owned()))
            })
            .collect::<Vec<_>>();
        match recovery {
            Ok((base, instance)) => {
                for (window, route) in views {
                    #[cfg(target_os = "macos")]
                    if window.label() == crate::settings::WINDOW {
                        if let Err(error) =
                            crate::settings::recover_window(app, &window, &base, &instance)
                        {
                            let _ = window.destroy();
                            crate::window_host::report("Settings couldn’t reopen", &error);
                        }
                        continue;
                    }
                    if base.origin() == old_base.origin() || window.label() == opening::PRIMARY {
                        if let Ok(target) = base.join(&route)
                            && target.origin() == base.origin()
                            && desktop
                                .staged_setup
                                .attach_window(app, window.label(), &base, &instance)
                                .is_ok()
                        {
                            if window.label() == opening::PRIMARY {
                                *lock(&desktop.primary) =
                                    opening::PrimaryWindow::Product(base.origin());
                            }
                            let _ = window.navigate(target);
                        }
                    } else {
                        #[cfg(target_os = "macos")]
                        let replacement = crate::open_product(app, &route, None);
                        #[cfg(windows)]
                        let replacement = crate::open_product_on_worker(app, &route, &instance);
                        match replacement {
                            Ok(_) => {
                                let _ = crate::destroy_window(&window);
                            }
                            #[cfg(windows)]
                            Err(error) => {
                                crate::window_host::report("Atlas couldn’t reopen a window", &error)
                            }
                            #[cfg(target_os = "macos")]
                            Err(_) => {}
                        }
                    }
                }
            }
            Err(error) => {
                #[cfg(target_os = "macos")]
                if let Some(settings) = app.get_webview_window(crate::settings::WINDOW) {
                    let _ = settings.destroy();
                }
                desktop.recovery.store(true, Ordering::Release);
                let _ = crate::show_bootstrap(app);
                crate::window_host::report("Atlas couldn’t reconnect", &error);
            }
        }
    };
    // Installation already runs on a worker. Keep WebView2 creation there
    // while each old window remains available until its replacement succeeds.
    #[cfg(windows)]
    restore(app);
    #[cfg(target_os = "macos")]
    if let Err(error) = crate::run_on_main(app, restore) {
        eprintln!("Could not restore Atlas windows: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_controls_update_and_channel_views_independently() {
        use atlas_desktop_protocol::workspace_selection::{
            Config, ManagedWorkspaceInfo, ReturnChoice,
        };

        let views = |state: &State, active, waiting: Option<Waiting>, installed: bool| {
            let pinned = std::path::PathBuf::from("/Atlas/original.sqlite");
            let selection = SelectionSnapshot {
                config: Config {
                    active,
                    returning: waiting.map(|updates| ReturnChoice {
                        workspace: Role::Original,
                        updates,
                    }),
                    ..Config::default()
                },
                workspaces: vec![ManagedWorkspaceInfo {
                    role: Role::Original,
                    path: pinned.clone(),
                    exists: true,
                }],
            };
            let build = installed.then(|| Build {
                tag: "release/v1.0.0".into(),
                version: "1.0.0".into(),
                commit: "a".repeat(40),
            });
            let offer = Some(Snapshot {
                version: "1.1.0".into(),
                phase: Phase::Available,
                switching: false,
                error: None,
            });
            selection_views(state, pinned, selection, build, offer)
        };
        let idle = State::default();
        let ordinary = views(&idle, Role::Original, None, true);
        assert_eq!(ordinary.update.unwrap().version, "1.1.0");
        assert!(!ordinary.channel.unwrap().busy);
        let changed = views(&idle, Role::Separate, None, true);
        assert!(changed.update.is_none());
        let channel = changed.channel.unwrap();
        assert!(channel.busy);
        assert_eq!(
            channel.error.as_deref(),
            Some(workspace_switch::CHANGED_WORKSPACE)
        );
        let held = views(&idle, Role::Original, Some(Waiting::Hold), true);
        assert!(held.update.is_none());
        assert_eq!(
            held.channel.unwrap().returning.unwrap().updates,
            Waiting::Hold
        );
        let unmarked = views(&idle, Role::Original, Some(Waiting::Continue), false);
        assert!(unmarked.update.is_some());
        assert!(unmarked.channel.is_none());

        let switching = State {
            attempt: Some(Snapshot {
                version: "2.0.0".into(),
                phase: Phase::Downloading { percent: Some(41) },
                switching: true,
                error: None,
            }),
            ..State::default()
        };
        let switching_view = views(&switching, Role::Original, Some(Waiting::Hold), true);
        let update = switching_view.update.unwrap();
        assert!(update.switching);
        assert_eq!(update.version, "2.0.0");
        assert!(matches!(
            update.phase,
            Phase::Downloading { percent: Some(41) }
        ));
        assert!(switching_view.channel.unwrap().busy);
    }

    fn reviewed_switch(destination: Option<u32>, source: Option<u32>) -> Switch {
        let admission = |role: Role, schema: Option<u32>| Admission {
            role,
            workspace: std::path::PathBuf::from(format!("/Atlas/{}.sqlite", role.as_arg())),
            key: schema.map(|_| "workspace-key".to_owned()),
            schema,
            file_id: schema.map(|_| (1, 2)),
        };
        Switch {
            descriptor: Descriptor {
                protocol: 2,
                channel: "nightly".into(),
                version: "0.2.0-nightly.20261119092300.gaaaaaaaaaaaa".into(),
                tag: "nightly/20261119092300.gaaaaaaaaaaaa".into(),
                commit: "a".repeat(40),
                asset: PACKAGE.into(),
                sha256: "b".repeat(64),
                migration_ledger: serde_json::Value::Null,
                compatibility: Compatibility {
                    workspace_schema: 45,
                    foundry_commit: "c".repeat(40),
                    foundry_version: "8.6.1".into(),
                },
            },
            admission: admission(Role::Separate, destination),
            seed_source: source.map(|schema| admission(Role::Original, Some(schema))),
            config: workspace_switch::Config::default(),
            returning_to_stable: false,
        }
    }

    #[test]
    fn update_impact_only_reports_an_ordinary_restart_after_a_successful_check() {
        let target = reviewed_switch(Some(45), None).descriptor.compatibility;
        assert_eq!(setup_impact(&target, None, || None), UpdateImpact::Unknown);
        assert_eq!(
            setup_impact(&target, Some(44), || None),
            UpdateImpact::Upgrade
        );
        assert_eq!(
            setup_impact(&target, Some(45), || None),
            UpdateImpact::Unknown
        );
        let reference = |managed, revision| {
            Some(atlas_desktop_protocol::ReferenceStatus {
                version: atlas_desktop_protocol::VERSION,
                workspace: "/Atlas/workspace.sqlite".into(),
                generation: String::new(),
                revision,
                target: String::new(),
                target_version: String::new(),
                managed,
                offer: None,
                model_offer: None,
                can_change_models: false,
                preferred: None,
                models: Vec::new(),
            })
        };
        for (managed, revision, expected) in [
            (true, None, UpdateImpact::Unknown),
            (false, None, UpdateImpact::Restart),
            (
                true,
                Some(target.foundry_commit.clone()),
                UpdateImpact::Restart,
            ),
            (
                true,
                Some("older".into()),
                UpdateImpact::Reference {
                    version: target.foundry_version.clone(),
                },
            ),
        ] {
            assert_eq!(
                setup_impact(&target, Some(45), || reference(managed, revision)),
                expected
            );
        }
    }

    #[test]
    fn channel_review_warns_about_the_data_that_the_target_will_open() {
        for (name, destination, source, will_upgrade) in [
            ("new copy of older data", None, Some(43), true),
            ("new copy of current data", None, Some(45), false),
            ("retained older data", Some(43), None, true),
            ("retained current data", Some(45), None, false),
        ] {
            let transition = reviewed_switch(destination, source);
            transition.validate_destination().unwrap();
            let review = ChannelReview::from_switch(&transition).unwrap();
            assert_eq!(
                review.version, "0.2.0-nightly.20261119092300.gaaaaaaaaaaaa",
                "{name}"
            );
            assert_eq!(review.workspace, Role::Separate, "{name}");
            assert_eq!(review.will_seed, source.is_some(), "{name}");
            assert!(!review.needs_setup, "{name}");
            assert_eq!(review.will_upgrade, will_upgrade, "{name}");
        }
    }

    #[test]
    fn a_missing_destination_requires_copying_data_or_an_exact_pending_initialization() {
        let mut missing = reviewed_switch(None, None);
        assert!(
            missing
                .validate_destination()
                .unwrap_err()
                .contains("selected data is missing")
        );
        let before = Build {
            version: "0.1.0".into(),
            tag: "release/v0.1.0".into(),
            commit: "d".repeat(40),
        };
        missing.config.pending = Some(
            workspace_switch::new_transition(
                before,
                missing.descriptor.build(),
                missing.admission.clone(),
                false,
            )
            .unwrap(),
        );
        missing.validate_destination().unwrap();
        let review = ChannelReview::from_switch(&missing).unwrap();
        assert!(review.needs_setup);
        assert!(!review.will_upgrade);
        missing.config.pending.as_mut().unwrap().target.commit = "e".repeat(40);
        assert!(missing.validate_destination().is_err());
        reviewed_switch(None, Some(43))
            .validate_destination()
            .unwrap();
    }

    #[test]
    fn admission_failure_uses_the_target_release_even_for_same_channel_updates() {
        use atlas_desktop_protocol::workspace_selection::{
            TransitionFailure, TransitionFailureKind,
        };
        let mut descriptor = Descriptor {
            protocol: 2,
            channel: "stable".into(),
            version: "0.2.0".into(),
            tag: "release/v0.2.0".into(),
            commit: "a".repeat(40),
            asset: PACKAGE.into(),
            sha256: "b".repeat(64),
            migration_ledger: serde_json::Value::Null,
            compatibility: Compatibility {
                workspace_schema: 45,
                foundry_commit: "c".repeat(40),
                foundry_version: "8.6.1".into(),
            },
        };
        let failure = || TransitionFailure {
            kind: TransitionFailureKind::NewerSchema,
            diagnostic: "schema 46, target 45".into(),
        };
        assert_eq!(
            descriptor.admission_error(failure(), "your current data"),
            ChannelError::Unavailable(
                "This Stable release can’t open your current data yet.".into()
            )
        );
        descriptor.channel = "nightly".into();
        assert_eq!(
            descriptor.admission_error(failure(), "your current data"),
            ChannelError::Unavailable(
                "This Nightly release can’t open your current data yet.".into()
            )
        );
        assert_eq!(
            updater_check_error(tauri_plugin_updater::Error::ReleaseNotFound),
            "Atlas couldn’t retrieve this release’s update information."
        );
    }

    #[test]
    fn download_failure_copy_distinguishes_connection_from_verification() {
        let network = download_error(tauri_plugin_updater::Error::Network(
            "private diagnostic".into(),
        ));
        assert!(network.contains("couldn’t download"));
        assert!(!network.contains("private diagnostic"));
        let verification = download_error(tauri_plugin_updater::Error::MissingSignedVersion);
        assert_eq!(
            verification,
            "Atlas couldn’t verify the downloaded update. It won’t be installed."
        );
        let metadata = updater_check_error(tauri_plugin_updater::Error::SignatureUtf8(
            "private signature".into(),
        ));
        assert_eq!(
            metadata,
            "Atlas couldn’t verify this release’s update information. This release can’t be installed through Atlas."
        );
    }

    #[test]
    fn isolates_channels_and_orders_same_core_nightlies() {
        let stable = Version::parse("0.2.2").unwrap();
        let next = Version::parse("0.2.3").unwrap();
        let first = Version::parse("0.2.3-nightly.20261115092300.gabcdef123456").unwrap();
        let second = Version::parse("0.2.3-nightly.20261116092300.gabcdef123456").unwrap();
        let rc = Version::parse("0.2.3-rc.1").unwrap();
        assert!(newer(&stable, &next));
        assert!(newer(&first, &second));
        assert!(!newer(&second, &first));
        assert!(!newer(&stable, &first));
        assert!(!newer(&first, &next));
        assert!(!newer(&stable, &rc));
        assert!(!newer(&rc, &next));
    }
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "requires the Tauri CLI installed with npm --prefix desktop ci"]
    fn signed_upgrade_replaces_only_the_disposable_app() {
        use std::{fs, process::Command};
        use tauri::test::{mock_builder, mock_context, noop_assets};

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let bundle_path = "PF2e Atlas.app/Contents/MacOS/atlas-desktop";
        let current = root.join(bundle_path);
        let candidate = root.join("candidate");
        let replacement = candidate.join(bundle_path);
        for (executable, contents) in [(&current, "installed"), (&replacement, "updated")] {
            fs::create_dir_all(executable.parent().unwrap()).unwrap();
            fs::write(executable, contents).unwrap();
        }
        let cli = "PF2e Atlas.app/Contents/MacOS/atlas";
        fs::write(root.join(cli), "old cli").unwrap();
        fs::write(candidate.join(cli), "new cli").unwrap();
        let package = root.join(PACKAGE);
        assert!(
            Command::new("tar")
                .env("COPYFILE_DISABLE", "1")
                .arg("-czf")
                .arg(&package)
                .arg("-C")
                .arg(&candidate)
                .arg("PF2e Atlas.app")
                .status()
                .unwrap()
                .success()
        );

        let signer = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../node_modules/@tauri-apps/cli/tauri.js");
        let key = root.join("ephemeral.key");
        let status = Command::new("node")
            .arg(&signer)
            .args(["signer", "generate", "--ci", "--write-keys"])
            .arg(&key)
            .output()
            .unwrap()
            .status;
        assert!(status.success());
        let version = "0.2.3-nightly.20261116092300.gabcdef123456";
        let sign = |path: &std::path::Path| {
            assert!(
                Command::new("node")
                    .arg(&signer)
                    .args(["signer", "sign", "--app-version", version])
                    .arg(path)
                    .env_remove("TAURI_SIGNING_PRIVATE_KEY")
                    .env("TAURI_SIGNING_PRIVATE_KEY_PATH", &key)
                    .env("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "")
                    .output()
                    .unwrap()
                    .status
                    .success()
            );
        };
        sign(&package);
        let bytes = fs::read(&package).unwrap();
        let descriptor_path = root.join("proof.json");
        let tag = "nightly/20261116092300.gabcdef123456";
        let proof = serde_json::to_vec(&serde_json::json!({
            "protocol": 2,
            "channel": "nightly",
            "tag": tag,
            "commit": format!("abcdef123456{}", "0".repeat(28)),
            "version": version,
            "asset": PACKAGE,
            "sha256": format!("{:x}", Sha256::digest(&bytes)),
            "workspaceSchema": 45,
            "migrationLedger": {"applicationId": 1, "entries": [{"version": 45, "name": "test", "checksum": "0".repeat(64)}]},
            "foundryCommit": "0".repeat(40),
            "foundryVersion": "8.7.0"
        })).unwrap();
        fs::write(&descriptor_path, &proof).unwrap();
        sign(&descriptor_path);
        let key = fs::read_to_string(root.join("ephemeral.key.pub")).unwrap();
        let signature = fs::read_to_string(root.join(format!("{PACKAGE}.sig"))).unwrap();
        let mut context = mock_context(noop_assets());
        context
            .config_mut()
            .plugins
            .0
            .insert("updater".into(), serde_json::json!({ "pubkey": "" }));
        let app = mock_builder()
            .plugin(tauri_plugin_updater::Builder::new().build())
            .build(context)
            .unwrap();
        let updater = app
            .updater_builder()
            .pubkey(key.trim())
            .executable_path(current.clone())
            .endpoints(vec![
                Url::parse("https://example.invalid/manifest.json").unwrap(),
            ])
            .unwrap()
            .build()
            .unwrap();
        let manifest = |version| {
            serde_json::json!({
                "version": version,
                "platforms": {
                    "darwin-aarch64": {
                        "url": format!("https://example.invalid/{PACKAGE}"),
                        "signature": signature.trim()
                    }
                }
            })
        };
        let proof_signature = fs::read(root.join("proof.json.sig")).unwrap();
        let parsed_version = Version::parse(version).unwrap();
        let authenticated =
            verify_descriptor(&proof, &proof_signature, &key, &parsed_version, tag).unwrap();
        authenticated.verify_bytes(&bytes).unwrap();
        assert!(authenticated.verify_bytes(b"different package").is_err());
        let mut altered = proof.clone();
        altered.push(b' ');
        assert!(verify_descriptor(&altered, &proof_signature, &key, &parsed_version, tag).is_err());
        let wrong_version = updater.restore_update(manifest("0.2.3")).unwrap().unwrap();
        assert!(wrong_version.install(&bytes).is_err());
        assert_eq!(fs::read_to_string(&current).unwrap(), "installed");

        let signed = updater.restore_update(manifest(version)).unwrap().unwrap();
        signed.install(&bytes).unwrap();
        assert_eq!(fs::read_to_string(&current).unwrap(), "updated");
        assert_eq!(fs::read_to_string(root.join(cli)).unwrap(), "new cli");
    }
}
