//! The tray item on Windows and macOS: the same menu as on Linux, so closing
//! the window leaves the music playing on every desktop.
//!
//! Windows runs the item on its own thread with a message loop, like the
//! Linux one. macOS allows status items on the main thread only, and only
//! while its event loop runs, so there the item is created with the first
//! window and, while no window exists, the headless loop in `main` pumps
//! the application's events itself. Its Dock icon remains available and a
//! Dock activation recreates the window.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::backend::Waker;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayCommand {
    Show,
    ShowHide,
    PlayPause,
    Next,
    Previous,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitReason {
    Wake,
    Deadline,
}

type Wake = Arc<dyn Fn() + Send + Sync>;

const SHOW: &str = "show";
const PLAY_PAUSE: &str = "play-pause";
const NEXT: &str = "next";
const PREVIOUS: &str = "previous";
const QUIT: &str = "quit";

fn command_for(id: &MenuId) -> Option<TrayCommand> {
    match id.0.as_str() {
        SHOW => Some(TrayCommand::ShowHide),
        PLAY_PAUSE => Some(TrayCommand::PlayPause),
        NEXT => Some(TrayCommand::Next),
        PREVIOUS => Some(TrayCommand::Previous),
        QUIT => Some(TrayCommand::Quit),
        _ => None,
    }
}

fn play_pause_label(playing: bool) -> &'static str {
    if playing { "Pause" } else { "Play" }
}

fn left_click(sender: &Sender<TrayCommand>, wake: &Wake, on_windows: bool) {
    // Each release of a Windows double-click arrives separately, possibly
    // on opposite sides of window creation. Both must ask to show it.
    let command = if on_windows {
        TrayCommand::Show
    } else {
        TrayCommand::ShowHide
    };
    if sender.send(command).is_ok() {
        wake();
    }
}

/// The item, and the menu entry whose label follows playback.
struct Item {
    _icon: TrayIcon,
    play_pause: MenuItem,
}

/// Builds the item on the current thread and routes its events to `sender`.
fn build(sender: Sender<TrayCommand>, wake: Wake) -> Result<Item, Box<dyn std::error::Error>> {
    let size = 32u32;
    #[cfg(not(target_os = "macos"))]
    let icon = Icon::from_rgba(crate::util::app_icon_rgba(size as usize), size, size)?;
    // macOS draws template images itself, black or white to match the menu
    // bar, so the item looks native in either theme.
    #[cfg(target_os = "macos")]
    let icon = Icon::from_rgba(crate::util::tray_template_rgba(size as usize), size, size)?;
    let menu = Menu::new();
    let play_pause = MenuItem::with_id(PLAY_PAUSE, play_pause_label(false), true, None);
    menu.append_items(&[
        &MenuItem::with_id(SHOW, "Show or hide Spotidark", true, None),
        &PredefinedMenuItem::separator(),
        &play_pause,
        &MenuItem::with_id(NEXT, "Next", true, None),
        &MenuItem::with_id(PREVIOUS, "Previous", true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(QUIT, "Quit", true, None),
    ])?;
    let builder = TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("Spotidark")
        .with_menu(Box::new(menu));
    // A Windows click raises the window; macOS keeps its visibility toggle.
    // Both platforms keep the menu on right click.
    #[cfg(any(windows, target_os = "macos"))]
    let builder = builder.with_menu_on_left_click(false);
    #[cfg(target_os = "macos")]
    let builder = builder.with_icon_as_template(true);
    let icon = builder.build()?;

    let menu_sender = sender.clone();
    let menu_wake = Arc::clone(&wake);
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if let Some(command) = command_for(&event.id)
            && menu_sender.send(command).is_ok()
        {
            menu_wake();
        }
    }));
    tray_icon::TrayIconEvent::set_event_handler(Some(move |event: tray_icon::TrayIconEvent| {
        if let tray_icon::TrayIconEvent::Click {
            button: tray_icon::MouseButton::Left,
            button_state: tray_icon::MouseButtonState::Up,
            ..
        } = event
        {
            left_click(&sender, &wake, cfg!(windows));
        }
    }));

    Ok(Item {
        _icon: icon,
        play_pause,
    })
}

