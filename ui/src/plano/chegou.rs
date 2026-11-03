//! «O que chegou» (`design/planejar/tela-proposta.mjs`): a própria folha, com o novo tracejado e o alterado marcado, e ao lado o resumo da versão pendente.
//! Só mostra: nada se aprova aqui. O alterado e o removido levam ao diff, dentro da ação; o novo de `salvar_plano` pode ser colocado na folha.

use super::agentes_movimento as mov;
use super::escala::{Escala, LINHA, MARGEM, X0};
use super::linha::{self, Letra, Pincel};
use super::modelo::{Acao as AcaoPlano, Chegou, Folha, Projeto};
use super::Sub;
use crate::documento::{Documento, RAIZ};
use crate::i18n::{tr, tr_com};
use crate::menu::{self, Alvo};
use crate::planta::{self, Papel};
use crate::telas::{Acao, Tela, Vista};
use crate::tema::{AMBAR, MEDIO, PAPEL, TINTA, VERMELHO};
use egui::{pos2, vec2, Color32, Rect, Sense, Stroke, Ui};
use serde_json::Value;

// o título, a moldura, a legenda, o selo e os botões são os da folha; lá o módulo é privado de `folha.rs`, então entra aqui pelo mesmo arquivo, como em `detalhe.rs`
#[path = "folha_topo.rs"]
#[allow(dead_code)]
mod cabeca;
use cabeca::{botoes, legenda, moldura, selo, titulo, Cabeca, Topo};

#[path = "chegou_cartao.rs"]
mod cartao;
#[path = "chegou_painel.rs"]
mod lista;
#[path = "chegou_regua.rs"]
mod regua_dias;
use lista::{altura_painel, corpo_colocar, painel, secoes, Oferta};

pub const TEXTOS: &[(&str, &str)] = &[
    ("← Folha", "← Sheet"),
    ("O plano mudou", "The plan changed"),
    ("Nada chegou", "Nothing arrived"),
    ("Nenhuma versão à sua espera.", "No version is waiting for you."),
    ("novo", "new"),
    ("novos", "new"),
    ("alterados", "changed"),
    ("igual", "unchanged"),
    ("iguais", "unchanged"),
    ("removido", "removed"),
    ("removidos", "removed"),
    ("Planos alterados", "Changed plans"),
    ("Planos novos", "New plans"),
    ("Planos removidos", "Removed plans"),
    ("Ver diff", "See diff"),
    ("Colocar", "Place"),
    ("Novo plano", "New plan"),
    ("Alterado", "Changed"),
    ("Removido", "Removed"),
    ("{n} plano", "{n} plan"),
    ("{n} planos", "{n} plans"),
    ("{n} detalhe", "{n} detail"),
    ("{n} detalhes", "{n} details"),
    ("{n} checkpoint", "{n} checkpoint"),
    ("{n} checkpoints", "{n} checkpoints"),
    ("ainda sem planos", "no plans yet"),
    ("+{n} det.", "+{n} det."),
    ("{n} outros", "{n} others"),
    ("Sai do plano ao aprovar", "Leaves the plan on approval"),
    ("O agente", "The agent"),
    ("Versão {n}", "Version {n}"),
    ("enviado por {agente}", "sent by {agente}"),
];

const LARGURA_MIN: f32 = 960.0;
const ALTURA_MIN: f32 = 560.0;
const Y_REGUA: f32 = 138.0;
const Y_INICIO: f32 = 182.0;
const Y_PAINEL: f32 = 112.0;
/// O painel fica colado à direita, com a largura do protótipo (`X1 - XP` sobre 1100); a régua vai de `X_REGUA` até perto dele.
const LARGO_PAINEL: f32 = 344.0;
const X_REGUA: f32 = 330.0;
const VAO_PAINEL: f32 = 20.0;
const X_ID: f32 = X0 + 100.0;
const X_NOME: f32 = X_ID + 40.0;

/// A etiqueta na linha da ação.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marca {
    Nova,
    Alterada,
    Removida,
}

