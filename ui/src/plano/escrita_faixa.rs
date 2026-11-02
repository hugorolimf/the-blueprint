//! A faixa da escrita ao vivo, acima dos projetos: quem escreve (com o cursor que pisca), quantas ações já têm plano e a barra de andamento do plano inteiro, uma peça por ação.
//! Com a escrita parada, âmbar, há quanto tempo e `Descartar`. Só desenha; o gesto sai pelo `Estado` que a folha passa.

use super::super::agentes::figuras;
use super::super::agentes_movimento as mov;
use super::super::agentes_pecas as pecas;
use super::super::caixinha::botao_mini;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{EstadoEscrita, Folha};
use super::super::orquestrador::figura_svg_alfa;
use super::super::Estado;
use super::{contagem, corpo_descartar, frase, minutos_parada, ALTURA};
use crate::i18n::tr_com;
use crate::planta::{self, botao};
use crate::tema::{AMBAR, AZUL, BAIXO, CAMPO, CARTAO, MEDIO, TINTA, VERDE};
use egui::{Color32, Rect, Stroke, StrokeKind, Ui};

const LARGURA_FIGURA: f32 = 56.0;
const X_TEXTO: f32 = 82.0;
/// O cursor pisca a cada meio segundo.
const PISCA_S: f64 = 1.0;
const MAX_PECAS: usize = 36;

fn cursor_aceso(ctx: &egui::Context) -> bool {
    if !mov::ligado() {
        return true;
    }
    mov::vivo(ctx);
    mov::agora(ctx) % PISCA_S < PISCA_S / 2.0
}

/// A figura do orquestrador: trabalhando enquanto escreve, dormindo quando a escrita parou.
fn figura(ui: &Ui, c: &Pincel, parada: bool, x: f32, y: f32) {
    let ms = if mov::ligado() { (mov::agora(ui.ctx()) * 1000.0) as u64 } else { 0 };
    let ((nome, gz), proximo) = figuras::do_orquestrador(!parada, ms);
    if let Some(espera) = proximo {
        ui.ctx().request_repaint_after(espera);
    }
    figura_svg_alfa(ui, c.caixa(x, y, LARGURA_FIGURA, LARGURA_FIGURA * 170.0 / 220.0), nome, gz, 1.0);
}

/// A barra do plano inteiro: uma peça por ação. As prontas ficam cheias, a que está sendo escrita respira e as que aguardam só têm o contorno.
fn pecas_do_plano(c: &Pincel, f: &Folha, cor: Color32, parada: bool, x: f32, y: f32, w: f32) {
    let acoes: Vec<EstadoEscrita> = f.projetos.iter().flat_map(|p| &p.acoes).map(|a| a.escrita).collect();
    let (prontas, total) = contagem(f);
    if acoes.is_empty() || acoes.len() > MAX_PECAS {
        let fracao = mov::suave(c.p.ctx(), "escrita-fracao", prontas as f32 / total.max(1) as f32);
        pecas::barra(c, x, y, w, 5.0, pecas::Modo::Fracao(fracao), cor);
        return;
    }
    let (vao, n) = (3.0, acoes.len() as f32);
    let largura = ((w - vao * (n - 1.0)) / n).max(3.0);
    let respira = 0.45 + 0.55 * mov::onda(mov::fase(c.p.ctx(), 1.3));
    for (i, estado) in acoes.into_iter().enumerate() {
        let r = Rect::from_min_size(c.pt(x + i as f32 * (largura + vao), y), egui::vec2(largura, 5.0));
        let chegou = mov::chegada(c.p.ctx(), ("escrita-peca", i, estado as u8), 0.45);
        match estado {
            EstadoEscrita::Pronta => {
                c.p.rect_filled(r, 0, cor.gamma_multiply(0.35 + 0.65 * chegou));
            }
            EstadoEscrita::Escrevendo if !parada => {
                c.p.rect_filled(r, 0, cor.gamma_multiply(respira));
                mov::varredura(c.p, r, mov::fase(c.p.ctx(), 1.1), TINTA.gamma_multiply(0.7));
            }
            _ => {
                c.p.rect_filled(r, 0, CAMPO);
            }
        }
        c.p.rect_stroke(r, 0, Stroke::new(1.0, BAIXO.gamma_multiply(0.55)), StrokeKind::Inside);
    }
}

/// A faixa em `(x, y)` com a largura `w`: quem escreve, quanto já tem plano e, com a escrita parada, há quanto tempo e `Descartar`. Devolve a altura ocupada, a mesma de `altura`.
#[allow(clippy::too_many_arguments)]
pub fn faixa(ui: &mut Ui, c: &Pincel, plano: &Estado, servidor: &str, f: &Folha, x: f32, y: f32, w: f32) -> f32 {
    let Some(e) = &f.escrita else {
        return 0.0;
    };
    let ctx = ui.ctx().clone();
    let parada = minutos_parada(e).is_some();
    let t = mov::chegada(&ctx, ("escrita-faixa", &f.blueprint.id), 0.45);
    let entra = mov::Entra::nova(c, t);
    let c = &entra.pincel();
    let (texto, tom) = frase(e);
    let cor = mov::suavizar(&ctx, "escrita-cor", if parada { AMBAR } else { AZUL });
    pecas::cartao(c, x, y, w, ALTURA, CARTAO.gamma_multiply(0.92), cor.gamma_multiply(0.85), Some(cor));
    planta::canto(c.p, c.caixa(x, y, w, ALTURA));
    figura(ui, c, parada, x + 14.0, y + 8.0);

    let mut fim = x + w - 18.0;
    if parada {
        let b = botao_mini(crate::i18n::tr("Descartar"));
        fim -= b.largura(c.p);
        let r = mov::com_opacidade(ui, t, |ui| botao(ui, c.pt(fim, y + 10.0), &b).on_hover_text(crate::i18n::tr("Apaga o que foi escrito nesta versão. A folha não muda.")));
        if r.clicked() {
            plano.gesto(&ctx, servidor, "descartar-escrita", &corpo_descartar(e));
        }
        fim -= 14.0;
    }
    let (prontas, total) = contagem(f);
    let pct = (mov::suave(&ctx, "escrita-pct", prontas as f32 / total.max(1) as f32) * 100.0).round() as u32;
    let porcento = Letra::nova(20.0, cor).forte().fim();
    let largura_pct = c.texto(&format!("{pct}%"), fim, y + 27.0, porcento);
    fim -= largura_pct + 16.0;

    let forte = Letra::nova(12.0, tom).forte();
    let (corte, _) = c.cortado(&texto, (fim - x - X_TEXTO - 14.0).max(40.0), &forte);
    let largura = c.texto(&corte, x + X_TEXTO, y + 25.0, forte);
    if !parada && cursor_aceso(&ctx) {
        c.ret(x + X_TEXTO + largura + 4.0, y + 14.0, 2.0, 14.0, Some(AZUL), None);
    }
    let leve = Letra::nova(10.5, MEDIO);
    let conta = tr_com("{prontas} de {total} ações com plano", &[("prontas", &prontas.to_string()), ("total", &total.to_string())]);
    c.texto(&c.cortado(&conta, (fim - x - X_TEXTO).max(40.0), &leve).0, x + X_TEXTO, y + 42.0, leve);
    pecas_do_plano(c, f, if parada { AMBAR } else { VERDE }, parada, x + X_TEXTO, y + ALTURA - 13.0, w - X_TEXTO - 18.0);
    ALTURA
}