#[cfg(windows)]
mod host {
    use super::*;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, MSG, PostThreadMessageW, TranslateMessage, WM_APP,
    };

    /// Runs the item on its own thread. Answers with the thread's id once
    /// the item exists, or with why it could not be made.
    pub fn start(
        sender: Sender<TrayCommand>,
        wake: Wake,
        playing: Receiver<bool>,
    ) -> Result<u32, String> {
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("fastpotify-tray".to_owned())
            .spawn(move || {
                let item = match build(sender, wake) {
                    Ok(item) => item,
                    Err(error) => {
                        let _ = ready_tx.send(Err(error.to_string()));
                        return;
                    }
                };
                let _ = ready_tx.send(Ok(unsafe { GetCurrentThreadId() }));
                let mut message: MSG = unsafe { std::mem::zeroed() };
                while unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) } > 0 {
                    if message.message == WM_APP {
                        while let Ok(playing) = playing.try_recv() {
                            item.play_pause.set_text(play_pause_label(playing));
                        }
                        continue;
                    }
                    unsafe {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            });
        if let Err(error) = spawned {
            return Err(error.to_string());
        }
        ready_rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "the tray thread did not answer".to_string())?
    }

    /// Wakes the thread's message loop to read what was sent to it.
    pub fn poke(thread_id: u32) {
        unsafe {
            PostThreadMessageW(thread_id, WM_APP, 0, 0);
        }
    }
}

#[cfg(windows)]
pub struct TrayService {
    commands: Receiver<TrayCommand>,
    playing: bool,
    playing_tx: Sender<bool>,
    thread_id: u32,
}

#[cfg(windows)]
impl TrayService {
    /// Registers the tray item. `None` when it cannot be made.
    pub fn spawn(wake: impl Fn() + Send + Sync + 'static) -> Option<Self> {
        let (sender, commands) = std::sync::mpsc::channel();
        let (playing_tx, playing_rx) = std::sync::mpsc::channel();
        match host::start(sender, Arc::new(wake), playing_rx) {
            Ok(thread_id) => Some(Self {
                commands,
                playing: false,
                playing_tx,
                thread_id,
            }),
            Err(error) => {
                log::info!("no system tray available: {error}");
                None
            }
        }
    }

    pub fn drain_commands(&self) -> Vec<TrayCommand> {
        self.commands.try_iter().collect()
    }

    /// Keeps the menu's Play/Pause label matching reality.
    pub fn set_playing(&mut self, playing: bool) {
        if self.playing != playing {
            self.playing = playing;
            if self.playing_tx.send(playing).is_ok() {
                host::poke(self.thread_id);
            }
        }
    }

    /// Nothing to do: the item lives on its own thread from the start.
    pub fn attach(&mut self) {}

    /// Nothing to do either; see `attach`.
    pub fn hidden(&mut self) {}
}

#[cfg(target_os = "macos")]
mod host {
    use std::cell::RefCell;
    use std::ffi::{CString, c_void};

    use objc2::runtime::{AnyClass, AnyObject, Bool, MethodImplementation, Sel};
    use objc2::{Encode, MainThreadMarker, rc::autoreleasepool, sel};
    use objc2_app_kit::{NSApplication, NSEventMask};
    use objc2_foundation::{NSDate, NSDefaultRunLoopMode};

    use super::*;

    type CFRunLoopRef = *mut c_void;
    type CFRunLoopSourceRef = *mut c_void;
    type CFStringRef = *const c_void;

    #[repr(C)]
    struct CFRunLoopSourceContext {
        version: isize,
        info: *mut c_void,
        retain: Option<extern "C" fn(*const c_void) -> *const c_void>,
        release: Option<extern "C" fn(*const c_void)>,
        copy_description: Option<extern "C" fn(*const c_void) -> CFStringRef>,
        equal: Option<extern "C" fn(*const c_void, *const c_void) -> u8>,
        hash: Option<extern "C" fn(*const c_void) -> usize>,
        schedule: Option<extern "C" fn(*const c_void, CFRunLoopRef, CFStringRef)>,
        cancel: Option<extern "C" fn(*const c_void, CFRunLoopRef, CFStringRef)>,
        perform: extern "C" fn(*const c_void),
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        static kCFRunLoopDefaultMode: CFStringRef;
        static kCFRunLoopCommonModes: CFStringRef;
        fn CFRunLoopGetMain() -> CFRunLoopRef;
        fn CFRunLoopRunInMode(mode: CFStringRef, seconds: f64, return_after_source: u8) -> i32;
        fn CFRunLoopWakeUp(run_loop: CFRunLoopRef);
        fn CFRunLoopAddSource(
            run_loop: CFRunLoopRef,
            source: CFRunLoopSourceRef,
            mode: CFStringRef,
        );
        fn CFRunLoopRemoveSource(
            run_loop: CFRunLoopRef,
            source: CFRunLoopSourceRef,
            mode: CFStringRef,
        );
        fn CFRunLoopSourceCreate(
            allocator: *const c_void,
            order: isize,
            context: *mut CFRunLoopSourceContext,
        ) -> CFRunLoopSourceRef;
        fn CFRunLoopSourceInvalidate(source: CFRunLoopSourceRef);
        fn CFRunLoopSourceSignal(source: CFRunLoopSourceRef);
        fn CFRelease(value: *const c_void);
    }

