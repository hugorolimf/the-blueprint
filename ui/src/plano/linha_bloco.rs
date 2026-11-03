//! O bloco de uma ação na folha, em cima do passo dela na escada: a miniatura da obra que o agente planejou e, ao lado, o número, o nome, uma linha do que ela é e as peças, cada uma com a sua cor. O clique numa peça abre a ação e voa até ela.
//! As esteiras entre as ações moram em `linha_esteiras.rs`. Sem datas: a posição é só a ordem.

use super::{situacao, Letra, Pincel};
use crate::dominio::Situacao;
use crate::i18n::{tr, tr_com};
use crate::plano::acao;
use crate::plano::escala::{Escala, LINHA_FOLHA};
use crate::plano::folha::estilo::{com_alfa, pulso, ANDANDO, ESPERA};
use crate::plano::modelo::{Acao, EstadoEscrita, Folha};
use crate::tema::{DESTAQUE, MEDIO, TINTA};
use egui::{Color32, Id, Ui};
use std::collections::HashMap;
use std::sync::Arc;

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

pub const W_MINI: f32 = 170.0;
pub const ALTO_MINI: f32 = 98.0;
const VAO: f32 = 14.0;
/// Onde o texto começa, a partir do começo do passo.
pub const X_TEXTO: f32 = W_MINI + VAO;
/// O meio da fileira de peças, a partir do meio da linha.
pub const DESCE_PECAS: f32 = 24.0;
const SOBE_NOME: f32 = 20.0;
const LARGURA_MAX: f32 = 620.0;

/// A primeira frase da ação, sem quebra de linha: é o que cabe numa linha só.
fn resumo(a: &Acao) -> &str {
    let t = a.texto.lines().next().unwrap_or("").trim();
    t.split_inclusive(". ").next().unwrap_or(t).trim()
}

/// O que o texto da linha ocupa: o começo e a largura, depois da miniatura, ou antes dela no fim da escada. `xa` é o começo do passo em que foi medido.
struct Medido {
    xa: f32,
    xt: f32,
    w: f32,
    nome: String,
    frase: String,
    w_nome: f32,
    aguarda: bool,
}

/// A folha, a escada e a escala da tela em que os textos foram medidos.
type Chave = (u64, u32, u32, usize, u32);

/// Os textos medidos de cada ação: as linhas e as esteiras medem as mesmas ações a cada quadro, e o resultado só muda com a folha, a escada ou a escala.
#[derive(Clone, Default)]
struct Medidos {
    chave: Chave,
    por_acao: HashMap<String, Arc<Medido>>,
}

fn com_medidos<R>(p: &Pincel, chave: Chave, faz: impl FnOnce(&mut HashMap<String, Arc<Medido>>) -> R) -> R {
    p.p.ctx().data_mut(|d| {
        let m = d.get_temp_mut_or_default::<Medidos>(Id::unique("plano-linha-medidas"));
        if m.chave != chave {
            *m = Medidos { chave, ..Medidos::default() };
        }
        faz(&mut m.por_acao)
    })
}

/// `medir`, uma vez por folha: a conta roda fora da trava do contexto. Folha sem geração não guarda.
fn medido(p: &Pincel, f: &Folha, e: &Escala, a: &Acao, xa: f32) -> Arc<Medido> {
    if f.geracao == 0 {
        return Arc::new(medir(p, f, e, a, xa));
    }
    let chave = (f.geracao, e.x0.to_bits(), e.x1.to_bits(), e.total(), p.p.pixels_per_point().to_bits());
    if let Some(m) = com_medidos(p, chave, |m| m.get(&a.id).filter(|m| m.xa == xa).cloned()) {
        return m;
    }
    let m = Arc::new(medir(p, f, e, a, xa));
    com_medidos(p, chave, |por| por.insert(a.id.clone(), m.clone()));
    m
}

fn letra_do_nome(luz: f32) -> Letra {
    Letra::nova(13.0, if luz > 0.5 { Color32::WHITE } else { TINTA })
}

