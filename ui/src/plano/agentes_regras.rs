//! As regras da tela de agentes, sem desenho: a figura e o rótulo de cada situação, o ramo, o limite, os botões da linha e os grupos por ação. Os textos ficam em `agentes::TEXTOS`.

use super::super::modelo::{PlanoAgente, Situacao};
use super::super::orquestrador::Pedido;
use crate::i18n::{tr, tr_com};
use crate::tema::{AMBAR, AZUL, MEDIO, VERDE, VERMELHO};
use egui::Color32;
use serde_json::{json, Value};

const LIMITE_MIN: u32 = 1;
const LIMITE_MAX: u32 = 16;

/// A figura de cada situação (`SITUACOES` de `agentes.mjs`). As poses são as dos agentes de plano; a `principal` é do orquestrador.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    Parado,
    Trabalhando,
    Esperando,
    Bloqueado,
    Concluido,
}

pub fn pose(s: Situacao) -> Pose {
    match s {
        Situacao::Nao | Situacao::Outra => Pose::Parado,
        Situacao::Trabalhando => Pose::Trabalhando,
        Situacao::Revisao | Situacao::Fila | Situacao::Dependencia | Situacao::Area => Pose::Esperando,
        Situacao::Parado | Situacao::Bloqueado => Pose::Bloqueado,
        Situacao::Pronto => Pose::Concluido,
    }
}

/// O que a coluna Situação escreve, e em que cor. Sem sinal só avisa: a situação continua a mesma.
pub fn rotulo_situacao(q: &PlanoAgente) -> (String, Color32) {
    let (rotulo, cor) = match q.situacao {
        Situacao::Nao => (tr("Não iniciado"), MEDIO),
        Situacao::Revisao => (tr("Revise a mudança"), AMBAR),
        Situacao::Trabalhando => (tr("Trabalhando"), AZUL),
        Situacao::Fila => (tr("Na fila"), MEDIO),
        Situacao::Dependencia => (tr("Esperando plano"), AMBAR),
        Situacao::Area => (tr("Esperando área"), AMBAR),
        Situacao::Parado => (tr("Parado"), VERMELHO),
        Situacao::Bloqueado => match q.causa.as_str() {
            "sumiu" => (tr("O agente sumiu"), VERMELHO),
            "falhou" => (tr("O agente falhou"), VERMELHO),
            _ => (tr("Pediu ajuda"), VERMELHO),
        },
        Situacao::Pronto => (tr("Pronto"), VERDE),
        Situacao::Outra => ("—", MEDIO),
    };
    let fim = if q.sem_sinal { tr(" · sem sinal") } else { "" };
    (format!("{rotulo}{fim}"), cor)
}

/// O motivo vem do servidor como código; o que não é conhecido aparece como veio.
fn motivo_escrito(motivo: &str) -> String {
    match motivo {
        "terminou-sem-concluir" => tr("Terminou sem concluir o plano."),
        "base-em-conflito" => tr("Os ramos de que este plano parte conflitam entre si."),
        "repositorio-sem-commit" => tr("O repositório ainda não tem nenhum commit."),
        "plano-removido" => tr("O plano saiu do blueprint."),
        outro => outro,
    }
    .to_string()
}

/// O que a dica da situação conta além do rótulo: o motivo, por quem espera a área, o sinal e os subagentes.
pub fn dica_situacao(q: &PlanoAgente) -> Option<String> {
    let mut partes = Vec::new();
    if !q.motivo.is_empty() {
        partes.push(motivo_escrito(&q.motivo));
    }
    if q.situacao == Situacao::Area && !q.por.is_empty() {
        partes.push(tr_com("Espera {por}.", &[("por", &q.por)]));
    }
    if q.sem_sinal {
        partes.push(tr("Sem sinal do agente há mais de 2 minutos.").to_string());
    }
    if q.subagentes > 0 {
        partes.push(if q.subagentes == 1 { tr("1 subagente trabalhando.").to_string() } else { tr_com("{n} subagentes trabalhando.", &[("n", &q.subagentes.to_string())]) });
    }
    (!partes.is_empty()).then(|| partes.join("\n"))
}

/// De quem depende: quem está segurando, se está esperando alguém (com destaque); senão, a lista.
pub fn dependencia(q: &PlanoAgente) -> (String, bool) {
    if matches!(q.situacao, Situacao::Dependencia | Situacao::Parado) && !q.por.is_empty() {
        (q.por.clone(), true)
    } else {
        (q.depende_de.join(", "), false)
    }
}