    extern "C" fn hidden_wake_perform(_: *const c_void) {}

    pub struct NativeWake {
        active: std::sync::Mutex<bool>,
        source: CFRunLoopSourceRef,
        run_loop: CFRunLoopRef,
    }

    // SAFETY: Core Foundation explicitly permits signaling and waking a run
    // loop from other threads. `active` serializes those calls with main-thread
    // source invalidation, and the owning Arc keeps both pointers alive.
    unsafe impl Send for NativeWake {}
    unsafe impl Sync for NativeWake {}

    impl NativeWake {
        pub fn new() -> Option<Arc<Self>> {
            MainThreadMarker::new()?;
            let mut context = CFRunLoopSourceContext {
                version: 0,
                info: std::ptr::null_mut(),
                retain: None,
                release: None,
                copy_description: None,
                equal: None,
                hash: None,
                schedule: None,
                cancel: None,
                perform: hidden_wake_perform,
            };
            // SAFETY: this runs on the main thread; the version-zero source
            // keeps no Rust pointer and is owned by the returned Arc.
            let (run_loop, source) = unsafe {
                let run_loop = CFRunLoopGetMain();
                let source = CFRunLoopSourceCreate(std::ptr::null(), isize::MAX - 1, &mut context);
                (run_loop, source)
            };
            if run_loop.is_null() || source.is_null() {
                if !source.is_null() {
                    // SAFETY: `source` is a create-rule Core Foundation object.
                    unsafe { CFRelease(source.cast_const()) };
                }
                log::warn!("the hidden-window run-loop wake source is unavailable");
                return None;
            }
            // SAFETY: both handles are valid and CommonModes is a process-wide
            // Core Foundation constant. Registration is performed on main.
            unsafe {
                CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
            }
            Some(Arc::new(Self {
                active: std::sync::Mutex::new(true),
                source,
                run_loop,
            }))
        }

        pub fn signal(&self) {
            let active = self.active.lock().unwrap_or_else(|p| p.into_inner());
            if *active {
                // SAFETY: signaling and waking are thread-safe Core Foundation
                // operations; the Arc retains the source for both calls.
                unsafe {
                    CFRunLoopSourceSignal(self.source);
                    CFRunLoopWakeUp(self.run_loop);
                }
            }
        }

        pub fn deactivate(&self) {
            debug_assert!(MainThreadMarker::new().is_some());
            let mut active = self.active.lock().unwrap_or_else(|p| p.into_inner());
            if *active {
                // SAFETY: the scoped waiter is created and dropped on the main
                // thread. The mutex excludes an in-flight signal operation.
                unsafe {
                    CFRunLoopRemoveSource(self.run_loop, self.source, kCFRunLoopCommonModes);
                    CFRunLoopSourceInvalidate(self.source);
                }
                *active = false;
            }
        }
    }

    impl Drop for NativeWake {
        fn drop(&mut self) {
            // SAFETY: the create-rule source is released exactly once. An
            // in-flight signal owns another Arc and therefore delays this drop.
            unsafe { CFRelease(self.source.cast_const()) };
        }
    }

    thread_local! {
        /// The status item, which only the main thread may touch.
        pub static ITEM: RefCell<Option<Item>> = const { RefCell::new(None) };
        /// The channel a Dock click asks through, and the wake that makes
        /// somebody read it.
        pub(super) static REOPEN: RefCell<Option<(Sender<TrayCommand>, Wake)>> = const { RefCell::new(None) };
    }

