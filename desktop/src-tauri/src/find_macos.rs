//! AppKit's page finder and its bar belong to the window containing the document.

#![allow(unsafe_code)]

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ptr::NonNull,
};

use block2::RcBlock;
use objc2::{
    DefinedClass, MainThreadOnly, Message, define_class, msg_send,
    rc::Retained,
    runtime::{NSObject, NSObjectProtocol, ProtocolObject, Sel},
};
use objc2_app_kit::{
    NSControlTextDidChangeNotification, NSSearchField, NSTextFinder, NSTextFinderAction,
    NSTextFinderBarContainer, NSTextFinderClient, NSTitlebarAccessoryViewController, NSView,
};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSOperationQueue, NSString};
use objc2_web_kit::WKWebView;
use tauri::WebviewWindow;

thread_local! {
    static FINDERS: RefCell<HashMap<String, PageFind>> = RefCell::default();
}

struct FindSession {
    page: Retained<WKWebView>,
    bar: RefCell<Option<Retained<NSView>>>,
    accessory: RefCell<Option<Retained<NSTitlebarAccessoryViewController>>>,
    visible: Cell<bool>,
}

// AppKit uses the client's editability to decide whether to offer Replace.
// Keep the live WebKit client responsible for every other Find operation.
define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = FindSession]
    #[name = "AtlasReadOnlyFindClient"]
    struct ReadOnlyFindClient;

    unsafe impl NSObjectProtocol for ReadOnlyFindClient {
        #[unsafe(method(respondsToSelector:))]
        fn responds_to_selector(&self, selector: Sel) -> bool {
            let own: bool = unsafe { msg_send![super(self), respondsToSelector: selector] };
            own || self.ivars().page.respondsToSelector(selector)
        }
    }

    unsafe impl NSTextFinderClient for ReadOnlyFindClient {
        #[unsafe(method(isEditable))]
        fn is_editable(&self) -> bool {
            false
        }
    }

    impl ReadOnlyFindClient {
        #[unsafe(method_id(forwardingTargetForSelector:))]
        fn forwarding_target(&self, _selector: Sel) -> Retained<WKWebView> {
            self.ivars().page.clone()
        }
    }

    // AppKit supplies the stock bar. Hosting it under the title bar lets the
    // native window reserve space without moving or resizing the live webview.
    unsafe impl NSTextFinderBarContainer for ReadOnlyFindClient {
        #[unsafe(method_id(findBarView))]
        fn find_bar(&self) -> Option<Retained<NSView>> {
            self.ivars().bar.borrow().clone()
        }

        #[unsafe(method(setFindBarView:))]
        fn set_find_bar(&self, bar: Option<&NSView>) {
            self.ivars().bar.replace(bar.map(Message::retain));
            self.update_accessory();
        }

        #[unsafe(method(isFindBarVisible))]
        fn bar_visible(&self) -> bool {
            self.ivars().visible.get()
        }

        #[unsafe(method(setFindBarVisible:))]
        fn set_bar_visible(&self, visible: bool) {
            let state = self.ivars();
            if state.visible.replace(visible) == visible {
                return;
            }
            self.update_accessory();
            notify_document(&state.page, visible);
        }

        #[unsafe(method(findBarViewDidChangeHeight))]
        fn bar_height_changed(&self) {
            if let Some(accessory) = self.ivars().accessory.borrow().as_ref() {
                accessory.setFullScreenMinHeight(accessory.view().frame().size.height);
            }
        }

        #[unsafe(method_id(contentView))]
        fn content_view(&self) -> Option<Retained<NSView>> {
            Some(self.ivars().page.clone().into_super())
        }
    }
);

impl ReadOnlyFindClient {
    fn update_accessory(&self) {
        let state = self.ivars();
        if let Some(previous) = state.accessory.take() {
            previous.removeFromParentViewController();
            // AppKit checks whether the stock bar is attached before it shows
            // it again. Drop the prior accessory's view hierarchy on dismissal.
            previous.view().removeFromSuperview();
        }
        let bar = state.bar.borrow().clone();
        if state.visible.get()
            && let (Some(bar), Some(window)) = (bar, state.page.window())
        {
            let accessory = NSTitlebarAccessoryViewController::new(self.mtm());
            accessory.setView(&bar);
            accessory.setFullScreenMinHeight(bar.frame().size.height);
            state.accessory.replace(Some(accessory.clone()));
            window.addTitlebarAccessoryViewController(&accessory);
        }
    }
}

