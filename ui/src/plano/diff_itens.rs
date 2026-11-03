//! As linhas do diff já prontas para pintar: os dois lados de cada plano, as notas e o rascunho de «Editar».

use super::quebra::{quebrar, Fileira};
use crate::fonte::{familia, Peso};
use crate::i18n::{tr, tr_com};
use crate::planta::Texto;
use crate::plano::modelo::{Acao, Diff, LinhaDiff, Palavra, Resumo, TipoLinha, TipoPalavra};
use crate::tema::{AMBAR, MEDIO, TINTA, VERDE, VERMELHO};
use egui::text::LayoutJob;
use egui::{Color32, FontId, Id, Painter};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub const TAM: f32 = 14.0;
pub const TAM_NOTA: f32 = 12.0;
pub const ALT: f32 = 28.0;
/// De uma linha quebrada para a seguinte.
pub const PASSO: f32 = 20.0;
pub const PASSO_NOTA: f32 = 17.0;
pub const X_TEXTO: f32 = 78.0;
pub const MARGEM_DIR: f32 = 12.0;

/// Um lado da linha alterada: o que é igual e o que é só deste lado, em destaque.
fn lado(palavras: &[Palavra], este: TipoPalavra) -> Vec<(String, bool)> {
    palavras.iter().filter(|p| p.t == TipoPalavra::Igual || p.t == este).map(|p| (p.s.clone(), p.t == este)).collect()
}

/// O resumo do título: `+1 · −1 · 2 alterados · +1 detalhe`.
pub fn partes(r: &Resumo) -> String {
    let n = |v: u32, um: &'static str, varios: &'static str| tr_com(if v > 1 { varios } else { um }, &[("n", &v.to_string())]);
    let mut p = Vec::new();
    if r.mais > 0 {
        p.push(format!("+{}", r.mais));
    }
    if r.menos > 0 {
        p.push(format!("−{}", r.menos));
    }
    if r.alterados > 0 {
        p.push(n(r.alterados, "{n} alterado", "{n} alterados"));
    }
    if r.detalhes > 0 {
        p.push(n(r.detalhes, "+{n} detalhe", "+{n} detalhes"));
    }
    p.join(" · ")
}

/// O resumo do título em pedaços, cada um com a cor que ele leva: `+1` verde, `−1` vermelho, `2 alterados` âmbar, `+1 detalhe` neutro.
pub fn pedacos(r: &Resumo) -> Vec<(String, Option<Color32>)> {
    let n = |v: u32, um: &'static str, varios: &'static str| tr_com(if v > 1 { varios } else { um }, &[("n", &v.to_string())]);
    let mut p = Vec::new();
    if r.mais > 0 {
        p.push((format!("+{}", r.mais), Some(VERDE)));
    }
    if r.menos > 0 {
        p.push((format!("−{}", r.menos), Some(VERMELHO)));
    }
    if r.alterados > 0 {
        p.push((n(r.alterados, "{n} alterado", "{n} alterados"), Some(AMBAR)));
    }
    if r.detalhes > 0 {
        p.push((n(r.detalhes, "+{n} detalhe", "+{n} detalhes"), None));
    }
    p
}

fn nome_campo(c: &str) -> String {
    match c {
        "area" => tr("área").to_string(),
        "depende_de" => tr("dependências").to_string(),
        "instrucoes" => tr("instruções").to_string(),
        "forma" => tr("forma").to_string(),
        "entrega" => tr("entrega").to_string(),
        "nome" => tr("nome").to_string(),
        "texto" => tr("texto").to_string(),
        "checkpoints" => tr("checkpoints").to_string(),
        "tabelas" => tr("tabelas").to_string(),
        outro => outro.to_string(),
    }
}

fn tambem_mudou(campos: &[String]) -> String {
    let c = campos.iter().map(|c| nome_campo(c)).collect::<Vec<_>>().join(", ");
    tr_com("Também mudou: {c}", &[("c", &c)])
}

/// O rascunho de «Editar», preso ao `hash` do diff: chegou outra versão, o rascunho da anterior não vale.
#[derive(Clone, Default)]
pub struct Edicao {
    pub hash: String,
    pub aberta: bool,
    pub textos: BTreeMap<String, String>,
}

pub fn id_edicao(chave: &str) -> Id {
    Id::unique(("plano-diff-edicao", chave))
}

pub fn ler_edicao(ctx: &egui::Context, chave: &str, hash: &str) -> Edicao {
    let e = ctx.data(|d| d.get_temp::<Edicao>(id_edicao(chave))).unwrap_or_default();
    if e.hash == hash {
        e
    } else {
        Edicao { hash: hash.to_string(), ..Edicao::default() }
    }
}

pub fn editaveis(d: &Diff) -> bool {
    d.linhas.iter().any(|l| l.tipo == TipoLinha::Alterado)
}

/// O campo `textos` de `gesto/revisar`: só o que mudou de verdade, por `ref`. Linha apagada também vai: é o servidor quem recusa, e o usuário vê o motivo em vez de aprovar a versão do agente sem saber.
pub fn textos_editados(linhas: &[LinhaDiff], rascunho: &BTreeMap<String, String>) -> Option<Value> {
    let mut mudados = Map::new();
    for l in linhas.iter().filter(|l| l.tipo == TipoLinha::Alterado) {
        if let Some(novo) = rascunho.get(&l.referencia).map(|t| t.trim()).filter(|t| *t != l.texto.trim()) {
            mudados.insert(l.referencia.clone(), Value::String(novo.to_string()));
        }
    }
    (!mudados.is_empty()).then_some(Value::Object(mudados))
}

