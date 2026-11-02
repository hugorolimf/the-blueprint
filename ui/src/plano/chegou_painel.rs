//! O painel de «O que chegou» (`painel` de `tela-proposta.mjs`): quem mandou e a versão, os números e os planos em cartões, os alterados com a contagem do diff e os novos como resumo.
//! Os únicos dois botões são os de `chegou_cartao`. Sem nada à espera, a bandeja vazia; enquanto carrega, o esqueleto. Os textos ficam em `chegou::TEXTOS`.

use super::super::agentes_movimento as mov;
use super::super::agentes_pecas as pecas;
use super::super::agentes_tempo as tempo;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Chegou, Resumo};
use super::super::vazio_figuras::{self, BANDEJA_TAM};
use super::cartao::{self, cor, cor_da_ficha, VAO};
use crate::i18n::{tr, tr_com};
use crate::planta;
use crate::tema::{AMBAR, AZUL, BAIXO, CARTAO, MEDIO, TINTA, VERMELHO};
use egui::{Color32, Ui};
use serde_json::{json, Value};

const Y_PAINEL: f32 = 112.0;
const PAD: f32 = 16.0;
const CABECA_PAINEL: f32 = 172.0;
const ALTURA_VAZIA: f32 = 292.0;
const ROTULO: f32 = 30.0;
const FIM_SECAO: f32 = 10.0;
const FIM_PAINEL: f32 = 30.0;

/// Os únicos dois botões desta tela. Nenhum decide o plano: «Ver diff» só leva à ação, e «Colocar» só existe para o que `salvar_plano` deixou esperando.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oferta {
    VerDiff,
    Colocar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tipo {
    Novo,
    Alterado,
    Removido,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub referencia: String,
    pub nome: String,
    /// as fichas do cartão: o que o plano traz ou o que o diff mudou, cada uma com a sua cor
    pub fichas: Vec<(String, Color32)>,
    pub oferta: Option<Oferta>,
}

fn plural(n: u32, um: &'static str, varios: &'static str) -> String {
    tr_com(if n == 1 { um } else { varios }, &[("n", &n.to_string())])
}

fn fichas_novo(planos: u32, detalhes: u32, checkpoints: u32) -> Vec<(String, Color32)> {
    let partes = [(planos, "{n} plano", "{n} planos"), (detalhes, "{n} detalhe", "{n} detalhes"), (checkpoints, "{n} checkpoint", "{n} checkpoints")];
    partes.into_iter().filter(|p| p.0 > 0).map(|(n, um, varios)| (plural(n, um, varios), TINTA)).collect()
}

fn fichas_alterado(r: &Resumo) -> Vec<(String, Color32)> {
    let mut partes = Vec::with_capacity(5);
    if r.mais > 0 {
        partes.push(format!("+{}", r.mais));
    }
    if r.menos > 0 {
        partes.push(format!("−{}", r.menos));
    }
    if r.alterados > 0 {
        partes.push(format!("~{}", r.alterados));
    }
    if r.detalhes > 0 {
        partes.push(tr_com("+{n} det.", &[("n", &r.detalhes.to_string())]));
    }
    if r.outros > 0 {
        partes.push(tr_com("{n} outros", &[("n", &r.outros.to_string())]));
    }
    partes.into_iter().map(|p| (p.clone(), cor_da_ficha(&p))).collect()
}

