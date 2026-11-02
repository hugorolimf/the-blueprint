//! O bloco de uma ação na linha do tempo, com as peças da folha original (`folha/acoes.rs` e `folha/barra.rs`): o rótulo em caixa alta em cima, a barra
//! de cantos retos e 26 px nas datas, os losangos dos checkpoints. Os pontos (planos, detalhes, decisões, o que espera você) moram dentro da barra;
//! quando as datas são curtas, ficam ao lado dela, onde a original punha o percentual. Zero não se desenha.

use super::{situacao, Letra, Pincel};
use crate::documento::dia;
use crate::dominio::Situacao;
use crate::folha::barra::{self, Vez};
use crate::folha::cena::{Ancora, Cena, Fio};
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com, trn};
use crate::plano::escala::{curta, Escala};
use crate::plano::folha::estilo::{com_alfa, pulso, ANDANDO, ESPERA};
use crate::plano::modelo::{Acao, EstadoEscrita};
use crate::tema::{rgba, MEDIO, PAPEL, TINTA};
use egui::Color32;

pub const TEXTOS: &[(&str, &str)] = &[
    ("Em andamento", "In progress"),
    ("Concluída", "Done"),
    ("Planejada", "Planned"),
    ("Proposta", "Proposal"),
    ("{feitos} de {total} planos", "{feitos} of {total} plans"),
    ("{n} detalhes", "{n} details"),
    ("{n} decisões", "{n} decisions"),
    ("{n} para você aprovar", "{n} for you to approve"),
    ("Mudou desde a última versão", "Changed since the last version"),
    ("Clique para abrir", "Click to open"),
];

/// A altura da barra e onde ela começa, a partir do topo da linha (os 20 e 26 px da original).
const Y_BARRA: f32 = 20.0;
const ALTO: f32 = 26.0;
const Y_ROTULO: f32 = 12.0;
/// O topo da linha, a partir do meio dela.
const ACIMA_DO_MEIO: f32 = 32.0;
/// O bloco de quem ainda não tem data ou plano: fica ao lado do hoje.
const LARGURA_SEM_DATA: f32 = 210.0;
const VAO_PONTOS: f32 = 10.0;
const PAD: f32 = 8.0;

fn letra_numero(cor: Color32) -> Letra {
    Letra::nova(10.0, cor).forte()
}

#[derive(Clone, Copy, PartialEq)]
enum Icone {
    Plano,
    Detalhe,
    Decisao,
}

fn pontos_de(a: &Acao) -> Vec<(Icone, String)> {
    let n = &a.numeros;
    let mut lista = Vec::with_capacity(3);
    if n.planos > 0 {
        lista.push((Icone::Plano, format!("{}/{}", n.prontos, n.planos)));
    }
    if n.detalhes > 0 {
        lista.push((Icone::Detalhe, n.detalhes.to_string()));
    }
    if n.decisoes > 0 {
        lista.push((Icone::Decisao, n.decisoes.to_string()));
    }
    lista
}

fn icone(c: &Pincel, i: Icone, x: f32, y: f32, cor: Color32) {
    let y0 = y - 5.0;
    match i {
        Icone::Plano => c.ret(x, y0, 10.0, 10.0, None, Some((cor, 1.2))),
        Icone::Detalhe => {
            c.ret(x, y0, 10.0, 10.0, None, Some((cor, 1.2)));
            c.fio(x, y + 3.0, x + 3.6, y - 0.5, cor, 1.1);
            c.fio(x + 3.6, y - 0.5, x + 6.0, y + 2.0, cor, 1.1);
            c.fio(x + 6.0, y + 2.0, x + 10.0, y - 1.5, cor, 1.1);
        }
        Icone::Decisao => {
            c.fio(x, y, x + 3.6, y, cor, 1.2);
            c.fio(x + 3.6, y, x + 9.0, y - 4.2, cor, 1.2);
            c.fio(x + 3.6, y, x + 9.0, y + 4.2, cor, 1.2);
        }
    }
}

fn largura_aprovar(c: &Pincel, n: u32) -> f32 {
    c.largura(&n.to_string(), &letra_numero(ESPERA)) + 14.0
}

fn largura_pontos(c: &Pincel, a: &Acao) -> f32 {
    let pontos: f32 = pontos_de(a).iter().map(|(_, s)| 14.0 + c.largura(s, &letra_numero(TINTA)) + VAO_PONTOS).sum();
    pontos + if a.numeros.aprovar > 0 { largura_aprovar(c, a.numeros.aprovar) } else { -VAO_PONTOS }
}