#[derive(Clone, Copy, PartialEq)]
pub enum Sinal {
    Mais,
    Menos,
    Neutro,
}

impl Sinal {
    pub fn cor(self) -> Color32 {
        match self {
            Sinal::Mais => VERDE,
            Sinal::Menos => VERMELHO,
            Sinal::Neutro => MEDIO,
        }
    }
    pub fn letra(self) -> &'static str {
        match self {
            Sinal::Mais => "+",
            Sinal::Menos => "−",
            Sinal::Neutro => " ",
        }
    }
}

pub enum Item {
    Linha { sinal: Sinal, id: String, fileiras: Vec<Fileira> },
    Editor { sinal: Sinal, id: String, referencia: String, texto: String },
    Nota(Vec<Fileira>),
}

pub fn medidor<'a>(p: &'a Painter, tam: f32) -> impl Fn(&str) -> f32 + 'a {
    move |s: &str| Texto::novo(s.to_string(), tam).trilha(0.06).largura(p)
}

/// As linhas do diff já quebradas, nesta largura. Com `edicao`, a linha nova de cada plano alterado vira campo.
pub fn montar(p: &Painter, acao: &Acao, d: &Diff, w: f32, edicao: Option<&Edicao>) -> Vec<Item> {
    let medir = medidor(p, TAM);
    let medir_nota = medidor(p, TAM_NOTA);
    let largura = w - X_TEXTO - MARGEM_DIR;
    let linha = |sinal: Sinal, id: &str, pedacos: Vec<(String, bool)>| Item::Linha { sinal, id: id.to_string(), fileiras: quebrar(&pedacos, largura, &medir) };
    let nota = |t: String| Item::Nota(quebrar(&[(t, false)], largura, &medir_nota));
    let mut itens = Vec::new();
    for l in &d.linhas {
        let r = l.referencia.as_str();
        let texto = if l.texto.is_empty() { &l.antes } else { &l.texto };
        match l.tipo {
            TipoLinha::Igual => itens.push(linha(Sinal::Neutro, r, vec![(texto.clone(), false)])),
            TipoLinha::Novo => itens.push(linha(Sinal::Mais, r, vec![(texto.clone(), false)])),
            TipoLinha::Removido => itens.push(linha(Sinal::Menos, r, vec![(texto.clone(), false)])),
            TipoLinha::Alterado => {
                let mudou_texto = l.palavras.iter().any(|q| q.t != TipoPalavra::Igual);
                if mudou_texto {
                    itens.push(linha(Sinal::Menos, r, lado(&l.palavras, TipoPalavra::Menos)));
                }
                let sinal = if mudou_texto { Sinal::Mais } else { Sinal::Neutro };
                if let Some(e) = edicao {
                    let atual = e.textos.get(r).unwrap_or(&l.texto).clone();
                    itens.push(Item::Editor { sinal, id: r.to_string(), referencia: r.to_string(), texto: atual });
                } else if mudou_texto {
                    itens.push(linha(sinal, r, lado(&l.palavras, TipoPalavra::Mais)));
                } else {
                    itens.push(linha(sinal, r, vec![(texto.clone(), false)]));
                }
                if !l.campos.is_empty() {
                    itens.push(nota(tambem_mudou(&l.campos)));
                }
                if l.situacao == "reabre" {
                    itens.push(nota(tr("Plano feito: volta a não feito ao aprovar.").to_string()));
                }
            }
        }
    }
    for det in &d.detalhes_novos {
        let titulo = tr_com("Detalhe: {t}", &[("t", &det.titulo)]);
        itens.push(linha(Sinal::Mais, &format!("{}-{}", acao.referencia, det.id), vec![(titulo, false)]));
    }
    for det in &d.detalhes_mudados {
        let titulo = tr_com("Detalhe refeito: {t}", &[("t", &det.titulo)]);
        itens.push(linha(Sinal::Neutro, &format!("{}-{}", acao.referencia, det.id), vec![(titulo, false)]));
    }
    if !d.outros.is_empty() {
        itens.push(nota(tambem_mudou(&d.outros)));
    }
    itens
}

pub fn galeria_editor(p: &Painter, texto: &str, largura: f32) -> std::sync::Arc<egui::Galley> {
    p.layout_job(LayoutJob::simple(texto.to_string(), FontId::new(TAM, familia(Peso::Normal)), TINTA, largura))
}

pub fn altura_item(p: &Painter, item: &Item, w: f32) -> f32 {
    match item {
        Item::Linha { fileiras, .. } => ALT + (fileiras.len().max(1) - 1) as f32 * PASSO,
        Item::Editor { texto, .. } => (galeria_editor(p, texto, w - X_TEXTO - MARGEM_DIR).size().y + 6.0).max(ALT),
        Item::Nota(fileiras) => fileiras.len().max(1) as f32 * PASSO_NOTA + 7.0,
    }
}