/// As seções do painel, na ordem do protótipo: alterados, novos e, por último, os removidos, que o protótipo não previa e o servidor manda. Seção vazia não aparece.
pub fn secoes(c: &Chegou) -> Vec<(&'static str, Tipo, Vec<Item>)> {
    let alteradas = c.alteradas.iter().map(|a| Item { referencia: a.referencia.clone(), nome: a.nome.clone(), fichas: fichas_alterado(&a.resumo), oferta: Some(Oferta::VerDiff) }).collect();
    let novas = c.novas.iter().map(|a| Item { referencia: a.referencia.clone(), nome: a.nome.clone(), fichas: fichas_novo(a.planos, a.detalhes, a.checkpoints), oferta: a.pode_colocar.then_some(Oferta::Colocar) }).collect();
    let removidas = c.removidas.iter().map(|a| Item { referencia: a.referencia.clone(), nome: a.nome.clone(), fichas: vec![(tr("Sai do plano ao aprovar").to_string(), VERMELHO)], oferta: Some(Oferta::VerDiff) }).collect();
    let todas: [(&'static str, Tipo, Vec<Item>); 3] = [("Planos alterados", Tipo::Alterado, alteradas), ("Planos novos", Tipo::Novo, novas), ("Planos removidos", Tipo::Removido, removidas)];
    todas.into_iter().filter(|s| !s.2.is_empty()).collect()
}

/// O corpo de `gesto/colocar`: preso ao `numero` da versão, para o clique não valer numa versão que chegou depois.
pub fn corpo_colocar(ws: &str, bp: &str, numero: i64, referencia: &str) -> Value {
    json!({"workspaceId": ws, "blueprintId": bp, "numero": numero, "acoes": [referencia]})
}

pub fn altura_painel(c: &Chegou, tem: bool) -> f32 {
    if !tem {
        return ALTURA_VAZIA;
    }
    secoes(c).iter().fold(CABECA_PAINEL, |t, (_, tipo, itens)| t + ROTULO + itens.len() as f32 * (cartao::alto(*tipo) + VAO) + FIM_SECAO) + FIM_PAINEL
}

/// O ícone da versão que chegou: uma folha com a ponta dobrada e o ponto que chama a atenção.
fn folha_nova(c: &Pincel, x: f32, y: f32) {
    let corpo = vec![c.pt(x, y), c.pt(x + 14.0, y), c.pt(x + 20.0, y + 6.0), c.pt(x + 20.0, y + 24.0), c.pt(x, y + 24.0)];
    c.p.add(egui::Shape::convex_polygon(corpo, CARTAO, egui::Stroke::new(1.2, TINTA)));
    c.fio(x + 14.0, y, x + 14.0, y + 6.0, TINTA, 1.2);
    c.fio(x + 14.0, y + 6.0, x + 20.0, y + 6.0, TINTA, 1.2);
    for (i, l) in [11.0, 15.0, 19.0].into_iter().enumerate() {
        c.fio(x + 4.0, y + l, x + 12.0 + (i % 2) as f32 * 3.0, y + l, MEDIO, 1.0);
    }
    pecas::ponto(c, x + 20.0, y + 1.0, AZUL, true);
}

/// Quem mandou, a versão e há quanto esta janela a viu, o título e os números.
fn cabeca(ui: &Ui, c: &Pincel, d: &Chegou, x: f32, y: f32, wi: f32) {
    folha_nova(c, x, y + 8.0);
    let letra = Letra::nova(10.0, MEDIO).trilha(0.12);
    let agente = if d.agente.is_empty() { tr("O agente").to_string() } else { d.agente.clone() };
    c.rotulo(&c.cortado(&agente.to_uppercase(), wi - 40.0, &letra).0, x + 32.0, y + 17.0, letra);
    let visto = tempo::escrito(tempo::visto_ha(ui.ctx(), ("chegou", d.numero)));
    let versao = tr_com("Versão {n} · visto há {tempo}", &[("n", &d.numero.to_string()), ("tempo", &visto)]);
    c.texto(&c.cortado(&versao, wi - 40.0, &Letra::nova(9.5, MEDIO)).0, x + 32.0, y + 31.0, Letra::nova(9.5, MEDIO));
    c.texto(tr("O plano mudou"), x, y + 62.0, Letra::nova(15.0, TINTA).forte());
    numeros(ui, c, d, x, y + 76.0, wi);
}

/// Os números como mini cartões, cada um com o seu traço de cor.
fn numeros(ui: &Ui, c: &Pincel, d: &Chegou, x: f32, y: f32, wi: f32) {
    let (novas, alteradas, removidas) = (d.novas.len() as u32, d.alteradas.len() as u32, d.removidas.len() as u32);
    let mut lista = vec![(novas, if novas == 1 { "novo" } else { "novos" }, TINTA), (alteradas, if alteradas == 1 { "alterado" } else { "alterados" }, AMBAR), (d.iguais, if d.iguais == 1 { "igual" } else { "iguais" }, MEDIO)];
    if removidas > 0 {
        lista.push((removidas, if removidas == 1 { "removido" } else { "removidos" }, VERMELHO));
    }
    let (vao, n) = (6.0, lista.len() as f32);
    let w = (wi - vao * (n - 1.0)) / n;
    for (i, (valor, rotulo, tom)) in lista.into_iter().enumerate() {
        let xi = x + i as f32 * (w + vao);
        let apagado = valor == 0;
        pecas::cartao(c, xi, y, w, 52.0, CARTAO.gamma_multiply(0.7), BAIXO.gamma_multiply(0.4), Some(tom.gamma_multiply(if apagado { 0.25 } else { 0.95 })));
        let numero = mov::suave(ui.ctx(), ("chegou-n", rotulo), valor as f32).round() as u32;
        c.texto(&numero.to_string(), xi + 9.0, y + 30.0, Letra::nova(20.0, if apagado { MEDIO.gamma_multiply(0.6) } else { tom }).forte());
        c.texto(tr(rotulo), xi + 9.0, y + 45.0, Letra::nova(9.5, MEDIO));
    }
}

/// O título da seção: o nome em caixa alta, quantos são e um fio por baixo.
fn rotulo_da_secao(c: &Pincel, nome: &str, n: usize, tom: Color32, x: f32, y: f32, wi: f32) {
    let letra = Letra::nova(9.5, MEDIO).trilha(0.12);
    let w = c.rotulo(nome, x, y, letra);
    pecas::ficha(c, x + w + 8.0, y - 11.5, &n.to_string(), tom);
    c.fio(x, y + 7.0, x + wi, y + 7.0, BAIXO.gamma_multiply(0.45), 1.0);
}

/// O painel vazio: a bandeja à espera e a frase que diz que está tudo em dia.
fn vazio(c: &Pincel, carregou: bool, x: f32, y: f32, wi: f32) {
    if !carregou {
        c.texto(tr("Carregando…"), x, y + 36.0, Letra::nova(14.0, TINTA).forte());
        vazio_figuras::esqueleto(c, x, y + 60.0, wi, 3, 62.0, 8.0);
        return;
    }
    let k = (wi / BANDEJA_TAM.0).min(22.0);
    let alto = BANDEJA_TAM.1 * k;
    vazio_figuras::bandeja(c.p, c.pt(x + wi / 2.0, y + 36.0 + alto / 2.0), k);
    let base = y + 70.0 + alto;
    c.texto(tr("Nada chegou"), x + wi / 2.0, base, Letra::nova(15.0, TINTA).forte().meio());
    c.texto(&c.cortado(tr("Nenhuma versão à sua espera."), wi, &Letra::nova(10.5, MEDIO)).0, x + wi / 2.0, base + 20.0, Letra::nova(10.5, MEDIO).meio());
}

/// O painel da direita: quem mandou, os números (quatro, se algo saiu) e os cartões. Devolve o botão clicado.
#[allow(clippy::too_many_arguments)]
pub fn painel(ui: &mut Ui, c: &Pincel, d: &Chegou, carregou: bool, tem: bool, xp: f32, x1: f32, h: f32) -> Option<(Oferta, String)> {
    let t = mov::entrada(ui.ctx(), "chegou", 0);
    let entra = mov::Entra::nova(c, t);
    let cp = &entra.pincel();
    let altura = if tem { h - 110.0 - Y_PAINEL } else { ALTURA_VAZIA };
    let area = cp.caixa(xp, Y_PAINEL, x1 - xp, altura);
    cp.ret(xp, Y_PAINEL, x1 - xp, altura, Some(CARTAO.gamma_multiply(0.9)), Some((TINTA.gamma_multiply(0.9), 1.0)));
    planta::canto(cp.p, area);
    let (x, wi) = (xp + PAD, x1 - xp - 2.0 * PAD);
    if !tem {
        vazio(cp, carregou, x, Y_PAINEL + 14.0, wi);
        return None;
    }
    cabeca(ui, cp, d, x, Y_PAINEL + 16.0, wi);
    let (mut y, mut clicado, mut indice) = (Y_PAINEL + CABECA_PAINEL, None, 0);
    for (rotulo, tipo, itens) in secoes(d) {
        rotulo_da_secao(cp, tr(rotulo), itens.len(), cor(tipo), x, y, wi);
        y += ROTULO - 10.0;
        for item in itens {
            if let Some(o) = cartao::desenhar(ui, c, &item, tipo, indice, &d.agente, d.numero, x, y, wi) {
                clicado = Some((o, item.referencia.clone()));
            }
            y += cartao::alto(tipo) + VAO;
            indice += 1;
        }
        y += FIM_SECAO + 10.0;
    }
    clicado
}
