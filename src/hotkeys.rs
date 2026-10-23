use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::GlobalHotKeyManager;
use log::error;

pub struct Hotkeys {
    manager: GlobalHotKeyManager,
    pub toggle: HotKey,
    pub finish: HotKey,
    finish_registered: bool,
}

impl Hotkeys {
    /// Must be created on the main thread.
    pub fn new() -> Option<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| error!("Cannot create hotkey manager: {e}"))
            .ok()?;
        let toggle = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyT);
        let finish = HotKey::new(Some(Modifiers::CONTROL), Code::KeyX);
        if let Err(e) = manager.register(toggle) {
            error!("Cannot register Cmd+Shift+T: {e}");
        }
        Some(Self {
            manager,
            toggle,
            finish,
            finish_registered: false,
        })
    }

    pub fn register_finish(&mut self) {
        if !self.finish_registered {
            match self.manager.register(self.finish) {
                Ok(()) => self.finish_registered = true,
                Err(e) => error!("Cannot register Ctrl+X: {e}"),
            }
        }
    }

    pub fn unregister_finish(&mut self) {
        if self.finish_registered {
            let _ = self.manager.unregister(self.finish);
            self.finish_registered = false;
        }
    }
}
