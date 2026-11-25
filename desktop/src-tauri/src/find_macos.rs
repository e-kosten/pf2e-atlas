//! AppKit's page finder and its bar belong to the window containing the document.

#![allow(unsafe_code)]

use std::{cell::RefCell, collections::HashMap, ptr::NonNull};

use block2::RcBlock;
use objc2::{
    MainThreadMarker, MainThreadOnly, Message, define_class, msg_send,
    rc::Retained,
    runtime::{NSObjectProtocol, ProtocolObject},
};
use objc2_app_kit::{
    NSBorderType, NSControlTextDidChangeNotification, NSScrollView, NSSearchField, NSTextFinder,
    NSTextFinderAction, NSTextFinderBarContainer,
};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSOperationQueue, NSString};
use objc2_web_kit::WKWebView;
use tauri::WebviewWindow;

thread_local! {
    static FINDERS: RefCell<HashMap<String, PageFind>> = RefCell::default();
}

// AppKit can change the DOM selection without sending the document a blur.
// Report the actual lifetime of the Find bar, including Done and Escape.
define_class!(
    #[unsafe(super = NSScrollView)]
    #[thread_kind = MainThreadOnly]
    #[name = "AtlasFindScrollView"]
    struct FindScrollView;

    impl FindScrollView {
        #[unsafe(method(setFindBarVisible:))]
        fn set_find_bar_visible(&self, visible: bool) {
            // Forward the NSTextFinderBarContainer setter to AppKit.
            unsafe {
                let _: () = msg_send![super(self), setFindBarVisible: visible];
            }
            if let Some(view) = self.documentView().and_then(|view| view.downcast::<WKWebView>().ok()) {
                let script = NSString::from_str(&format!(
                    "document.documentElement.toggleAttribute('data-atlas-find-open', {visible});
                     window.dispatchEvent(new Event('atlas-find-visibility'));"
                ));
                // The only interpolated value is a native boolean.
                unsafe { view.evaluateJavaScript_completionHandler(&script, None) };
            }
        }
    }
);

struct PageFind {
    finder: Retained<NSTextFinder>,
    changes: Retained<ProtocolObject<dyn NSObjectProtocol>>,
    view: Retained<WKWebView>,
    // NSTextFinder does not retain its client or bar container.
    _container: Retained<NSScrollView>,
}

impl PageFind {
    fn install(view: &WKWebView, main: MainThreadMarker) -> Result<Self, &'static str> {
        // The key NSWindow owns the superview throughout this main-thread call.
        let parent = unsafe { view.superview() }.ok_or("The Atlas page has no containing view.")?;
        let container: Retained<FindScrollView> =
            unsafe { msg_send![FindScrollView::alloc(main), initWithFrame: view.frame()] };
        let container = container.into_super();
        container.setAutoresizingMask(view.autoresizingMask());
        container.setBorderType(NSBorderType::NoBorder);
        container.setHasHorizontalScroller(false);
        container.setHasVerticalScroller(false);
        container.setDrawsBackground(false);

        // Keep Wry's window content view and the live document. The scroll view
        // supplies AppKit's bar and the webview tracks its remaining content area.
        parent.replaceSubview_with(view, &container);
        container.setDocumentView(Some(view));
        view.setFrameSize(container.contentSize());

        let finder = NSTextFinder::new();

        // Both objects are held until Drop disconnects these unretained links.
        unsafe {
            finder.setClient(Some(ProtocolObject::from_ref(view)));
            finder.setFindBarContainer(Some(ProtocolObject::from_ref(&*container)));
        }
        let changes = observe_query(&finder, &container);
        Ok(Self {
            changes,
            finder,
            view: view.retain(),
            _container: container,
        })
    }
}

// WKWebView's incremental NSTextFinder client counts offscreen matches without
// revealing them. Its directional search selects the text and scrolls the
// enclosing HTML pane. Use it for edits as well as the bar's arrow buttons.
fn observe_query(
    finder: &NSTextFinder,
    container: &NSScrollView,
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
                && std::ptr::eq(&*entry.view, view)
            {
                return Ok(entry.finder.clone());
            }
            let entry = PageFind::install(view, view.mtm())?;
            let finder = entry.finder.clone();
            windows.insert(label, entry);
            Ok(finder)
        });
        match finder {
            Ok(finder) => finder.performAction(NSTextFinderAction::ShowFindInterface),
            Err(error) => crate::window_host::report("Could not open Find", error),
        }
    })
}

pub fn forget(label: &str) {
    FINDERS.with_borrow_mut(|windows| {
        windows.remove(label);
    });
}
