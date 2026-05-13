use ksni::{menu::StandardItem, MenuItem, Tray, TrayService};
use crossbeam_channel::Sender;
use crate::i18n::{t, Lang};
use std::sync::{Arc, Mutex};

pub enum TrayMsg {
    ShowWindow,
    Quit,
}

struct XJTray {
    tx: Sender<TrayMsg>,
    lang: Arc<Mutex<Lang>>,
}

impl Tray for XJTray {
    fn id(&self) -> String {
        "xjemulator".into()
    }

    fn icon_name(&self) -> String {
        "xjemulator".into()
    }

    fn title(&self) -> String {
        "XJEmulator".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let current_lang = if let Ok(l) = self.lang.lock() {
            *l
        } else {
            Lang::En
        };

        vec![
            StandardItem {
                label: t(&current_lang, "tray_show").to_string(),
                activate: Box::new(|this: &mut XJTray| {
                    let _ = this.tx.send(TrayMsg::ShowWindow);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: t(&current_lang, "tray_quit").to_string(),
                activate: Box::new(|this: &mut XJTray| {
                    let _ = this.tx.send(TrayMsg::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub struct TrayManager {
    lang_ptr: Arc<Mutex<Lang>>,
}

impl TrayManager {
    pub fn new(lang: &Lang, tx: Sender<TrayMsg>) -> Self {
        let lang_ptr = Arc::new(Mutex::new(lang.clone()));
        let tray = XJTray {
            tx,
            lang: lang_ptr.clone(),
        };
        
        let service = TrayService::new(tray);
        service.spawn();

        Self {
            lang_ptr,
        }
    }

    pub fn update_lang(&self, lang: Lang) {
        if let Ok(mut l) = self.lang_ptr.lock() {
            *l = lang;
            // No se requiere llamar a update(); ksni llamará a menu() 
            // la próxima vez que el usuario abra el menú.
        }
    }
}
