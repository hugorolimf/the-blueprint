//! O ícone na bandeja do sistema: clique com o botão esquerdo abre o The Blueprint (beta); o menu tem "Abrir" e "Sair".
//! Os cliques viram `Evento`s do laço da janela (sem laço de consulta: com a janela fechada o app não gasta CPU).
use super::{icone, Evento};
use tao::event_loop::EventLoopProxy;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub fn criar(proxy: EventLoopProxy<Evento>) -> Result<TrayIcon, String> {
    let abrir = MenuItem::new("Abrir o The Blueprint (beta)", true, None);
    let sair = MenuItem::new("Sair", true, None);
    let menu = Menu::new();
    menu.append_items(&[&abrir, &sair]).map_err(|e| e.to_string())?;
    let (id_abrir, id_sair) = (abrir.id().clone(), sair.id().clone());
    let p = proxy.clone();
    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        if e.id == id_abrir {
            let _ = p.send_event(Evento::Mostrar);
        } else if e.id == id_sair {
            let _ = p.send_event(Evento::Sair);
        }
    }));
    TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
        if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
            let _ = proxy.send_event(Evento::Mostrar);
        }
    }));
    let icone = Icon::from_rgba(icone::rgba(32), 32, 32).map_err(|e| e.to_string())?;
    TrayIconBuilder::new().with_tooltip("The Blueprint (beta)").with_icon(icone).with_menu(Box::new(menu)).with_menu_on_left_click(false).build().map_err(|e| e.to_string())
}
