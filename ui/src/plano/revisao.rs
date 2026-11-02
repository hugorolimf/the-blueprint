//! Revisar faz parte da folha: na linha da peça, os três botões; depois de decidida, o resultado com desfazer.
//! A observação só existe em Revisar. O clique vai amarrado a `numero` e `hash` do que estava na tela; o estado fica no servidor, e o rascunho, na memória do egui.
//! Quem usa: `Peca::decisao`/`detalhe`/`diff`, `altura_nota` antes de empilhar, `canto` na linha da peça e `nota` embaixo dela.

use super::caixinha::Lapis;
use super::modelo::{Decisao, Detalhe, Diff, EstadoRevisao, Folha, Revisao};
use super::Estado;
use crate::i18n::tr;
use crate::tema::{AMBAR, MEDIO, TINTA, VERDE, VERMELHO};
use egui::{Color32, Ui};
use serde_json::{json, Value};

#[path = "revisao_nota.rs"]
mod observacao;
pub use super::cartao_carimbo::{carimbo, desfazer, risco, situacao};
pub use super::cartao_controle::{controle, medir_partes};
pub use observacao::{altura_nota, compondo, nota};

pub const TEXTOS: &[(&str, &str)] = &[
    ("Aguardando você", "Waiting for you"),
    ("Aprovado", "Approved"),
    ("Em revisão", "In review"),
    ("Reprovado", "Rejected"),
    ("Aprovar", "Approve"),
    ("Revisar", "Review"),
    ("Editar", "Edit"),
    ("Reprovar", "Reject"),
    ("desfazer", "undo"),
    ("Diga o que mudar…", "Say what to change…"),
    ("Enviar", "Send"),
    ("Cancelar", "Cancel"),
];

/// Como a peça está para o usuário. Sem revisão valendo (nenhuma, ou o `hash` mudou e o servidor devolveu `null`): pendente.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EstadoPeca {
    Pendente,
    Aprovada,
    Revisar,
    Reprovada,
}

pub fn estado_de(revisao: Option<&Revisao>) -> EstadoPeca {
    match revisao.map(|r| r.estado) {
        Some(EstadoRevisao::Aprovada) => EstadoPeca::Aprovada,
        Some(EstadoRevisao::Revisar) => EstadoPeca::Revisar,
        Some(EstadoRevisao::Reprovada) => EstadoPeca::Reprovada,
        // `respondida` é só de pergunta, que não tem este controle
        Some(EstadoRevisao::Respondida) | None => EstadoPeca::Pendente,
    }
}

pub fn rotulo(e: EstadoPeca) -> (&'static str, Color32) {
    match e {
        EstadoPeca::Pendente => (tr("Aguardando você"), AMBAR),
        EstadoPeca::Aprovada => (tr("Aprovado"), VERDE),
        EstadoPeca::Revisar => (tr("Em revisão"), AMBAR),
        EstadoPeca::Reprovada => (tr("Reprovado"), VERMELHO),
    }
}

/// A cor do corpo da peça: aprovada e reprovada saem do primeiro plano, mas continuam à vista.
pub fn cor_corpo(e: EstadoPeca) -> Color32 {
    if matches!(e, EstadoPeca::Aprovada | EstadoPeca::Reprovada) {
        MEDIO
    } else {
        TINTA
    }
}

pub struct Peca<'a> {
    pub chave: String,
    pub numero: i64,
    pub hash: &'a str,
    pub revisao: Option<&'a Revisao>,
    /// os textos de «Editar», que vão junto ao aprovar um diff
    pub textos: Option<Value>,
}

fn numero_da_folha(folha: &Folha) -> i64 {
    folha.pendente.as_ref().map_or(0, |p| p.numero)
}

