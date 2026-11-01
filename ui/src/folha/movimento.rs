//! O movimento das figuras do Claude (`claudeEntra`, `claudeSai` e `claudeAnda` de web/js/claude-camada.js): entrar crescendo a partir do centro (380 ms), sair
//! encolhendo (300 ms) e andar até o plano novo (400 a 1600 ms, 4 ms por pixel, devagar nas pontas). A figura que muda de tela só aparece no lugar, sem andar.
//! O estado fica na memória do egui, por agente e por escopo (o blueprint), e some quando a figura não é mais pedida.

use crate::agentes::Lugar;
use egui::Ui;
use std::collections::HashMap;

const ENTRA: f64 = 0.38;
const SAI: f64 = 0.30;
/// Sem ser pedida por tanto tempo, a figura é esquecida (o usuário saiu da tela dela).
const ESQUECE: f64 = 10.0;

/// O que a folha quer de uma figura neste quadro.
pub struct Alvo {
    pub agente: String,
    pub papel: Option<&'static str>,
    pub pose: &'static str,
    pub lugar: Lugar,
    pub x: f32,
    pub y: f32,
}

/// Como desenhar a figura agora.
pub struct Desenho {
    pub agente: String,
    pub papel: Option<&'static str>,
    pub pose: &'static str,
    pub x: f32,
    pub y: f32,
    /// 0 a 1, a partir do centro; vale também para a opacidade
    pub escala: f32,
    /// saindo: já não recebe clique
    pub vivo: bool,
}

#[derive(Clone)]
struct Anda {
    de: (f32, f32),
    ate: (f32, f32),
    ini: f64,
    dur: f64,
}

#[derive(Clone)]
struct Fig {
    agente: String,
    papel: Option<&'static str>,
    pose: &'static str,
    lugar: Lugar,
    tela: String,
    x: f32,
    y: f32,
    anda: Option<Anda>,
    nasceu: f64,
    saindo: Option<f64>,
    ultimo: f64,
}

#[derive(Clone, Default)]
struct Memoria(HashMap<String, Fig>);

fn suave(t: f64) -> f64 {
    if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
}

/// Avança as figuras do `escopo` (o blueprint) na `tela` para o quadro de agora e devolve o que desenhar.
pub fn passo(ui: &Ui, escopo: &str, tela: &str, alvos: &[Alvo]) -> Vec<Desenho> {
    let agora = ui.input(|i| i.time);
    let id = chave_memoria();
    let mut mem: Memoria = ui.ctx().data_mut(|d| d.get_temp(id)).unwrap_or_default();
    let (mut saida, mut animando) = (Vec::new(), false);
    for a in alvos {
        let chave = format!("{escopo}|{}", a.agente);
        let f = mem.0.entry(chave).or_insert_with(|| Fig { agente: a.agente.clone(), papel: a.papel, pose: a.pose, lugar: a.lugar.clone(), tela: tela.into(), x: a.x, y: a.y, anda: None, nasceu: agora, saindo: None, ultimo: agora });
        if f.saindo.take().is_some() {
            f.nasceu = agora;
        }
        let mudou_tela = f.tela != tela;
        if f.lugar != a.lugar && !mudou_tela {
            let (x0, y0) = atual(f, agora);
            let dist = f64::from((a.x - x0).hypot(a.y - y0));
            f.anda = Some(Anda { de: (x0, y0), ate: (a.x, a.y), ini: agora, dur: (dist * 4.0).clamp(400.0, 1600.0) / 1000.0 });
        } else if let Some(an) = &mut f.anda {
            an.ate = (a.x, a.y); // mudou o destino no meio do caminho: segue dali
        } else {
            f.x = a.x;
            f.y = a.y;
        }
        if mudou_tela {
            f.anda = None;
            f.x = a.x;
            f.y = a.y;
        }
        f.lugar = a.lugar.clone();
        f.tela = tela.into();
        f.ultimo = agora;
        f.papel = a.papel;
        f.pose = a.pose;
        let andando = f.anda.is_some() && a.pose == "trabalhando";
        let (x, y) = atual(f, agora);
        if f.anda.as_ref().is_some_and(|an| agora >= an.ini + an.dur) {
            f.anda = None;
            f.x = a.x;
            f.y = a.y;
        }
        let t = ((agora - f.nasceu) / ENTRA).clamp(0.0, 1.0);
        animando |= f.anda.is_some() || t < 1.0;
        saida.push(Desenho { agente: a.agente.clone(), papel: a.papel, pose: if andando { "andando" } else { a.pose }, x, y, escala: (1.0 - (1.0 - t).powi(3)) as f32, vivo: true });
    }
    // quem estava neste escopo e não foi pedido: encolhe e sai
    let prefixo = format!("{escopo}|");
    mem.0.retain(|chave, f| {
        if !chave.starts_with(&prefixo) || f.ultimo == agora {
            return agora - f.ultimo < ESQUECE;
        }
        let ini = *f.saindo.get_or_insert(agora);
        let t = ((agora - ini) / SAI).clamp(0.0, 1.0);
        if t >= 1.0 {
            return false;
        }
        animando = true;
        saida.push(Desenho { agente: f.agente.clone(), papel: f.papel, pose: f.pose, x: f.x, y: f.y, escala: (1.0 - t * t) as f32, vivo: false });
        true
    });
    ui.ctx().data_mut(|d| d.insert_temp(id, mem));
    if animando {
        ui.ctx().request_repaint();
    }
    saida
}

fn atual(f: &Fig, agora: f64) -> (f32, f32) {
    match &f.anda {
        Some(an) => {
            let e = suave(((agora - an.ini) / an.dur).clamp(0.0, 1.0)) as f32;
            (an.de.0 + (an.ate.0 - an.de.0) * e, an.de.1 + (an.ate.1 - an.de.1) * e)
        }
        None => (f.x, f.y),
    }
}

#[allow(deprecated)]
fn chave_memoria() -> egui::Id {
    egui::Id::new("claude-movimento")
}