    /// A Dock click, asking for the app back.
    ///
    /// `has_visible_windows` is not the question it sounds like: a window
    /// sitting in the Dock still counts as visible, which is exactly the
    /// case that needs help, so the flag is not consulted. Asking for a
    /// window that is already up costs a focus and nothing else.
    pub(super) fn request_reopen(_has_visible_windows: bool) -> Bool {
        REOPEN.with(|slot| {
            if let Some((sender, wake)) = slot.borrow().as_ref() {
                let _ = sender.send(TrayCommand::Show);
                // Ask for a frame as well as leaving a message. A minimized
                // window is drawn none at all, and every repaint this app
                // schedules is armed by the frame before it, so the only
                // reader of that message is a loop that has already stopped.
                wake();
            }
        });
        Bool::YES
    }

    extern "C-unwind" fn application_should_handle_reopen(
        _delegate: *mut AnyObject,
        _selector: Sel,
        _application: *mut NSApplication,
        has_visible_windows: Bool,
    ) -> Bool {
        request_reopen(has_visible_windows.as_bool())
    }

    fn install_reopen_handler(app: &NSApplication) {
        let Some(delegate) = app.delegate() else {
            log::warn!("the macOS application delegate is unavailable");
            return;
        };
        let delegate: &AnyObject = AsRef::<AnyObject>::as_ref(&*delegate);
        let class = delegate.class();
        let selector = sel!(applicationShouldHandleReopen:hasVisibleWindows:);
        if class.responds_to(selector) {
            return;
        }
        let implementation: extern "C-unwind" fn(
            *mut AnyObject,
            Sel,
            *mut NSApplication,
            Bool,
        ) -> Bool = application_should_handle_reopen;
        let types = CString::new(format!("{}@:@{}", Bool::ENCODING, Bool::ENCODING))
            .expect("valid Objective-C type encoding");
        let installed = unsafe {
            objc2::ffi::class_addMethod(
                class as *const AnyClass as *mut AnyClass,
                selector,
                implementation.__imp(),
                types.as_ptr(),
            )
        };
        if !installed.as_bool() {
            log::warn!("the macOS Dock reopen handler could not be installed");
        }
    }

    /// Creates the item, once, on the main thread.
    pub fn create(sender: Sender<TrayCommand>, wake: Wake, playing: bool) {
        let Some(mtm) = MainThreadMarker::new() else {
            log::warn!("the status item can only be made on the main thread");
            return;
        };
        REOPEN.with(|slot| *slot.borrow_mut() = Some((sender.clone(), Arc::clone(&wake))));
        install_reopen_handler(&NSApplication::sharedApplication(mtm));
        match build(sender, wake) {
            Ok(item) => {
                item.play_pause.set_text(play_pause_label(playing));
                ITEM.with(|slot| *slot.borrow_mut() = Some(item));
            }
            Err(error) => log::info!("no status item: {error}"),
        }
    }

    pub fn exists() -> bool {
        ITEM.with(|slot| slot.borrow().is_some())
    }

    pub fn set_playing(playing: bool) {
        ITEM.with(|slot| {
            if let Some(item) = slot.borrow().as_ref() {
                item.play_pause.set_text(play_pause_label(playing));
            }
        });
    }

    pub fn activate() {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let app = NSApplication::sharedApplication(mtm);
        #[allow(deprecated)]
        app.activateIgnoringOtherApps(true);
    }