impl<'a> Peca<'a> {
    pub fn decisao(folha: &Folha, d: &'a Decisao) -> Self {
        Peca { chave: format!("d:{}", d.id), numero: numero_da_folha(folha), hash: &d.hash, revisao: d.revisao.as_ref(), textos: None }
    }
    /// `acao` é o `ref` da ação.
    pub fn detalhe(folha: &Folha, acao: &str, d: &'a Detalhe) -> Self {
        Peca { chave: format!("det:{acao}/{}", d.id), numero: numero_da_folha(folha), hash: &d.hash, revisao: d.revisao.as_ref(), textos: None }
    }
    /// `acao` é o `ref` da ação; o número é o da versão que o diff mostra.
    pub fn diff(acao: &str, d: &'a Diff) -> Self {
        Peca { chave: format!("diff:{acao}"), numero: d.numero, hash: &d.hash, revisao: d.revisao.as_ref(), textos: None }
    }
    pub fn estado(&self) -> EstadoPeca {
        estado_de(self.revisao)
    }
}

/// Para onde o gesto vai. `Copy`, para passar junto com a folha, que é emprestada do mesmo `Estado`.
#[derive(Clone, Copy)]
pub struct Alvo<'a> {
    pub estado: &'a Estado,
    pub servidor: &'a str,
    pub ws: &'a str,
    pub bp: &'a str,
}

/// O corpo de `gesto/revisar`. `estado`: `aprovada`, `revisar`, `reprovada` ou `pendente` (desfazer). A observação só vale em `revisar`.
pub fn corpo(ws: &str, bp: &str, peca: &Peca, estado: &str, observacao: &str) -> Value {
    let mut c = json!({"workspaceId": ws, "blueprintId": bp, "peca": peca.chave, "numero": peca.numero, "hash": peca.hash, "estado": estado});
    if estado == "revisar" {
        c["observacao"] = json!(observacao.trim());
    }
    if let (Some(textos), "aprovada") = (&peca.textos, estado) {
        c["textos"] = textos.clone();
    }
    c
}

fn enviar(ctx: &egui::Context, alvo: Alvo, peca: &Peca, estado: &str, observacao: &str) {
    alvo.estado.gesto(ctx, alvo.servidor, "revisar", &corpo(alvo.ws, alvo.bp, peca, estado, observacao));
}

/// A parte do meio: «Revisar» numa peça comum, «Editar» num plano alterado (com `true` enquanto a edição está aberta).
#[derive(Clone, Copy, PartialEq)]
pub enum Meio {
    Revisar,
    Editar(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parte {
    Aprovar,
    Meio,
    Reprovar,
}

/// O canto direito da peça: o controle enquanto espera você, o carimbo com desfazer depois. `y` é a linha de base do título da peça, como `canto` do protótipo. Devolve `true` se pediu «Editar», que fica com quem chamou.
pub fn canto(l: Lapis, ui: &mut Ui, alvo: Alvo, x_fim: f32, y: f32, peca: &Peca, meio: Meio) -> bool {
    let ctx = ui.ctx().clone();
    let estado = peca.estado();
    if estado != EstadoPeca::Pendente {
        let x0 = carimbo(l, x_fim, y, estado, peca.revisao.map_or(0, |r| r.em));
        if desfazer(l, ui, &peca.chave, x0 - 12.0, y) {
            enviar(&ctx, alvo, peca, "pendente", "");
        }
        return false;
    }
    let escrevendo = compondo(&ctx, &peca.chave);
    let ativo = escrevendo || meio == Meio::Editar(true);
    match controle(l, ui, x_fim, y - 16.0, meio, ativo) {
        Some(Parte::Aprovar) => enviar(&ctx, alvo, peca, "aprovada", ""),
        Some(Parte::Reprovar) => enviar(&ctx, alvo, peca, "reprovada", ""),
        Some(Parte::Meio) if meio != Meio::Revisar => return true,
        Some(Parte::Meio) => observacao::alternar(&ctx, &peca.chave, !escrevendo),
        None => {}
    }
    false
}
