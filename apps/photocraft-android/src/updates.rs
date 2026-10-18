//! Help › Check for Updates…
//!
//! Updates are manual-only: Rust asks the Activity to check now (the bridge's `check-update`
//! marker), and the Activity handles fetching, downloading and installation. Nothing checks at
//! launch and there is no startup-notification preference to confuse users.

use crate::bridge::Bridge;
use photocraft_ui_egui::menus::MenuItem;
use photocraft_ui_egui::platform_menu::PlatformMenu;
use serde_json::Value;
use std::sync::Arc;

const CHECK: &str = "android.update.check";
/// The Help row. The directory argument is retained for API compatibility with the Android shell.
pub fn menu(_files_dir: &std::path::Path, bridge: Arc<Bridge>) -> PlatformMenu {
    PlatformMenu {
        items: Box::new(move || {
            let row = |id: &str, label: &str, checked: Option<bool>| MenuItem {
                id: id.into(),
                label: label.into(),
                path: vec!["Help".into()],
                shortcut: None,
                enabled: true,
                checked,
                color: None,
            };
            vec![row(CHECK, "Check for Updates…", None)]
        }),
        run: Box::new(move |id| {
            match id {
                CHECK => bridge.request_update_check()?,
                _ => return Err(format!("unknown update command {id}")),
            }
            Ok(Value::Null)
        }),
    }
}

/// Several shells' rows as one menu: each row runs in the menu that made it.
pub fn combine(menus: Vec<PlatformMenu>) -> PlatformMenu {
    let menus = std::rc::Rc::new(std::cell::RefCell::new(menus));
    let read = menus.clone();
    PlatformMenu {
        items: Box::new(move || read.borrow().iter().flat_map(|m| (m.items)()).collect()),
        run: Box::new(move |id| {
            let mut menus = menus.borrow_mut();
            let owner = menus.iter_mut().find(|m| (m.items)().iter().any(|i| i.id == id));
            match owner {
                Some(m) => (m.run)(id),
                None => Err(format!("unknown command {id}")),
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dir() -> PathBuf {
        let d = std::env::temp_dir().join(format!("pc-updates-{}-{:?}", std::process::id(), std::thread::current().id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn updates_are_manual_only() {
        let d = dir();
        let bridge = Arc::new(Bridge::new(&d).unwrap());
        let mut m = menu(&d, bridge.clone());
        let items = (m.items)();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, CHECK);
        assert_eq!(items[0].label, "Check for Updates…");
        (m.run)(CHECK).unwrap();
        assert!(bridge.root().join("check-update").exists());
        assert!((m.run)("android.nope").is_err());
        std::fs::remove_dir_all(d).ok();
    }

    #[test]
    fn combined_menus_dispatch_to_their_owner() {
        let d = dir();
        let bridge = Arc::new(Bridge::new(&d).unwrap());
        let pad = crate::pad::Pad::new(&d);
        let mut m = combine(vec![pad.menu(), menu(&d, bridge.clone())]);
        let ids: Vec<String> = (m.items)().into_iter().map(|i| i.id).collect();
        assert!(ids.iter().any(|i| i == CHECK) && ids.iter().any(|i| i.starts_with("android.pad.")));
        (m.run)(CHECK).unwrap();
        assert!(bridge.root().join("check-update").exists());
        assert!((m.run)("android.nope").is_err());
        std::fs::remove_dir_all(d).ok();
    }
}