fn desenhar_pontos(c: &Pincel, a: &Acao, x: f32, y: f32) {
    let mut xx = x;
    for (i, s) in pontos_de(a) {
        icone(c, i, xx, y, TINTA);
        xx += 14.0;
        xx += c.texto_meio(&s, xx, y, letra_numero(TINTA)) + VAO_PONTOS;
    }
    if a.numeros.aprovar > 0 {
        c.visto(xx + 5.0, y, 7.0, ESPERA, 1.5);
        c.texto_meio(&a.numeros.aprovar.to_string(), xx + 14.0, y, letra_numero(ESPERA));
    }
}

/// O trecho do bloco na régua. `sem_data`: o lugar ao lado do hoje.
fn trecho(e: &Escala, a: &Acao) -> Option<(f32, f32)> {
    if a.escrita == EstadoEscrita::Aguardando || a.sem_data {
        let x = e.x(e.hoje) + 12.0;
        return Some((x, (x + LARGURA_SEM_DATA).min(e.x1)));
    }
    let (de, ate) = (dia(&a.de)?, dia(&a.ate)?);
    // o último dia conta inteiro
    e.recorte(de, ate + 1).map(|(xa, xb)| (xa, xb.max(xa + 6.0)))
}

/// A situação como a barra da original a entende. Aqui não há miolo de andamento: só o traço mais forte.
fn caixa(c: &Cena, a: &Acao, x: f32, y: f32, w: f32, vez: Vez) {
    if a.escrita == EstadoEscrita::Aguardando {
        let p = pulso(c.p.ctx(), 2.6);
        return c.moldura(x, y, w, ALTO, com_alfa(TINTA, 0.6 + 0.4 * p), 1.2, true);
    }
    if a.nova || a.sem_data {
        return c.moldura(x, y, w, ALTO, if a.nova { TINTA } else { MEDIO }, 1.2, true);
    }
    let s = situacao(a);
    barra::caixa(c, x, y, w, ALTO, s, vez);
    if s == Situacao::Concluido {
        barra::miolo(c, x + 1.0, y + 1.0, w - 2.0, ALTO - 2.0, s, 1.0, vez);
    }
}

fn losangos(c: &Cena, e: &Escala, a: &Acao, xa: f32, xb: f32, yc: f32, vez: Vez) {
    for k in &a.checkpoints {
        let x = e.x_iso(&k.data).or_else(|| if a.sem_data { None } else { e.x_iso(&a.ate) });
        if let Some(x) = x.filter(|x| *x >= e.x0 && *x <= e.x1) {
            barra::losango(c, xa, xb, x, yc, 7.0, k.feito, vez);
        }
    }
}

/// A linha com o meio em `y`. `luz` é o mouse em cima (0 a 1). `aberta`: o painel embaixo já traz os pontos, então a barra fica só com a forma.
pub fn desenhar(p: &Pincel, e: &Escala, a: &Acao, y: f32, luz: f32, aberta: bool) {
    let Some((xa, xb)) = trecho(e, a) else { return };
    let c = Cena { p: p.p, o: p.o };
    let topo = y - ACIMA_DO_MEIO;
    let (by, w, yc) = (topo + Y_BARRA, xb - xa, topo + Y_BARRA + ALTO / 2.0);
    let vez = Vez::da(&c, topo, by + ALTO);

    // o rótulo: fica na ponta da barra e recua quando passaria da régua
    let rotulo = format!("{} · {}", a.referencia, a.nome).to_uppercase();
    let letra = Letra::nova(11.0, TINTA);
    let extras = if a.diff.is_some() { 16.0 } else { 0.0 } + if a.agentes > 0 { 44.0 } else { 0.0 };
    let (corte, _) = p.cortado(&rotulo, e.x1 - e.x0 - extras, &letra);
    let largura = p.largura(&corte, &letra);
    let lx = xa.min(e.x1 - largura - extras).max(e.x0);
    c.texto(&corte, lx, topo + Y_ROTULO, 11.0, Peso::Normal, if luz > 0.5 { Color32::WHITE } else { TINTA }, Ancora::Inicio, false);
    let mut x_extra = lx + largura + 8.0;
    if a.diff.is_some() {
        c.p.circle_filled(c.pt(x_extra + 3.0, topo + Y_ROTULO - 4.0), 3.2, ESPERA);
        x_extra += 16.0;
    }
    if a.agentes > 0 {
        c.p.circle_filled(c.pt(x_extra + 3.0, topo + Y_ROTULO - 4.0), 2.6 + pulso(c.p.ctx(), 1.8), ANDANDO);
        p.texto(&tr_com("×{n}", &[("n", &a.agentes.to_string())]), x_extra + 10.0, topo + Y_ROTULO, letra_numero(ANDANDO));
    }

    caixa(&c, a, xa, by, w, vez);
    if luz > 0.0 {
        c.retangulo(xa, by, w, ALTO, Some(rgba(0xffffff, (0x14 as f32 * luz) as u8)), None);
    }
    losangos(&c, e, a, xa, xb, yc, vez);
    if aberta {
        if !a.sem_data && a.escrita != EstadoEscrita::Aguardando {
            cota(&c, a, xa, xb, by + ALTO);
        }
        return;
    }

    if a.escrita == EstadoEscrita::Aguardando {
        return palavra(p, tr("Aguardando plano"), xa, xb, e.x1, yc, ESPERA);
    }
    if pontos_de(a).is_empty() && a.numeros.aprovar == 0 {
        if a.sem_data {
            palavra(p, tr("sem data"), xa, xb, e.x1, yc, MEDIO);
        }
        return;
    }
    let wp = largura_pontos(p, a);
    let xp = if w >= wp + PAD * 2.0 {
        xa + PAD
    } else if xb + 10.0 + wp <= e.x1 {
        xb + 10.0
    } else {
        (xa - 10.0 - wp).max(e.x0)
    };
    // na barra hachurada, os pontos ganham o papel atrás para continuar legíveis
    if situacao(a) == Situacao::Concluido && !a.nova {
        p.ret(xp - 4.0, yc - 9.0, wp + 8.0, 18.0, Some(PAPEL), None);
    }
    desenhar_pontos(p, a, xp, yc);
}

