//! The native history boundary for a single attached macOS workspace document.

#![allow(unsafe_code)]

use std::{cell::RefCell, collections::HashMap, ptr};

use block2::DynBlock;
use objc2::{
    DefinedClass, MainThreadOnly, Message, define_class, msg_send,
    rc::{Retained, Weak},
    runtime::{Bool, NSObject, NSObjectProtocol, ProtocolObject, Sel},
};
use objc2_web_kit::{WKBackForwardListItem, WKNavigationDelegate, WKWebView};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow, async_runtime};
use url::{Origin, Url};

use crate::{
    current_product, display, focused_window, history::Direction, product_view, staged_setup,
};

thread_local! {
    static HISTORIES: RefCell<HashMap<String, Retained<HistoryDelegate>>> = RefCell::default();
}

struct Document {
    visited: Vec<Weak<WKBackForwardListItem>>,
    last_settled_pop: u64,
    binding: Option<Binding>,
    traversal: Option<Traversal>,
}

struct Binding {
    instance: String,
    token: String,
    origin: Origin,
}

struct Traversal {
    source: Retained<WKBackForwardListItem>,
    destination: Retained<WKBackForwardListItem>,
    phase: Phase,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Navigating,
    Returning,
}

struct HistoryWindow {
    window: WebviewWindow,
    original: Retained<ProtocolObject<dyn WKNavigationDelegate>>,
    document: RefCell<Option<Document>>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = HistoryWindow]
    #[name = "AtlasHistoryDelegate"]
    struct HistoryDelegate;

    unsafe impl NSObjectProtocol for HistoryDelegate {
        #[unsafe(method(respondsToSelector:))]
        fn responds_to_selector(&self, selector: Sel) -> bool {
            let own: bool = unsafe { msg_send![super(self), respondsToSelector: selector] };
            own || self.ivars().original.respondsToSelector(selector)
        }
    }

    impl HistoryDelegate {
        #[unsafe(method_id(forwardingTargetForSelector:))]
        fn forwarding_target(&self, _selector: Sel) -> Retained<ProtocolObject<dyn WKNavigationDelegate>> {
            self.ivars().original.clone()
        }
    }

    unsafe impl WKNavigationDelegate for HistoryDelegate {
        #[unsafe(method(webView:shouldGoToBackForwardListItem:willUseInstantBack:completionHandler:))]
        fn should_traverse(
            &self,
            view: &WKWebView,
            destination: &WKBackForwardListItem,
            _instant: bool,
            completion: &DynBlock<dyn Fn(Bool)>,
        ) {
            // WebKit may be waiting in synchronous IPC. The decision uses native
            // state prepared before the callback, including any rollback permit.
            let allowed = self.admit(view, destination);
            completion.call((Bool::new(allowed),));
        }
    }
);

fn visited(document: &Document, item: &WKBackForwardListItem) -> bool {
    document
        .visited
        .iter()
        .any(|known| known.load().is_some_and(|known| ptr::eq(&*known, item)))
}

fn eligible(document: &Document, item: &WKBackForwardListItem, origin: &Origin) -> bool {
    visited(document, item)
        && unsafe { item.URL().absoluteString() }
            .and_then(|url| Url::parse(&url.to_string()).ok())
            .is_some_and(|url| &url.origin() == origin)
}

fn remember_current(document: &mut Document, view: &WKWebView) {
    let list = unsafe { view.backForwardList() };
    let Some(current) = (unsafe { list.currentItem() }) else {
        return;
    };
    if visited(document, &current) {
        return;
    }
    document.visited.retain(|item| item.load().is_some());

    // Two pushes can precede the next notification. Entries strictly after
    // a known live item are from this attachment. The prefix may include
    // filtered or restored entries from earlier documents.
    let back: Vec<_> = unsafe { list.backList() }.iter().collect();
    if let Some(anchor) = back.iter().rposition(|item| visited(document, item)) {
        document
            .visited
            .extend(back[anchor + 1..].iter().map(|item| Weak::new(&**item)));
    }
    document.visited.push(Weak::new(&*current));
}

