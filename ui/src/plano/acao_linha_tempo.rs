//! Os checkpoints da ação aberta, como na folha original (`itens` de `folha/acoes.rs`): o losango, cheio quando feito, o nome e a data; a data fica em âmbar
//! quando passou e não foi feito.

use super::super::escala::curta;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Checkpoint};
use super::super::revisao::Alvo;
use super::rotulo;
use crate::i18n::{tr, tr_com};
use crate::menu::{self, Alvo as AlvoMenu};
use crate::tema::{AMBAR, MEDIO, TINTA};
use egui::{Color32, Sense, Ui};

const ALTO_LINHA: f32 = 24.0;
const ALTO_ROTULO: f32 = 22.0;
const X_MARCA: f32 = 6.5;
const X_TEXTO: f32 = 24.0;
const RAIO_MARCA: f32 = 6.5;

/// Quanto os checkpoints ocupam. Zero sem checkpoints.
pub fn altura(a: &Acao) -> f32 {
    if a.checkpoints.is_empty() {
        0.0
    } else {
        ALTO_ROTULO + a.checkpoints.len() as f32 * ALTO_LINHA
    }
}

fn tem_data(d: &str) -> bool {
    d.len() >= 10
}

/// Passou do dia marcado sem estar feito. As datas são ISO e comparam como texto.
fn atrasado(k: &Checkpoint, hoje: &str) -> bool {
    !k.feito && tem_data(&k.data) && !hoje.is_empty() && k.data.as_str() < hoje
}

/// Os checkpoints com o rótulo em `(x, y)`.
pub fn desenhar(ui: &Ui, c: &Pincel, alvo: Alvo, a: &Acao, hoje: &str, (x, y, w): (f32, f32, f32)) {
    if a.checkpoints.is_empty() {
        return;
    }
    let feitos = a.checkpoints.iter().filter(|k| k.feito).count();
    rotulo(c, x, y, &tr_com("Checkpoints · {feitos}/{total}", &[("feitos", &feitos.to_string()), ("total", &a.checkpoints.len().to_string())]), w);
    let topo = y + ALTO_ROTULO - 4.0;
    let centro = |i: usize| topo + i as f32 * ALTO_LINHA + ALTO_LINHA / 2.0 - 3.0;
    for (i, k) in a.checkpoints.iter().enumerate() {
        linha(ui, c, alvo, a, k, atrasado(k, hoje), (x, centro(i), w));
    }
}

fn linha(ui: &Ui, c: &Pincel, alvo: Alvo, a: &Acao, k: &Checkpoint, atraso: bool, (x, yc, w): (f32, f32, f32)) {
    let area = c.caixa(x, yc - ALTO_LINHA / 2.0, w, ALTO_LINHA);
    menu::marcar(ui, area, || AlvoMenu::Item { ws: alvo.ws.into(), bp: alvo.bp.into(), acao: a.id.clone(), tipo: "checkpoints", id: k.id.clone(), texto: k.nome.clone(), feito: k.feito });
    let resposta = ui.interact(area, ui.scope_id().with(("plano-checkpoint", &k.id)), Sense::hover());
    let sobre = resposta.hovered();
    c.losango(x + X_MARCA, yc, RAIO_MARCA, k.feito);
    let cor = if sobre { Color32::WHITE } else if k.feito { MEDIO } else { TINTA };
    let data = tem_data(&k.data).then(|| format!(" · {}", curta(&k.data)));
    let letra = Letra::nova(11.0, cor);
    let w_data = data.as_deref().map_or(0.0, |d| c.largura(d, &letra));
    let (corte, cortou) = cortar_fim(c, &k.nome, w - X_TEXTO - w_data, &letra);
    let w_nome = c.texto(&corte, x + X_TEXTO, yc + 4.0, letra);
    if let Some(d) = &data {
        c.texto(d, x + X_TEXTO + w_nome, yc + 4.0, if atraso { Letra::nova(11.0, AMBAR) } else { letra });
    }
    let dica = if atraso { Some(format!("{}\n{}", k.nome, tr("Passou da data sem ser feito"))) } else { cortou.then(|| k.nome.clone()) };
    if let Some(d) = dica {
        let _ = resposta.on_hover_text(d);
    }
}

fn cortar_fim(c: &Pincel, s: &str, max: f32, l: &Letra) -> (String, bool) {
    c.cortado(s, max.max(30.0), l)
}