/// O ramo só aparece em quem trabalha nele ou terminou nele.
pub fn ramo_visivel(q: &PlanoAgente) -> Option<&str> {
    (matches!(q.situacao, Situacao::Trabalhando | Situacao::Pronto) && !q.ramo.is_empty()).then_some(q.ramo.as_str())
}

/// O limite depois de − ou +; `None` se já está na ponta.
pub fn limite_novo(limite: u32, delta: i32) -> Option<u32> {
    let novo = i64::from(limite) + i64::from(delta);
    (i64::from(LIMITE_MIN)..=i64::from(LIMITE_MAX)).contains(&novo).then_some(novo as u32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BotaoLinha {
    Integrar,
    Retomar,
    Cancelar,
}

/// O que a linha oferece, da esquerda para a direita: quem decide é o servidor.
pub fn botoes_da_linha(q: &PlanoAgente) -> Vec<BotaoLinha> {
    [(q.pode_integrar, BotaoLinha::Integrar), (q.pode_retomar, BotaoLinha::Retomar), (q.pode_cancelar, BotaoLinha::Cancelar)].into_iter().filter(|(pode, _)| *pode).map(|(_, b)| b).collect()
}

pub fn rotulo_botao(b: BotaoLinha) -> &'static str {
    match b {
        BotaoLinha::Integrar => "Integrar",
        BotaoLinha::Retomar => "Retomar",
        BotaoLinha::Cancelar => "Cancelar",
    }
}

pub fn dica_botao(b: BotaoLinha, q: &PlanoAgente) -> Option<String> {
    match b {
        BotaoLinha::Integrar if q.destino.is_empty() => Some(tr("A pasta principal não está em nenhum ramo.").to_string()),
        BotaoLinha::Integrar => Some(tr_com("Juntar {ramo} em {destino}.", &[("ramo", &q.ramo), ("destino", &q.destino)])),
        _ => None,
    }
}

/// O gesto de cada botão. `ramoEsperado` é o ramo que a tela mostrou: se a pasta principal mudou de ramo, o servidor recusa.
pub fn corpo_botao(b: BotaoLinha, q: &PlanoAgente) -> Pedido {
    match b {
        BotaoLinha::Integrar => ("integrar", json!({"execucaoId": q.execucao_id, "ramoEsperado": q.destino})),
        BotaoLinha::Retomar => ("retomar", json!({"execucaoId": q.execucao_id})),
        BotaoLinha::Cancelar => ("cancelar", json!({"execucaoId": q.execucao_id})),
    }
}

/// Os planos de uma ação. O servidor lista de todos os blueprints; o título vem do documento e diz de qual blueprint é, se não for o aberto.
pub struct Grupo<'a> {
    pub acao_id: &'a str,
    pub titulo: String,
    pub planos: Vec<&'a PlanoAgente>,
}

fn titulo_do_grupo(w: &Value, q: &PlanoAgente, bp: &str) -> String {
    let campo = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let blueprints = w.get("blueprints").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let acao = blueprints.iter().find(|b| campo(b, "id") == q.blueprint_id).and_then(|b| b.get("acoes")).and_then(Value::as_array).and_then(|l| l.iter().find(|a| campo(a, "id") == q.acao_id));
    let nome = acao.map_or_else(String::new, |a| format!("{} {}", campo(a, "ref"), campo(a, "nome")).trim().to_string());
    match (q.blueprint_id == bp, nome.is_empty()) {
        (true, false) => nome,
        (false, false) => format!("{} · {nome}", q.blueprint_nome),
        (_, true) => q.blueprint_nome.clone(),
    }
}

pub fn agrupar<'a>(planos: &'a [PlanoAgente], w: &Value, bp: &str) -> Vec<Grupo<'a>> {
    let mut grupos: Vec<Grupo<'a>> = Vec::new();
    for q in planos {
        match grupos.iter_mut().find(|g| g.acao_id == q.acao_id) {
            Some(g) => g.planos.push(q),
            None => grupos.push(Grupo { acao_id: &q.acao_id, titulo: titulo_do_grupo(w, q, bp), planos: vec![q] }),
        }
    }
    grupos
}

impl Grupo<'_> {
    /// Quantos planos da ação já estão prontos e quantos ela tem.
    pub fn prontos(&self) -> (usize, usize) {
        (self.planos.iter().filter(|q| q.situacao == Situacao::Pronto).count(), self.planos.len())
    }
}