impl HistoryDelegate {
    fn admit(&self, view: &WKWebView, destination: &WKBackForwardListItem) -> bool {
        let state = self.ivars();
        let mut document = state.document.borrow_mut();
        let Some(document) = document.as_mut() else {
            return false;
        };
        let Some(binding) = &document.binding else {
            return false;
        };
        if !staged_setup::owns_attachment(&state.window, &binding.instance, &binding.token) {
            return false;
        }
        let Some(source) = (unsafe { view.backForwardList().currentItem() }) else {
            return false;
        };
        if let Some(traversal) = document.traversal.as_mut() {
            // With native gestures and commands suspended, the only admitted
            // traversal is the Router's exact, one-use compensation.
            if traversal.phase != Phase::Navigating
                || !ptr::eq(&*traversal.destination, &*source)
                || !ptr::eq(&*traversal.source, destination)
            {
                return false;
            }
            traversal.phase = Phase::Returning;
            return true;
        }

        if !eligible(document, destination, &binding.origin) {
            return false;
        }
        remember_current(document, view);
        document.traversal = Some(Traversal {
            source,
            destination: destination.retain(),
            phase: Phase::Navigating,
        });
        unsafe { view.setAllowsBackForwardNavigationGestures(false) };
        true
    }

    fn snapshot(&self, view: &WKWebView) -> Snapshot {
        let document = self.ivars().document.borrow();
        let mut snapshot = Snapshot::default();
        if let Some(document) = document.as_ref()
            && let Some(binding) = document.binding.as_ref()
            && document.traversal.is_none()
        {
            let list = unsafe { view.backForwardList() };
            snapshot.back = unsafe { list.backItem() }
                .is_some_and(|item| eligible(document, &item, &binding.origin));
            snapshot.forward = unsafe { list.forwardItem() }
                .is_some_and(|item| eligible(document, &item, &binding.origin));
        }
        snapshot
    }

    fn go(&self, view: &WKWebView, direction: Direction) {
        if !view.window().is_some_and(|window| window.isKeyWindow()) {
            return;
        }
        let snapshot = self.snapshot(view);
        let list = unsafe { view.backForwardList() };
        let target = match direction {
            Direction::Back if snapshot.back => unsafe { list.backItem() },
            Direction::Forward if snapshot.forward => unsafe { list.forwardItem() },
            _ => None,
        };
        if let Some(target) = target {
            // The call can reenter the delegate. Keep the state borrow above.
            unsafe { view.goToBackForwardListItem(&target) };
        }
    }

    fn finish(&self, view: &WKWebView, sequence: u64) {
        let mut document = self.ivars().document.borrow_mut();
        let Some(document) = document.as_mut() else {
            return;
        };
        if sequence <= document.last_settled_pop {
            return;
        }
        let Some(current) = (unsafe { view.backForwardList().currentItem() }) else {
            return;
        };
        if document.traversal.as_ref().is_none_or(|traversal| {
            let expected = match traversal.phase {
                Phase::Navigating => &traversal.destination,
                Phase::Returning => &traversal.source,
            };
            ptr::eq(&**expected, &*current)
        }) {
            document.last_settled_pop = sequence;
            document.traversal = None;
            remember_current(document, view);
            unsafe { view.setAllowsBackForwardNavigationGestures(true) };
        }
    }
}

