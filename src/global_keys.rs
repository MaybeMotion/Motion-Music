//! Windows: Ctrl+Alt shortcuts that control playback from any app, even
//! while Motion Music is hidden in the tray.
//!
//! A small thread registers the hotkeys with Windows and waits for them in
//! its own message loop. A shortcut another app already holds is skipped,
//! never taken from it. Other platforms have no global shortcuts here; the
//! desktop's media keys already reach the app through its media controls.

/// What a global shortcut asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    PlayPause,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
}

impl Key {
    /// Every shortcut, in the order their ids are given to Windows.
    #[cfg(any(test, windows))]
    const ALL: [Self; 5] = [
        Self::PlayPause,
        Self::Next,
        Self::Previous,
        Self::VolumeUp,
        Self::VolumeDown,
    ];

    /// The action the app applies for this shortcut.
    pub fn action(self) -> crate::model::Action {
        use crate::model::Action;
        match self {
            Self::PlayPause => Action::TogglePlay,
            Self::Next => Action::Next,
            Self::Previous => Action::Previous,
            Self::VolumeUp => Action::VolumeBy(5),
            Self::VolumeDown => Action::VolumeBy(-5),
        }
    }

    /// The shortcut a hotkey id stands for.
    #[cfg(any(test, windows))]
    fn from_id(id: usize) -> Option<Self> {
        id.checked_sub(1)
            .and_then(|index| Self::ALL.get(index).copied())
    }
}

#[cfg(windows)]
pub use windows_keys::GlobalKeys;

/// Off Windows there is nothing to register.
#[cfg(not(windows))]
pub struct GlobalKeys;

#[cfg(not(windows))]
impl GlobalKeys {
    pub fn start(_wake: impl Fn() + Send + 'static) -> Option<Self> {
        None
    }

    pub fn drain(&self) -> Vec<Key> {
        Vec::new()
    }
}

#[cfg(windows)]
mod windows_keys {
    use super::Key;
    use std::sync::mpsc;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, RegisterHotKey, UnregisterHotKey, VK_DOWN, VK_LEFT,
        VK_RIGHT, VK_SPACE, VK_UP,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetMessageW, MSG, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, WM_HOTKEY, WM_QUIT,
        WM_USER,
    };

    /// The registered shortcuts; dropping it unregisters them.
    pub struct GlobalKeys {
        thread: u32,
        keys: mpsc::Receiver<Key>,
    }

    impl GlobalKeys {
        /// Registers the shortcuts on their own thread. `wake` is called
        /// after each press so a hidden app still handles it.
        pub fn start(wake: impl Fn() + Send + 'static) -> Option<Self> {
            let (sender, keys) = mpsc::channel();
            let (ready, thread) = mpsc::sync_channel(1);
            std::thread::Builder::new()
                .name("global-shortcuts".into())
                .spawn(move || {
                    let mut message: MSG = unsafe { std::mem::zeroed() };
                    // SAFETY: plain Win32 calls on this thread's own queue.
                    unsafe {
                        // Make the queue before anyone posts to it.
                        PeekMessageW(
                            &mut message,
                            std::ptr::null_mut(),
                            WM_USER,
                            WM_USER,
                            PM_NOREMOVE,
                        );
                        let _ = ready.send(GetCurrentThreadId());
                        let held = MOD_CONTROL | MOD_ALT | MOD_NOREPEAT;
                        for (index, key) in Key::ALL.into_iter().enumerate() {
                            let vk = match key {
                                Key::PlayPause => VK_SPACE,
                                Key::Next => VK_RIGHT,
                                Key::Previous => VK_LEFT,
                                Key::VolumeUp => VK_UP,
                                Key::VolumeDown => VK_DOWN,
                            };
                            if RegisterHotKey(
                                std::ptr::null_mut(),
                                index as i32 + 1,
                                held,
                                u32::from(vk),
                            ) == 0
                            {
                                log::info!(
                                    "the global shortcut for {key:?} is taken by another app"
                                );
                            }
                        }
                        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                            if message.message == WM_HOTKEY
                                && let Some(key) = Key::from_id(message.wParam)
                            {
                                if sender.send(key).is_err() {
                                    break;
                                }
                                wake();
                            }
                        }
                        for index in 0..Key::ALL.len() {
                            UnregisterHotKey(std::ptr::null_mut(), index as i32 + 1);
                        }
                    }
                })
                .ok()?;
            let thread = thread.recv().ok()?;
            Some(Self { thread, keys })
        }

        /// The shortcuts pressed since the last call.
        pub fn drain(&self) -> Vec<Key> {
            self.keys.try_iter().collect()
        }
    }

    impl Drop for GlobalKeys {
        fn drop(&mut self) {
            // SAFETY: asks the shortcut thread's loop to end.
            unsafe { PostThreadMessageW(self.thread, WM_QUIT, 0, 0) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Action;

    #[test]
    fn each_shortcut_has_its_id_and_action() {
        assert_eq!(Key::from_id(0), None);
        assert_eq!(Key::from_id(1), Some(Key::PlayPause));
        assert_eq!(Key::from_id(5), Some(Key::VolumeDown));
        assert_eq!(Key::from_id(6), None);
        assert!(matches!(Key::PlayPause.action(), Action::TogglePlay));
        assert!(matches!(Key::VolumeUp.action(), Action::VolumeBy(5)));
        assert!(matches!(Key::VolumeDown.action(), Action::VolumeBy(-5)));
    }
}
