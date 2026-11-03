//! A folha enquanto o agente escreve (`tela-escrita.mjs`). Não é tela: é a própria folha com `escrita`, e as peças aparecem conforme chegam.
//! A folha chama `acompanhar` a cada quadro, reserva `altura` e desenha `faixa` acima dos projetos, e `etiqueta` em cada linha. A faixa e o esqueleto animado das ações ficam em
//! `escrita_faixa.rs` e `escrita_esqueleto.rs`; a caixa tracejada de «Aguardando plano» é de `linha::barra`, e o brilho passa por cima dela.

use super::modelo::{Acao, Escrita, EstadoEscrita, Folha};
use super::Estado;
use crate::i18n::{tr, tr_com};
use crate::tema::{AMBAR, AZUL, VERDE};
use egui::{Color32, Context, Id};
use std::collections::hash_map::DefaultHasher;
use serde_json::{json, Value};
use std::hash::{Hash, Hasher};

#[path = "escrita_esqueleto.rs"]
mod esqueleto;
#[path = "escrita_faixa.rs"]
mod painel;
pub use esqueleto::etiqueta;
pub use painel::faixa;

pub const TEXTOS: &[(&str, &str)] = &[
    ("Escrita parada há {n} min", "Writing stopped {n} min ago"),
    ("Descartar", "Discard"),
    ("Apaga o que foi escrito nesta versão. A folha não muda.", "Deletes what was written in this version. The sheet does not change."),
    ("O agente está escrevendo…", "The agent is writing…"),
    ("{agente} está escrevendo…", "{agente} is writing…"),
    ("{prontas} de {total} ações com plano", "{prontas} of {total} actions with a plan"),
    ("Escrevendo…", "Writing…"),
    ("Plano pronto", "Plan ready"),
];

/// A altura da faixa; a folha soma o vão que quiser depois dela.
pub const ALTURA: f32 = 68.0;
const MIN_MS: i64 = 60_000;
/// Quanto a folha anda quando uma peça chega, como nos outros gestos que a mexem.
const MEXER_MS: f32 = 350.0;

/// Os minutos da escrita parada; `None` enquanto o servidor não diz que parou. Conta só minutos inteiros já passados.
pub fn minutos_parada(e: &Escrita) -> Option<i64> {
    e.parada.then(|| (e.parada_ms / MIN_MS).max(1))
}

/// O corpo de `gesto/descartar-escrita`.
pub fn corpo_descartar(e: &Escrita) -> Value {
    json!({"escritaId": e.escrita_id})
}

/// A primeira linha da faixa e a cor dela.
pub fn frase(e: &Escrita) -> (String, Color32) {
    match minutos_parada(e) {
        Some(n) => (tr_com("Escrita parada há {n} min", &[("n", &n.to_string())]), AMBAR),
        None if e.agente.is_empty() => (tr("O agente está escrevendo…").to_string(), AZUL),
        None => (tr_com("{agente} está escrevendo…", &[("agente", &e.agente)]), AZUL),
    }
}

/// Quantas ações já têm plano e quantas o blueprint tem. Só as que aguardam ficam de fora.
pub fn contagem(f: &Folha) -> (usize, usize) {
    let (mut prontas, mut total) = (0, 0);
    for a in f.projetos.iter().flat_map(|p| &p.acoes) {
        total += 1;
        prontas += usize::from(a.escrita != EstadoEscrita::Aguardando);
    }
    (prontas, total)
}

/// O que a linha da ação diz durante a escrita. Ação que já estava na folha e não foi tocada fica sem marca: com o plano inteiro «pronto», a marca não diria nada.
pub fn marca(a: &Acao) -> Option<(&'static str, Color32)> {
    match a.escrita {
        EstadoEscrita::Escrevendo => Some((tr("Escrevendo…"), AZUL)),
        EstadoEscrita::Pronta if a.nova => Some((tr("Plano pronto"), VERDE)),
        _ => None,
    }
}

/// A ação em que o agente está agora (`ref`).
pub fn escrevendo(f: &Folha) -> Option<&str> {
    f.projetos.iter().flat_map(|p| &p.acoes).find(|a| a.escrita == EstadoEscrita::Escrevendo).map(|a| a.referencia.as_str())
}