fn notify_document(view: &WKWebView, visible: bool) {
    let script = NSString::from_str(&format!(
        "document.documentElement.toggleAttribute('data-atlas-find-open', {visible});
         window.dispatchEvent(new Event('atlas-find-visibility'));"
    ));
    // The only interpolated value is a native boolean.
    unsafe { view.evaluateJavaScript_completionHandler(&script, None) };
}

struct PageFind {
    finder: Retained<NSTextFinder>,
    changes: Retained<ProtocolObject<dyn NSObjectProtocol>>,
    client: Retained<ReadOnlyFindClient>,
}

impl PageFind {
    fn new(view: &WKWebView) -> Self {
        let finder = NSTextFinder::new();
        let client = ReadOnlyFindClient::alloc(view.mtm()).set_ivars(FindSession {
            page: view.retain(),
            bar: RefCell::default(),
            accessory: RefCell::default(),
            visible: Cell::default(),
        });
        // Initialize the NSObject after storing its WebKit forwarding target.
        let client: Retained<ReadOnlyFindClient> = unsafe { msg_send![super(client), init] };

        // AppKit leaves both references unretained. PageFind owns their lifetime.
        unsafe {
            finder.setClient(Some(ProtocolObject::from_ref(&*client)));
            finder.setFindBarContainer(Some(ProtocolObject::from_ref(&*client)));
        }
        let changes = observe_query(&finder, &client);
        Self {
            changes,
            finder,
            client,
        }
    }
}

// WKWebView's incremental NSTextFinder client counts offscreen matches without
// revealing them. Its directional search selects the text and scrolls the
// enclosing HTML pane. Use it for edits as well as the bar's arrow buttons.
fn observe_query(
    finder: &NSTextFinder,
    container: &ReadOnlyFindClient,
) -> Retained<ProtocolObject<dyn NSObjectProtocol>> {
    let finder = finder.retain();
    let container = container.retain();
    let on_edit = RcBlock::new(move |notification: NonNull<NSNotification>| {
        // The notification is live during the callback on the main queue.
        let notification = unsafe { notification.as_ref() };
        if let Some(field) = notification
            .object()
            .and_then(|obj| obj.downcast::<NSSearchField>().ok())
            && container.isFindBarVisible()
            && container
                .findBarView(container.mtm())
                .is_some_and(|bar| field.isDescendantOf(&bar))
            && field.stringValue().length() > 0
        {
            finder.performAction(NSTextFinderAction::NextMatch);
        }
    });
    // The center copies this block. PageFind unregisters it before teardown.
    unsafe {
        NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
            Some(NSControlTextDidChangeNotification),
            None,
            Some(&NSOperationQueue::mainQueue()),
            &on_edit,
        )
    }
}

impl Drop for PageFind {
    fn drop(&mut self) {
        unsafe {
            NSNotificationCenter::defaultCenter().removeObserver((*self.changes).as_ref());
            self.finder.setClient(None);
            self.finder.setFindBarContainer(None);
        }
    }
}

pub fn show(window: &WebviewWindow) -> tauri::Result<()> {
    let label = window.label().to_owned();
    window.with_webview(move |platform| {
        // Tauri invokes this closure on the main thread with its live WKWebView.
        let view = unsafe { &*platform.inner().cast::<WKWebView>() };
        if !view.window().is_some_and(|window| window.isKeyWindow()) {
            return;
        }
        let finder = FINDERS.with_borrow_mut(|windows| {
            if let Some(entry) = windows.get(&label)
                && std::ptr::eq(&*entry.client.ivars().page, view)
            {
                return entry.finder.clone();
            }
            let entry = PageFind::new(view);
            let finder = entry.finder.clone();
            windows.insert(label, entry);
            finder
        });
        finder.performAction(NSTextFinderAction::ShowFindInterface);
    })
}

// The bar is owned by the native window and can outlive a full page reload.
pub fn document_loaded(window: &WebviewWindow) {
    let label = window.label().to_owned();
    let _ = window.run_on_main_thread(move || {
        FINDERS.with_borrow(|windows| {
            if let Some(entry) = windows.get(&label)
                && entry.client.isFindBarVisible()
            {
                notify_document(&entry.client.ivars().page, true);
            }
        });
    });
}

pub fn forget(label: &str) {
    FINDERS.with_borrow_mut(|windows| {
        windows.remove(label);
    });
}