    /// Gives already-ready native work one bounded turn before a sticky wake
    /// returns. Repeated worker wakes must not starve Dock or media callbacks.
    fn service_one_ready_native_event(app: &NSApplication) {
        // SAFETY: the default run-loop mode is a process-wide constant. A
        // zero timeout services a ready source without beginning another wait.
        unsafe {
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.0, 1);
        }
        let past = NSDate::distantPast();
        // At most one queued AppKit event keeps the sticky path bounded.
        if let Some(event) = app.nextEventMatchingMask_untilDate_inMode_dequeue(
            NSEventMask::Any,
            Some(&past),
            unsafe { NSDefaultRunLoopMode },
            true,
        ) {
            app.sendEvent(&event);
        }
    }

    /// Runs AppKit and the main run loop until the sticky generation changes
    /// or the caller's deadline arrives.
    pub fn pump(waker: &Waker, observed: u64, duration: Duration) -> WaitReason {
        let Some(mtm) = MainThreadMarker::new() else {
            return if waker.wait_for_change(observed, duration) {
                WaitReason::Wake
            } else {
                WaitReason::Deadline
            };
        };
        autoreleasepool(|_| {
            let app = NSApplication::sharedApplication(mtm);
            let started = std::time::Instant::now();
            service_one_ready_native_event(&app);
            // SAFETY: these extern statics are process-wide constants.
            let (appkit_mode, run_loop_mode) =
                unsafe { (NSDefaultRunLoopMode, kCFRunLoopDefaultMode) };
            loop {
                if waker.generation() != observed {
                    return WaitReason::Wake;
                }
                let past = NSDate::distantPast();
                while let Some(event) = app.nextEventMatchingMask_untilDate_inMode_dequeue(
                    NSEventMask::Any,
                    Some(&past),
                    appkit_mode,
                    true,
                ) {
                    app.sendEvent(&event);
                    if waker.generation() != observed {
                        return WaitReason::Wake;
                    }
                    if started.elapsed() >= duration {
                        return WaitReason::Deadline;
                    }
                }
                if waker.generation() != observed {
                    return WaitReason::Wake;
                }
                let remaining = duration.saturating_sub(started.elapsed());
                if remaining.is_zero() {
                    return WaitReason::Deadline;
                }
                // SAFETY: the default run-loop mode is valid for the process.
                // The registered source makes this return after a worker wake;
                // normal AppKit sources are also served during the call.
                unsafe {
                    CFRunLoopRunInMode(run_loop_mode, remaining.as_secs_f64(), 1);
                }
            }
        })
    }

    /// A source allocation failure is rare, but must not strand the status
    /// item. Preserve the former bounded AppKit pump in that case.
    pub fn pump_fallback(waker: &Waker, observed: u64, duration: Duration) -> WaitReason {
        let duration = duration.min(Duration::from_millis(150));
        let Some(mtm) = MainThreadMarker::new() else {
            return if waker.wait_for_change(observed, duration) {
                WaitReason::Wake
            } else {
                WaitReason::Deadline
            };
        };
        autoreleasepool(|_| {
            let app = NSApplication::sharedApplication(mtm);
            let started = std::time::Instant::now();
            service_one_ready_native_event(&app);
            if waker.generation() != observed {
                return WaitReason::Wake;
            }
            let remaining = duration.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return WaitReason::Deadline;
            }
            let deadline = NSDate::dateWithTimeIntervalSinceNow(remaining.as_secs_f64());
            // SAFETY: reading an extern static AppKit defines and never changes.
            let mode = unsafe { NSDefaultRunLoopMode };
            loop {
                if waker.generation() != observed {
                    return WaitReason::Wake;
                }
                match app.nextEventMatchingMask_untilDate_inMode_dequeue(
                    NSEventMask::Any,
                    Some(&deadline),
                    mode,
                    true,
                ) {
                    Some(event) => {
                        app.sendEvent(&event);
                        if waker.generation() != observed {
                            return WaitReason::Wake;
                        }
                        if started.elapsed() >= duration {
                            return WaitReason::Deadline;
                        }
                    }
                    None => return WaitReason::Deadline,
                }
            }
        })
    }
}

#[cfg(target_os = "macos")]
pub struct TrayService {
    commands: Receiver<TrayCommand>,
    playing: bool,
    /// What the item needs, until the first window lets it be made.
    pending: Option<(Sender<TrayCommand>, Wake)>,
}

#[cfg(target_os = "macos")]
impl TrayService {
    /// Prepares the item. It is made with the first window, when AppKit's
    /// event loop is running, which it must be.
    pub fn spawn(wake: impl Fn() + Send + Sync + 'static) -> Option<Self> {
        let (sender, commands) = std::sync::mpsc::channel();
        Some(Self {
            commands,
            playing: false,
            pending: Some((sender, Arc::new(wake))),
        })
    }

    pub fn drain_commands(&self) -> Vec<TrayCommand> {
        self.commands.try_iter().collect()
    }

    /// Keeps the menu's Play/Pause label matching reality.
    pub fn set_playing(&mut self, playing: bool) {
        if self.playing != playing {
            self.playing = playing;
            host::set_playing(playing);
        }
    }

    /// A window exists: make the item if this is the first one and bring the
    /// application forward.
    pub fn attach(&mut self) {
        if let Some((sender, wake)) = self.pending.take() {
            host::create(sender, wake, self.playing);
        }
        if host::exists() {
            host::activate();
        }
    }

    /// No window: the status item and Dock icon both remain available.
    pub fn hidden(&mut self) {}
}

