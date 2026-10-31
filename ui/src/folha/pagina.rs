//! As peças de toda folha (`moldura`, `rodape` em web/js/render.js): a moldura dupla com as réguas dos meses e das linhas, a legenda e o carimbo.

use super::cena::{Ancora, Cena, Fio};
use crate::fonte::Peso;
use crate::i18n::tr;
use crate::tema::{rgba, MEDIO, TINTA};

pub const FM: f32 = 10.0;
pub const IL: f32 = 36.0;
/// A altura da faixa de baixo (legenda e carimbo).
pub const BAIXO_FAIXA: f32 = 110.0;

pub struct Medidas {
    pub w: f32,
    pub h: f32,
    pub l: f32,
    pub r: f32,
    /// o x de cada borda de mês (`NM + 1` valores)
    pub meses: Vec<f32>,
}

/// A moldura: o retângulo fino e o grosso, as marcas dos meses (1, 2, 3…) em cima e embaixo e as letras das linhas (A, B, C…) dos lados.
pub fn moldura(c: &Cena, m: &Medidas) {
    let (w, h) = (m.w, m.h);
    c.retangulo(FM, FM, w - 2.0 * FM, h - 2.0 * FM, None, Some((TINTA, 1.0)));
    c.retangulo(IL, IL, w - 2.0 * IL, h - 2.0 * IL, None, Some((TINTA, 2.2)));
    for (i, &xa) in m.meses.iter().enumerate() {
        c.fios(Fio::Baixo, &[(xa, FM, xa, IL), (xa, h - IL, xa, h - FM)]);
        if let Some(&prox) = m.meses.get(i + 1) {
            let xc = (xa + prox) / 2.0;
            let n = (i + 1).to_string();
            c.texto(&n, xc, FM + 17.0, 10.0, Peso::Normal, MEDIO, Ancora::Meio, false);
            c.texto(&n, xc, h - FM - 9.0, 10.0, Peso::Normal, MEDIO, Ancora::Meio, false);
        }
    }
    let mut k = 0;
    while IL + k as f32 * 120.0 < h - IL {
        let (ya, yb) = (IL + k as f32 * 120.0, (IL + k as f32 * 120.0 + 120.0).min(h - IL));
        let letra = ((b'A' + (k % 26) as u8) as char).to_string();
        c.fios(Fio::Baixo, &[(FM, ya, IL, ya), (w - IL, ya, w - FM, ya)]);
        c.texto(&letra, FM + 13.0, (ya + yb) / 2.0 + 4.0, 10.0, Peso::Normal, MEDIO, Ancora::Meio, false);
        c.texto(&letra, w - FM - 13.0, (ya + yb) / 2.0 + 4.0, 10.0, Peso::Normal, MEDIO, Ancora::Meio, false);
        k += 1;
    }
    c.fios(Fio::Baixo, &[(FM, h - IL, IL, h - IL), (w - IL, h - IL, w - FM, h - IL)]);
}

/// O que o carimbo diz (a lista de blueprints, na folha 1 de `folhas`).
pub struct Carimbo<'a> {
    pub empresa: &'a str,
    pub tela: &'a str,
    pub data: &'a str,
    pub progresso: f32,
    /// «folha/total»: a lista é a 1; o blueprint `n` é a folha `n + 2`
    pub folha: usize,
    pub folhas: usize,
    /// a legenda das barras só existe na lista
    pub legenda: bool,
    pub feitas: usize,
    pub acoes: usize,
}

/// A linha de cima, a legenda das barras, a dica e o carimbo no canto.
pub fn rodape(c: &Cena, m: &Medidas, dica: &str, k: &Carimbo) {
    let (l, r, h) = (m.l, m.r, m.h);
    let y_faixa = h - IL - BAIXO_FAIXA;
    let ly = y_faixa - 4.0;
    let (cw, ch) = (440.0, 96.0);
    let (cx, cy) = (r - cw + 16.0, h - IL - ch - 4.0);
    c.fio(TINTA, 1.0, (l, y_faixa - 8.0), (r, y_faixa - 8.0));
    // a legenda: a barra é a ação; cheio é feito, vazio é o que falta
    let y = ly + 60.0;
    if k.legenda {
    c.texto(&tr("BARRA = AÇÃO · PREENCHIDO = FEITO · VAZIO = FALTA"), l, y, 8.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    c.retangulo(l + 270.0, y - 8.0, 22.0, 8.0, None, Some((TINTA, 1.0)));
    c.hachura(l + 271.0, y - 7.0, 20.0, 6.0);
    c.texto(&tr("CONCLUÍDA"), l + 298.0, y, 8.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    c.retangulo(l + 360.0, y - 8.0, 22.0, 8.0, None, Some((TINTA, 1.8)));
    c.retangulo(l + 361.0, y - 7.0, 10.0, 6.0, Some(TINTA.gamma_multiply(0.9)), None);
    c.texto(&tr("EM ANDAMENTO"), l + 388.0, y, 8.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    c.moldura(l + 470.0, y - 8.0, 22.0, 8.0, MEDIO, 1.0, true);
    c.texto(&tr("A FAZER"), l + 498.0, y, 8.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    }
    c.texto(dica, l, ly + 100.0, 11.0, Peso::Normal, MEDIO, Ancora::Inicio, false);
    // o carimbo
    c.retangulo(cx, cy, cw, ch, None, Some((TINTA, 1.6)));
    c.fios(Fio::Medio, &[(cx, cy + 36.0, cx + cw, cy + 36.0), (cx, cy + 66.0, cx + cw, cy + 66.0), (cx + 150.0, cy + 66.0, cx + 150.0, cy + ch), (cx + 250.0, cy + 66.0, cx + 250.0, cy + ch), (cx + 340.0, cy + 66.0, cx + 340.0, cy + ch)]);
    c.texto(&tr("Workspace"), cx + 8.0, cy + 11.0, 8.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    c.texto(&corta(k.empresa, 28), cx + 8.0, cy + 28.0, 16.0, Peso::Negrito, TINTA, Ancora::Inicio, true);
    c.texto(&tr("Tela"), cx + 8.0, cy + 47.0, 8.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    c.texto(&corta(k.tela, 38), cx + 8.0, cy + 61.0, 12.0, Peso::Negrito, TINTA, Ancora::Inicio, true);
    let celulas = [(tr("Data"), k.data.to_string(), 0.0), (tr("Progresso"), format!("{}%", (k.progresso * 100.0).round() as i32), 150.0), (tr("Folha"), format!("{}/{}", k.folha, k.folhas), 250.0), (tr("Ações"), format!("{}/{}", k.feitas, k.acoes), 340.0)];
    for (rotulo, valor, desloca) in celulas {
        c.texto(rotulo, cx + desloca + 8.0, cy + 77.0, 8.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        c.texto(&valor, cx + desloca + 8.0, cy + 91.0, 12.0, Peso::Negrito, TINTA, Ancora::Inicio, false);
    }
    let _ = rgba;
}

/// `corta` da página: encurta com «…» para caber em `max` letras.
pub fn corta(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        s.chars().take(max.saturating_sub(1).max(1)).collect::<String>() + "…"
    } else {
        s.to_string()
    }
}