pub fn marca_de(c: &Chegou, referencia: &str) -> Option<Marca> {
    if c.novas.iter().any(|n| n.referencia == referencia) {
        Some(Marca::Nova)
    } else if c.alteradas.iter().any(|n| n.referencia == referencia) {
        Some(Marca::Alterada)
    } else if c.removidas.iter().any(|n| n.referencia == referencia) {
        Some(Marca::Removida)
    } else {
        None
    }
}

/// Até onde a coluna da esquerda vai.
fn fim_esquerda(f: &Folha) -> f32 {
    f.projetos.iter().fold(Y_INICIO, |y, p| y + 32.0 + p.acoes.len() as f32 * LINHA + 24.0)
}

/// O que a tela pede, depois de a folha voltar ao estado.
enum Saida {
    Voltar,
    Oferta(Oferta, String),
}

pub fn tela(ui: &mut Ui, vista: &mut Vista, doc: &Documento, ws: &str, bp: &str, servidor: &str) -> Option<Acao> {
    let Some(w) = doc.workspace(ws) else {
        vista.tela = Tela::Entrada;
        return None;
    };
    let lista = w.get("blueprints").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let Some(i) = lista.iter().position(|b| b.get("id").and_then(Value::as_str) == Some(bp)) else {
        vista.tela = Tela::Workspace(ws.to_string());
        return None;
    };
    let workspace = w.get(if ws == RAIZ { "empresa" } else { "nome" }).and_then(Value::as_str).unwrap_or("").to_string();
    let k = Cabeca { workspace, folha: i + 2, folhas: lista.len() + 1 };
    super::agentes::demo::aplicar_chegou(&mut vista.plano);
    // a folha e o resumo saem do estado enquanto desenham, sem copiá-los a cada quadro
    let f = vista.plano.folha.take()?;
    let chegou = vista.plano.chegou.take();
    let (pedido, saida) = desenhar(ui, &f, chegou.as_ref(), &vista.plano.aviso, &k, ws, bp, servidor);
    if let Some(s) = saida {
        match s {
            Saida::Voltar => vista.plano.tela = Sub::Folha,
            Saida::Oferta(Oferta::VerDiff, r) => {
                // a própria folha abre a ação e rola até ela; ação que não esteja mais lá só abre, sem rolar
                vista.ir_para = f.projetos.iter().flat_map(|p| &p.acoes).find(|a| a.referencia == r).map(|a| a.id.clone());
                vista.plano.aberta = Some(r);
                vista.plano.tela = Sub::Folha;
            }
            Saida::Oferta(Oferta::Colocar, r) => {
                let numero = chegou.as_ref().map_or(0, |c| c.numero);
                vista.plano.gesto(ui.ctx(), servidor, "colocar", &corpo_colocar(ws, bp, numero, &r));
            }
        }
        ui.ctx().request_repaint();
    }
    vista.plano.folha = Some(f);
    vista.plano.chegou = chegou;
    pedido
}