/// O resumo do que já foi escrito: muda quando chega uma peça ou um reenvio, e não com o que anda fora da escrita (progresso, revisão, situação dos agentes).
pub fn assinatura(f: &Folha) -> u64 {
    let mut h = DefaultHasher::new();
    for a in f.projetos.iter().flat_map(|p| &p.acoes) {
        (&a.referencia, &a.nome, &a.texto, a.escrita as u8).hash(&mut h);
        a.planos.iter().for_each(|q| (&q.referencia, &q.texto, &q.instrucoes, &q.area, &q.depende_de).hash(&mut h));
        a.checkpoints.iter().for_each(|k| (&k.id, &k.nome).hash(&mut h));
        a.tabelas.iter().for_each(|t| (&t.titulo, &t.colunas, &t.linhas).hash(&mut h));
        a.detalhes.iter().for_each(|d| (&d.id, &d.titulo, &d.legenda, d.figura.as_ref().map(|x| &x.hash)).hash(&mut h));
    }
    f.decisoes.iter().for_each(|d| (&d.id, &d.rotulo, &d.texto).hash(&mut h));
    f.perguntas.iter().for_each(|q| (&q.id, &q.texto, &q.opcoes).hash(&mut h));
    h.finish()
}

/// O que foi visto no quadro anterior, para perceber o que chegou.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Visto {
    pub assinatura: u64,
    pub escrevendo: Option<String>,
}

/// Se chegou peça desde o quadro anterior. O primeiro olhar não conta: é a folha abrindo, não uma peça chegando.
pub fn chegou_peca(antes: Option<&Visto>, agora: &Visto) -> bool {
    antes.is_some_and(|v| v.assinatura != agora.assinatura)
}

/// A ação a abrir porque o agente passou para ela. O painel acompanha o agente só enquanto o usuário não abriu outra por conta própria, e quem recolheu a ação em curso não a vê reabrir.
pub fn abrir_agora(aberta: Option<&str>, antes: Option<&Visto>, agora: &Visto) -> Option<String> {
    let atual = agora.escrevendo.as_deref()?;
    let anterior = antes.and_then(|v| v.escrevendo.as_deref());
    (anterior != Some(atual) && (aberta.is_none() || aberta == anterior)).then(|| atual.to_string())
}

/// A cada quadro: mexe a folha quando uma peça chega e abre a ação em que o agente está. Sem escrita, esquece o que viu.
/// O que foi visto vai junto da geração da folha (`Folha::geracao`): a mesma folha não muda entre um quadro e outro, então a assinatura só é refeita quando chega outra.
pub fn acompanhar(ctx: &Context, plano: &mut Estado, f: &Folha) {
    let id = Id::unique(("plano-escrita-visto", f.blueprint.id.as_str()));
    if f.escrita.is_none() {
        ctx.data_mut(|d| d.remove::<(u64, Visto)>(id));
        return;
    }
    let guardado = ctx.data(|d| d.get_temp::<(u64, Visto)>(id));
    if f.geracao != 0 && guardado.as_ref().is_some_and(|(g, _)| *g == f.geracao) {
        return;
    }
    let antes = guardado.map(|(_, v)| v);
    let agora = Visto { assinatura: assinatura(f), escrevendo: escrevendo(f).map(str::to_string) };
    if antes.as_ref() == Some(&agora) {
        ctx.data_mut(|d| d.insert_temp(id, (f.geracao, agora)));
        return;
    }
    if let Some(r) = abrir_agora(plano.aberta.as_deref(), antes.as_ref(), &agora) {
        plano.aberta = Some(r);
        plano.aberta_pelo_agente = true;
    }
    if chegou_peca(antes.as_ref(), &agora) {
        crate::anima::mexer(ctx, MEXER_MS);
    }
    ctx.data_mut(|d| d.insert_temp(id, (f.geracao, agora)));
}

/// Quanto a faixa ocupa nesta folha.
pub fn altura(f: &Folha) -> f32 {
    if f.escrita.is_some() {
        ALTURA
    } else {
        0.0
    }
}