pub fn document_committed(window: &WebviewWindow, workspace: bool) {
    let handle = window.clone();
    let _ = window.with_webview(move |platform| {
        // Tauri provides the live native WebKit object on the main thread.
        let view = unsafe { &*platform.inner().cast::<WKWebView>() };
        let label = handle.label().to_owned();
        unsafe { view.setAllowsBackForwardNavigationGestures(false) };

        let existing = delegate(&handle);
        let proxy = match existing {
            Some(proxy) => proxy,
            None if workspace => {
                let Some(original) = (unsafe { view.navigationDelegate() }) else {
                    return;
                };
                let allocated = HistoryDelegate::alloc(view.mtm()).set_ivars(HistoryWindow {
                    window: handle,
                    original,
                    document: RefCell::new(None),
                });
                let proxy: Retained<HistoryDelegate> = unsafe { msg_send![super(allocated), init] };
                unsafe { view.setNavigationDelegate(Some(ProtocolObject::from_ref(&*proxy))) };
                HISTORIES.with_borrow_mut(|views| views.insert(label, proxy.clone()));
                proxy
            }
            None => return,
        };
        let document = workspace.then(|| Document {
            // The public list can omit entries that WebKit skips for gestures.
            // Admit only items seen as current in this document.
            visited: unsafe { view.backForwardList().currentItem() }
                .into_iter()
                .map(|item| Weak::new(&*item))
                .collect(),
            last_settled_pop: 0,
            binding: None,
            traversal: None,
        });
        proxy.ivars().document.replace(document);
    });
}

fn delegate(window: &WebviewWindow) -> Option<Retained<HistoryDelegate>> {
    HISTORIES.with_borrow(|views| views.get(window.label()).cloned())
}

pub fn forget(label: &str) {
    HISTORIES.with_borrow_mut(|views| views.remove(label));
}

pub fn menu(app: &AppHandle, direction: Direction) {
    let Some(window) = focused_window(app) else {
        return;
    };
    let Ok((base, instance)) = current_product(app) else {
        return;
    };
    // Read the Tauri URL before entering its main-thread WebView callback.
    // The immediate dispatcher holds the window ID lock during that callback.
    if !product_view(&window, &base, &instance) {
        return;
    }
    let handle = window.clone();
    let _ = window.with_webview(move |platform| {
        let view = unsafe { &*platform.inner().cast::<WKWebView>() };
        if let Some(proxy) = delegate(&handle) {
            proxy.go(view, direction);
        }
    });
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum HistoryRequest {
    Status,
    Go { direction: Direction },
    Finish { sequence: u64 },
}

#[derive(Default, Serialize)]
pub struct Snapshot {
    back: bool,
    forward: bool,
}

#[tauri::command]
pub async fn desktop_history(
    window: WebviewWindow,
    token: String,
    request: HistoryRequest,
) -> Result<Snapshot, String> {
    let (reply, mut result) = async_runtime::channel(1);
    let handle = window.clone();
    window
        .with_webview(move |platform| {
            let response = (|| {
                let view = unsafe { &*platform.inner().cast::<WKWebView>() };
                let (base, instance) = current_product(handle.app_handle())?;
                if !product_view(&handle, &base, &instance) {
                    return Ok(Snapshot::default());
                }
                if !staged_setup::owns_document(&handle, &instance, &token) {
                    return Err("Atlas history belongs to an earlier document.".into());
                }
                let Some(proxy) = delegate(&handle) else {
                    return Ok(Snapshot::default());
                };
                {
                    let mut document = proxy.ivars().document.borrow_mut();
                    let Some(document) = document.as_mut() else {
                        return Ok(Snapshot::default());
                    };
                    if document
                        .binding
                        .as_ref()
                        .is_none_or(|binding| binding.token != token)
                    {
                        document.binding = Some(Binding {
                            instance,
                            token,
                            origin: base.origin(),
                        });
                        unsafe { view.setAllowsBackForwardNavigationGestures(true) };
                    }
                }
                match request {
                    HistoryRequest::Status => {
                        if let Some(document) = proxy.ivars().document.borrow_mut().as_mut()
                            && document.traversal.is_none()
                        {
                            remember_current(document, view);
                        }
                    }
                    HistoryRequest::Go { direction } => proxy.go(view, direction),
                    HistoryRequest::Finish { sequence } => proxy.finish(view, sequence),
                }
                Ok(proxy.snapshot(view))
            })();
            let _ = reply.try_send(response);
        })
        .map_err(display)?;
    result
        .recv()
        .await
        .ok_or("Atlas history is no longer attached.")?
}