/// A cota de dias da barra aberta, a mesma da folha original: as duas hastes, a linha, os traços oblíquos das pontas e as datas.
fn cota(c: &Cena, a: &Acao, x0: f32, x1: f32, fundo: f32) {
    let cy = fundo + 26.0;
    c.fios(Fio::Medio, &[(x0, fundo + 2.0, x0, cy + 6.0), (x1, fundo + 2.0, x1, cy + 6.0), (x0, cy, x1, cy), (x0 - 4.0, cy + 4.0, x0 + 4.0, cy - 4.0), (x1 - 4.0, cy + 4.0, x1 + 4.0, cy - 4.0)]);
    if let (Some(de), Some(ate)) = (dia(&a.de), dia(&a.ate)) {
        c.texto(&trn((ate - de + 1).max(1) as usize, "{n} DIA", "{n} DIAS"), (x0 + x1) / 2.0, cy - 5.0, 10.0, Peso::Normal, MEDIO, Ancora::Meio, false);
    }
    let (ini, fim) = (curta(&a.de), curta(&a.ate));
    if x1 - x0 < 104.0 {
        c.texto(&format!("{ini} → {fim}"), (x0 + x1) / 2.0, cy + 18.0, 10.0, Peso::Normal, MEDIO, Ancora::Meio, false);
    } else {
        c.texto(&ini, x0, cy + 18.0, 10.0, Peso::Normal, MEDIO, Ancora::Inicio, false);
        c.texto(&fim, x1, cy + 18.0, 10.0, Peso::Normal, MEDIO, Ancora::Fim, false);
    }
}

/// Uma palavra no lugar dos pontos, em caixa alta: dentro da barra se couber, senão ao lado.
fn palavra(p: &Pincel, s: &str, xa: f32, xb: f32, fim: f32, yc: f32, cor: Color32) {
    let (t, letra) = (s.to_uppercase(), Letra::nova(10.0, cor));
    let w = p.largura(&t, &letra);
    let x = if xb - xa >= w + PAD * 2.0 { xa + PAD } else if xb + 10.0 + w <= fim { xb + 10.0 } else { xa - 10.0 - w };
    p.texto_meio(&t, x, yc, letra);
}

/// A dica do bloco: a mesma informação por extenso, para quem ainda não conhece os ícones.
pub fn dica(a: &Acao) -> String {
    let n = &a.numeros;
    let estado = if a.nova {
        tr("Proposta")
    } else if a.escrita == EstadoEscrita::Aguardando {
        tr("Aguardando plano")
    } else {
        match situacao(a) {
            Situacao::Concluido => tr("Concluída"),
            Situacao::Andamento => tr("Em andamento"),
            Situacao::Planejado => tr("Planejada"),
        }
    };
    let mut partes = vec![format!("{} {}", a.referencia, a.nome), estado.to_string()];
    if !a.sem_data {
        partes.push(format!("{} → {}", curta(&a.de), curta(&a.ate)));
    }
    if n.planos > 0 {
        partes.push(tr_com("{feitos} de {total} planos", &[("feitos", &n.prontos.to_string()), ("total", &n.planos.to_string())]));
    }
    if n.detalhes > 0 {
        partes.push(tr_com("{n} detalhes", &[("n", &n.detalhes.to_string())]));
    }
    if n.decisoes > 0 {
        partes.push(tr_com("{n} decisões", &[("n", &n.decisoes.to_string())]));
    }
    if n.aprovar > 0 {
        partes.push(tr_com("{n} para você aprovar", &[("n", &n.aprovar.to_string())]));
    }
    if a.diff.is_some() {
        partes.push(tr("Mudou desde a última versão").to_string());
    }
    partes.push(tr("Clique para abrir").to_string());
    partes.join("\n")
}