/// A scoped wait target for the interval where no egui window exists.
pub struct HiddenWaiter {
    waker: Waker,
    #[cfg(target_os = "macos")]
    native: Option<Arc<host::NativeWake>>,
    #[cfg(target_os = "macos")]
    registration: Option<u64>,
    #[cfg(target_os = "macos")]
    _main_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl HiddenWaiter {
    pub fn new(waker: &Waker) -> Self {
        #[cfg(windows)]
        {
            Self {
                waker: waker.clone(),
            }
        }
        #[cfg(target_os = "macos")]
        {
            let native = host::NativeWake::new();
            let registration = native.as_ref().map(|native| {
                let native = Arc::downgrade(native);
                waker.register_native_wake(Arc::new(move || {
                    if let Some(native) = native.upgrade() {
                        native.signal();
                    }
                }))
            });
            Self {
                waker: waker.clone(),
                native,
                registration,
                _main_thread: std::marker::PhantomData,
            }
        }
    }

    pub fn generation(&self) -> u64 {
        self.waker.generation()
    }

    /// Windows services already own native message threads. macOS instead
    /// serves AppKit and its main run loop while the egui window is absent.
    pub fn wait(&self, observed: u64, duration: Duration) -> WaitReason {
        #[cfg(windows)]
        {
            if self.waker.wait_for_change(observed, duration) {
                WaitReason::Wake
            } else {
                WaitReason::Deadline
            }
        }
        #[cfg(target_os = "macos")]
        {
            if self.native.is_some() {
                host::pump(&self.waker, observed, duration)
            } else {
                host::pump_fallback(&self.waker, observed, duration)
            }
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for HiddenWaiter {
    fn drop(&mut self) {
        if let Some(registration) = self.registration.take() {
            self.waker.unregister_native_wake(registration);
        }
        if let Some(native) = self.native.take() {
            native.deactivate();
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// A Dock click asks for the window whatever AppKit says about visible
    /// ones, and asks for a frame as well as leaving a message.
    ///
    /// Both halves are load-bearing. A window in the Dock is reported as
    /// visible, so a handler that trusted that flag did nothing at all for
    /// the one case that needed it; and a minimized window is drawn no
    /// frames, so the message alone would wait for a reader that has
    /// stopped running.
    #[test]
    fn a_dock_click_asks_for_the_window_and_for_a_frame() {
        let (sender, commands) = std::sync::mpsc::channel();
        let woken = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = Arc::clone(&woken);
        let wake: Wake = Arc::new(move || {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        host::REOPEN.with(|slot| *slot.borrow_mut() = Some((sender, wake)));

        // A minimized window is the reported-visible case, and the one the
        // bug report is about.
        assert!(host::request_reopen(true).as_bool());
        assert_eq!(
            commands.try_recv(),
            Ok(TrayCommand::Show),
            "a window in the Dock reports as visible, and was left there"
        );
        assert_eq!(
            woken.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "the message was left but nobody was asked to read it"
        );

        // No window at all: the tray case, which already worked.
        assert!(host::request_reopen(false).as_bool());
        assert_eq!(commands.try_recv(), Ok(TrayCommand::Show));
        assert_eq!(woken.load(std::sync::atomic::Ordering::SeqCst), 2);
        host::REOPEN.with(|slot| *slot.borrow_mut() = None);
    }
}

#[cfg(test)]
mod click_tests {
    use super::*;

    #[test]
    fn hidden_waiter_observes_a_wake_published_before_waiting() {
        let waker = Waker::default();
        let waiter = HiddenWaiter::new(&waker);
        let observed = waiter.generation();

        waker.wake();

        assert_eq!(
            waiter.wait(observed, Duration::from_secs(1)),
            WaitReason::Wake
        );
    }

    #[test]
    fn both_windows_double_click_releases_request_show_and_wake_the_app() {
        let (sender, commands) = std::sync::mpsc::channel();
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = Arc::clone(&count);
        let wake: Wake = Arc::new(move || {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        left_click(&sender, &wake, true);
        left_click(&sender, &wake, true);
        assert_eq!(
            commands.try_iter().collect::<Vec<_>>(),
            vec![TrayCommand::Show, TrayCommand::Show]
        );
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 2);
        left_click(&sender, &wake, false);
        assert_eq!(commands.try_recv(), Ok(TrayCommand::ShowHide));
    }
}
