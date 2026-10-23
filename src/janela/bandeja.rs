//! O ícone na bandeja do sistema: clique com o botão esquerdo abre o The Blueprint (beta); o menu tem "Abrir", a atualização e "Sair".
//! Os cliques viram `Evento`s do laço da janela (sem laço de consulta: com a janela fechada o app não gasta CPU).
//! A atualização na bandeja: um item que mostra o estado (e vira "Atualizar para X…" quando o instalador já está baixado e conferido: o clique nele é a
//! autorização), "Verificar atualizações" e uma caixa para ligar/desligar a verificação automática.
use super::{icone, Evento};
use serde_json::Value;
use tao::event_loop::EventLoopProxy;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// O ícone e os itens que mudam com o estado da atualização (precisam viver tanto quanto o laço da janela).
pub struct Bandeja {
    _icone: TrayIcon,
    atualizar: MenuItem,
    automatica: CheckMenuItem,
}

impl Bandeja {
    /// Reflete o estado da atualização (o JSON de `Atualizador::json`) nos itens do menu.
    pub fn refletir(&self, estado: &Value) {
        let (texto, ativo) = rotulo(estado);
        self.atualizar.set_text(texto);
        self.atualizar.set_enabled(ativo);
        self.automatica.set_checked(estado["automatica"].as_bool().unwrap_or(true));
    }
}

/// O texto do item de atualização e se ele pode ser clicado: só com o instalador baixado e conferido (o clique é a autorização).
pub fn rotulo(estado: &Value) -> (String, bool) {
    let versao = estado["versao"].as_str().unwrap_or("");
    match estado["fase"].as_str().unwrap_or("") {
        "pronta" => (format!("Atualizar para a versão {versao}…"), true),
        "baixando" => (format!("Baixando a versão {versao} ({}%)", estado["percentual"].as_u64().unwrap_or(0)), false),
        "verificando" => ("Verificando atualizações…".to_string(), false),
        "instalando" => ("Atualizando…".to_string(), false),
        "erro" => ("Não consegui verificar atualizações".to_string(), false),
        _ => (format!("Versão {}: em dia", estado["atual"].as_str().unwrap_or("")), false),
    }
}

pub fn criar(proxy: EventLoopProxy<Evento>, estado: &Value) -> Result<Bandeja, String> {
    let abrir = MenuItem::new("Abrir o The Blueprint (beta)", true, None);
    let atualizar = MenuItem::new("Atualização", false, None);
    let verificar = MenuItem::new("Verificar atualizações", true, None);
    let automatica = CheckMenuItem::new("Verificar sozinho ao abrir", true, true, None);
    let sair = MenuItem::new("Sair", true, None);
    let menu = Menu::new();
    menu.append_items(&[&abrir, &PredefinedMenuItem::separator(), &atualizar, &verificar, &automatica, &PredefinedMenuItem::separator(), &sair]).map_err(|e| e.to_string())?;
    let (id_abrir, id_atualizar, id_verificar, id_auto, id_sair) = (abrir.id().clone(), atualizar.id().clone(), verificar.id().clone(), automatica.id().clone(), sair.id().clone());
    let p = proxy.clone();
    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        let evento = if e.id == id_abrir {
            Evento::Mostrar
        } else if e.id == id_atualizar {
            Evento::Atualizar
        } else if e.id == id_verificar {
            Evento::VerificarAtualizacao
        } else if e.id == id_auto {
            Evento::AlternarAutomatica
        } else if e.id == id_sair {
            Evento::Sair
        } else {
            return;
        };
        let _ = p.send_event(evento);
    }));
    TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
        if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
            let _ = proxy.send_event(Evento::Mostrar);
        }
    }));
    let icone = icone::bandeja()?;
    let tray = TrayIconBuilder::new().with_tooltip("The Blueprint (beta)").with_icon(icone).with_menu(Box::new(menu)).with_menu_on_left_click(false).build().map_err(|e| e.to_string())?;
    let bandeja = Bandeja { _icone: tray, atualizar, automatica };
    bandeja.refletir(estado);
    Ok(bandeja)
}

#[cfg(test)]
mod tests {
    use super::rotulo;
    use serde_json::json;

    #[test]
    fn so_a_versao_pronta_pode_ser_clicada() {
        assert_eq!(rotulo(&json!({"fase": "pronta", "versao": "0.2.0"})), ("Atualizar para a versão 0.2.0…".to_string(), true));
        assert_eq!(rotulo(&json!({"fase": "baixando", "versao": "0.2.0", "percentual": 40})), ("Baixando a versão 0.2.0 (40%)".to_string(), false));
        for fase in ["parada", "desligada", "verificando", "em-dia", "instalando", "erro", "inventada"] {
            assert!(!rotulo(&json!({"fase": fase, "versao": "0.2.0", "atual": "0.1.0"})).1, "{fase} não pode ser clicada");
        }
        assert_eq!(rotulo(&json!({"fase": "em-dia", "atual": "0.1.0"})).0, "Versão 0.1.0: em dia");
    }
}