#[allow(clippy::too_many_arguments)]
fn desenhar(ui: &mut Ui, f: &Folha, chegou: Option<&Chegou>, aviso: &str, k: &Cabeca, ws: &str, bp: &str, servidor: &str) -> (Option<Acao>, Option<Saida>) {
    let (alto, w) = (ui.available_height(), ui.available_width().max(LARGURA_MIN));
    let vazio = Chegou::default();
    let dados = chegou.unwrap_or(&vazio);
    let tem = dados.numero != 0 || !secoes(dados).is_empty();
    let (mut pedido, mut saida) = (None, None);
    mov::abrir(ui.ctx(), "chegou");
    crate::rolagem::mostrar(ui, egui::ScrollArea::both().id_salt("plano-chegou").auto_shrink([false, false]), |ui| {
        let h = alto.max(ALTURA_MIN).max(fim_esquerda(f) + 120.0).max(Y_PAINEL + altura_painel(dados, tem) + 110.0);
        let (pagina, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
        let painter = ui.painter().clone();
        planta::papel(&painter, pagina, Papel::Entrada);
        menu::marcar(ui, pagina, || Alvo::Folha { ws: ws.into(), bp: bp.into(), nome: f.blueprint.nome.clone(), chaves: Vec::new(), aberto: false });
        let c = Pincel { p: &painter, o: pagina.min };
        let x1 = w - MARGEM;
        let xp = x1 - LARGO_PAINEL;
        let e = Escala::de_folha(f, X_REGUA, xp - VAO_PAINEL);
        let recorte = Rect::from_min_max(pos2(pagina.min.x + e.x0, pagina.min.y), pos2(pagina.min.x + e.x1, pagina.max.y)).intersect(painter.clip_rect());
        let pintor_regua = painter.with_clip_rect(recorte);
        let r = Pincel { p: &pintor_regua, o: pagina.min };
        moldura(&c, w, h);
        pedido = titulo(ui, &c, f, k, ws, bp);
        escrever_aviso(ui, &c, aviso, xp);
        regua_dias::regua(&c, &e);
        linhas(ui, &c, &r, &e, f, dados);
        saida = painel(ui, &c, dados, chegou.is_some(), tem, xp, x1, h).map(|(o, r)| Saida::Oferta(o, r));
        legenda(&c, h);
        selo(&c, x1, h, &[
            (tr("Workspace"), k.workspace.clone(), 140.0),
            (tr("Blueprint"), f.blueprint.nome.clone(), 210.0),
            (tr("Folha"), tr_com("{n} de {total}", &[("n", &k.folha.to_string()), ("total", &k.folhas.to_string())]), 110.0),
        ]);
        let mut topo = vec![(Topo::Voltar, tr("← Folha").to_string())];
        topo.extend(["Planejar", "Compartilhar", "Apresentar"].into_iter().map(|t| (Topo::Web, tr(t).to_string())));
        match botoes(ui, &c, x1, &topo) {
            Some(Topo::Voltar) => saida = Some(Saida::Voltar),
            Some(_) => ui.ctx().open_url(egui::OpenUrl::new_tab(format!("{servidor}/#/c/{ws}"))),
            None => {}
        }
    });
    (pedido, saida)
}

/// O que o último gesto respondeu: a recusa de «Colocar» tem de aparecer aqui também.
fn escrever_aviso(ui: &Ui, c: &Pincel, aviso: &str, xp: f32) {
    if aviso.is_empty() {
        return;
    }
    let letra = Letra::nova(10.5, AMBAR);
    let (t, cortou) = c.cortado(aviso, xp - VAO_PAINEL - X0, &letra);
    let largura = c.texto(&t, X0, 104.0, letra);
    if cortou {
        let _ = ui.interact(c.caixa(X0, 93.0, largura, 16.0), ui.scope_id().with("plano-chegou-aviso"), Sense::hover()).on_hover_text(aviso);
    }
}

/// Os projetos e as ações à esquerda, só para ler: a etiqueta, o id, o nome e a barra.
fn linhas(ui: &Ui, c: &Pincel, r: &Pincel, e: &Escala, f: &Folha, dados: &Chegou) {
    let (mut y, visivel) = (Y_INICIO, ui.clip_rect());
    for (n, p) in f.projetos.iter().enumerate() {
        cabecalho(c, r, e, n + 1, p, &f.blueprint.nome, y);
        y += 32.0;
        for a in &p.acoes {
            if visivel.intersects(c.caixa(0.0, y - LINHA, e.x1 + 1.0, LINHA * 2.0)) {
                linha_acao(ui, c, r, e, a, marca_de(dados, &a.referencia), y);
                linha::separador(c, e, y + LINHA / 2.0);
            }
            y += LINHA;
        }
        y += 24.0;
    }
}

/// `linha::cabecalho`, sem a soma das colunas: o espaço delas é da régua.
fn cabecalho(c: &Pincel, r: &Pincel, e: &Escala, n: usize, p: &Projeto, nome_padrao: &str, y: f32) {
    c.p.circle(c.pt(X0 + 11.0, y - 4.0), 11.0, PAPEL, Stroke::new(1.0, TINTA));
    c.texto(&format!("{n:02}"), X0 + 11.0, y, Letra::nova(9.5, TINTA).forte().meio());
    let letra = Letra::nova(12.0, TINTA).forte();
    let nome = if p.nome.is_empty() { nome_padrao } else { &p.nome };
    c.texto(&c.cortado(nome, e.x0 - X0 - 36.0, &letra).0, X0 + 30.0, y, letra);
    // o trecho do projeto na escada, da primeira à última ação dele
    let trechos = p.acoes.first().and_then(|a| e.trecho(a)).zip(p.acoes.last().and_then(|a| e.trecho(a)));
    if let Some(((xa, _), (_, xb))) = trechos {
        r.fio(xa, y - 4.0, xb, y - 4.0, TINTA, 2.0);
        r.fio(xa, y - 8.0, xa, y, TINTA, 2.0);
        r.fio(xb, y - 8.0, xb, y, TINTA, 2.0);
    }
    c.fio(X0, y + 11.0, e.x1, y + 11.0, TINTA.gamma_multiply(0.7), 1.0);
}

fn linha_acao(ui: &Ui, c: &Pincel, r: &Pincel, e: &Escala, a: &AcaoPlano, marca: Option<Marca>, y: f32) {
    if let Some(m) = marca {
        // a faixa de cor leve que põe a linha em evidência, e a etiqueta por cima
        let tom = tom_da_marca(m);
        let t = mov::chegada(ui.ctx(), ("chegou-linha", a.id.as_str()), 0.5);
        c.ret(X0, y - LINHA / 2.0, e.x1 - X0, LINHA, Some(tom.gamma_multiply(0.07 * t)), None);
        c.ret(X0, y - LINHA / 2.0, 2.0, LINHA, Some(tom.gamma_multiply(0.85 * t)), None);
        etiqueta(c, X0 + 8.0, y + 4.0, m);
    }
    c.texto(&a.referencia, X_ID, y + 4.0, Letra::nova(11.0, MEDIO));
    let letra = Letra::nova(12.0, TINTA);
    let (nome, cortou) = c.cortado(&a.nome, e.x0 - X_NOME - 12.0, &letra);
    let largura = c.texto(&nome, X_NOME, y + 4.0, letra);
    if cortou {
        let _ = ui.interact(c.caixa(X_NOME, y - 8.0, largura, 18.0), ui.scope_id().with(("plano-chegou-nome", a.id.as_str())), Sense::hover()).on_hover_text(a.nome.as_str());
    }
    linha::barra(r, e, a, y);
}

/// A cor de cada marca: o novo é tinta, o alterado âmbar e o removido vermelho, as mesmas dos cartões do painel.
fn tom_da_marca(m: Marca) -> Color32 {
    match m {
        Marca::Nova => TINTA,
        Marca::Alterada => AMBAR,
        Marca::Removida => VERMELHO,
    }
}

/// `etiqueta` do protótipo, com a linha de base em `y`: o novo é cheio; o alterado e o removido, só o contorno.
fn etiqueta(c: &Pincel, x: f32, y: f32, m: Marca) {
    let (texto, cor, cheia) = match m {
        Marca::Nova => (tr("Novo plano"), tom_da_marca(m), true),
        Marca::Alterada => (tr("Alterado"), tom_da_marca(m), false),
        Marca::Removida => (tr("Removido"), tom_da_marca(m), false),
    };
    let letra = Letra::nova(9.0, if cheia { PAPEL } else { cor }).forte().trilha(0.1);
    let w = (c.largura(&texto.to_uppercase(), &letra) + 12.0).round();
    c.ret(x, y - 10.0, w, 14.0, cheia.then_some(cor), Some((cor, 1.0)));
    c.rotulo(texto, x + 6.0, y, letra);
}