fn letra_do_resumo() -> Letra {
    Letra::nova(11.0, MEDIO)
}

fn medir(p: &Pincel, f: &Folha, e: &Escala, a: &Acao, xa: f32) -> Medido {
    let max = LARGURA_MAX.min(e.x1 - e.x0 - X_TEXTO);
    let extras = if a.diff.is_some() { 16.0 } else { 0.0 } + if a.agentes > 0 { 44.0 } else { 0.0 };
    let nome = p.cortado(&format!("{} · {}", a.referencia, a.nome).to_uppercase(), max - extras, &letra_do_nome(0.0)).0;
    let frase = p.cortado(resumo(a), max, &letra_do_resumo()).0;
    let w_nome = p.largura(&nome, &letra_do_nome(0.0));
    let aguarda = a.escrita == EstadoEscrita::Aguardando;
    let w_pecas = if aguarda { 0.0 } else { acao::largura_pecas(p, f, a) };
    let w = (w_nome + extras).max(p.largura(&frase, &letra_do_resumo())).max(w_pecas);
    let xt = if xa + X_TEXTO + w <= e.x1 { xa + X_TEXTO } else { (xa - VAO - w).max(e.x0) };
    Medido { xa, xt, w, nome, frase, w_nome, aguarda }
}

/// De onde a onde o texto da ação vai, na horizontal: as esteiras passam por baixo dele.
pub fn faixa_do_texto(p: &Pincel, f: &Folha, e: &Escala, a: &Acao) -> Option<(f32, f32)> {
    let (xa, _) = e.trecho(a)?;
    let m = medido(p, f, e, a, xa);
    Some((m.xt, m.xt + m.w))
}

/// A linha com o meio em `y`. `luz` é o mouse em cima (0 a 1). Devolve se clicaram numa peça.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, p: &Pincel, f: &Folha, e: &Escala, a: &Acao, servidor: &str, y: f32, luz: f32) -> bool {
    let Some((xa, _)) = e.trecho(a) else { return false };
    let topo = y - LINHA_FOLHA / 2.0;
    if luz > 0.0 {
        p.ret(e.x0, topo, e.x1 - e.x0, LINHA_FOLHA, Some(com_alfa(TINTA, 0.05 * luz)), None);
    }
    acao::miniatura(p, f, a, servidor, p.caixa(xa, y - ALTO_MINI / 2.0, W_MINI, ALTO_MINI), luz);
    let m = medido(p, f, e, a, xa);
    // o ponto de início, ou o ponto em que a obra está, leva a cor: a miniatura acende o lote, e o nome acende junto
    let nome = if acao::tem_o_ponto(f, a) { Letra::nova(13.0, DESTAQUE) } else { letra_do_nome(luz) };
    p.texto(&m.nome, m.xt, y - SOBE_NOME, nome);
    let mut x_extra = m.xt + m.w_nome + 8.0;
    if a.diff.is_some() {
        p.p.circle_filled(p.pt(x_extra + 3.0, y - SOBE_NOME - 4.0), 3.2, ESPERA);
        x_extra += 16.0;
    }
    if a.agentes > 0 {
        p.p.circle_filled(p.pt(x_extra + 3.0, y - SOBE_NOME - 4.0), 2.6 + pulso(p.p.ctx(), 1.8), ANDANDO);
        p.texto(&tr_com("×{n}", &[("n", &a.agentes.to_string())]), x_extra + 10.0, y - SOBE_NOME, Letra::nova(11.5, ANDANDO).forte());
    }
    p.texto(&m.frase, m.xt, y + 1.0, letra_do_resumo());
    if m.aguarda {
        p.texto(&tr("Aguardando plano").to_uppercase(), m.xt + 104.0, y + DESCE_PECAS + 4.0, Letra::nova(11.5, ESPERA));
        return false;
    }
    acao::pecas(ui, p, f, a, m.xt, y + DESCE_PECAS)
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
