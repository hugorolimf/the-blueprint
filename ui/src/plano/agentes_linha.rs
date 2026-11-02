//! Um plano da tela de agentes como cartão: a figura do agente na pose da situação, o plano com a barra de andamento, a situação com o ponto que pulsa,
//! de quem depende, a área, o ramo, há quanto tempo está assim e os botões. O servidor decide quais botões existem; aqui eles só ficam claros: `Integrar` é o primário.

use super::super::agentes_movimento as mov;
use super::super::agentes_pecas::{self as pecas, Modo};
use super::super::agentes_tempo as tempo;
use super::super::caixinha::botao_mini;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{PlanoAgente, Situacao};
use super::super::orquestrador::{figura_svg_alfa, Pedido};
use super::{botoes_da_linha, corpo_botao, dependencia, dica_botao, dica_situacao, figuras, pose, ramo_visivel, rotulo_botao, rotulo_situacao, BotaoLinha};
use crate::i18n::tr;
use crate::planta::{botao, Botao};
use crate::tema::{AMBAR, AZUL, BAIXO, CARTAO, MEDIO, QUADRA, TINTA, VERDE, VERMELHO};
use egui::{Id, Sense, Ui};

pub const ALTO: f32 = 64.0;
pub const VAO: f32 = 8.0;
const FIGURA: f32 = 54.0;
const LARGURA_BOTOES: f32 = 232.0;
const LARGURA_TEMPO: f32 = 76.0;
const LARGURA_SITUACAO: f32 = 168.0;

/// Onde começa cada coluna; os títulos da tabela usam os mesmos x.
pub struct Colunas {
    pub plano: f32,
    pub situacao: f32,
    pub depende: f32,
    pub area: f32,
    pub ramo: f32,
    pub tempo: f32,
    pub botoes: f32,
}

pub fn colunas(x0: f32, x1: f32) -> Colunas {
    let w = x1 - x0;
    let botoes = x1 - 14.0 - LARGURA_BOTOES;
    let tempo = botoes - LARGURA_TEMPO - 6.0;
    let ramo = tempo - (w * 0.13).clamp(110.0, 200.0);
    let area = ramo - (w * 0.09).clamp(84.0, 150.0);
    let depende = area - (w * 0.07).clamp(70.0, 120.0);
    Colunas { plano: x0 + FIGURA + 28.0, situacao: depende - LARGURA_SITUACAO, depende, area, ramo, tempo, botoes }
}

/// Escreve cortando em `max`, com o texto inteiro na dica se cortou.
#[allow(clippy::too_many_arguments)]
fn escrito(ui: &Ui, c: &Pincel, s: &str, x: f32, y: f32, max: f32, letra: Letra, chave: (&str, &str)) -> f32 {
    let (t, cortou) = c.cortado(s, max.max(8.0), &letra);
    let largura = c.texto(&t, x, y, letra);
    if cortou {
        let _ = ui.interact(c.caixa(x, y - 11.0, largura, 15.0), Id::unique(("plano-agentes-dica", chave)), Sense::hover()).on_hover_text(s);
    }
    largura
}

/// Como a barra de andamento do plano se enche, e em que cor: feito, em curso, ou ainda esperando.
fn andamento(s: Situacao) -> (Modo, egui::Color32) {
    match s {
        Situacao::Pronto => (Modo::Fracao(1.0), VERDE),
        Situacao::Trabalhando => (Modo::Corre, AZUL),
        Situacao::Revisao | Situacao::Dependencia | Situacao::Area => (Modo::Espera, AMBAR),
        Situacao::Parado | Situacao::Bloqueado => (Modo::Espera, VERMELHO),
        Situacao::Fila | Situacao::Nao | Situacao::Outra => (Modo::Espera, MEDIO),
    }
}

/// Os botões da linha, da direita para a esquerda: `Integrar` cheio, `Retomar` com contorno, `Cancelar` discreto e vermelho.
fn botoes(ui: &mut Ui, c: &Pincel, q: &PlanoAgente, y: f32, x1: f32) -> Option<Pedido> {
    let (mut x, mut pedido, topo) = (x1 - 14.0, None, y + (ALTO - pecas::ALTO_BOTAO) / 2.0);
    for b in botoes_da_linha(q).into_iter().rev() {
        let rotulo = tr(rotulo_botao(b));
        let ativo = b != BotaoLinha::Integrar || !q.destino.is_empty();
        let (largura, resposta) = match b {
            BotaoLinha::Integrar => {
                let desenho = Botao { cheio: true, ativo, ..botao_mini(rotulo) };
                (desenho.largura(c.p), botao(ui, c.pt(x - desenho.largura(c.p), topo), &desenho))
            }
            BotaoLinha::Retomar => {
                let desenho = botao_mini(rotulo);
                (desenho.largura(c.p), botao(ui, c.pt(x - desenho.largura(c.p), topo), &desenho))
            }
            BotaoLinha::Cancelar => {
                let l = pecas::largura_botao_texto(c, rotulo);
                (l, pecas::botao_texto(ui, c, x - l, topo, rotulo, VERMELHO))
            }
        };
        x -= largura;
        let clicou = resposta.clicked();
        if let Some(dica) = dica_botao(b, q) {
            let _ = resposta.on_hover_text(dica);
        }
        if clicou {
            pedido = Some(corpo_botao(b, q));
        }
        x -= 6.0;
    }
    pedido
}

/// Uma linha-cartão de plano com o topo em `y`; `indice` é a vez dela na cascata de entrada.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, q: &PlanoAgente, col: &Colunas, indice: usize, y: f32, x0: f32, x1: f32) -> Option<Pedido> {
    let ctx = ui.ctx().clone();
    let t = mov::entrada(&ctx, "agentes", 8 + indice);
    let sobre = ui.rect_contains_pointer(c.caixa(x0, y, x1 - x0, ALTO));
    let sobre = ctx.animate_bool_with_time(Id::unique(("agentes-linha-sobre", &q.id)), sobre, 0.15);
    let entra = mov::Entra::nova(c, t);
    let c = &entra.pincel();
    let (rotulo, alvo) = rotulo_situacao(q);
    let cor = mov::suavizar(&ctx, ("linha-cor", &q.id), alvo);
    let trabalha = q.situacao == Situacao::Trabalhando;
    let pulso = if trabalha { mov::onda(mov::fase(&ctx, 2.4)) } else { 0.0 };
    let fundo = mov::suavizar(&ctx, ("linha-fundo", &q.id), if trabalha { QUADRA.gamma_multiply(0.5) } else { CARTAO.gamma_multiply(0.7) });
    let borda = BAIXO.gamma_multiply(0.4 + 0.2 * sobre).lerp_to_gamma(cor.gamma_multiply(0.9), pulso * 0.7);
    pecas::cartao(c, x0, y, x1 - x0, ALTO, fundo.lerp_to_gamma(QUADRA.gamma_multiply(0.45), sobre * 0.5), borda, None);
    c.ret(x0, y, 3.0, ALTO, Some(cor), None);

    let ms = if mov::ligado() { (mov::agora(&ctx) * 1000.0) as u64 } else { 0 };
    let ((nome, gz), proximo) = figuras::do_agente(pose(q.situacao), ms);
    if let Some(espera) = proximo {
        ctx.request_repaint_after(espera);
    }
    figura_svg_alfa(ui, c.caixa(x0 + 14.0, y + (ALTO - FIGURA * 170.0 / 220.0) / 2.0, FIGURA, FIGURA * 170.0 / 220.0), nome, gz, t);

    let ancho = col.situacao - 16.0 - col.plano;
    let ref_w = c.texto(&q.referencia, col.plano, y + 26.0, Letra::nova(10.0, MEDIO));
    escrito(ui, c, &q.texto, col.plano + ref_w + 8.0, y + 26.0, ancho - ref_w - 8.0, Letra::nova(12.0, TINTA), ("texto", &q.id));
    let (modo, cor_barra) = andamento(q.situacao);
    pecas::barra(c, col.plano, y + 35.0, ancho.min(440.0), 4.0, modo, cor_barra);
    let dica = dica_situacao(q);
    if let Some(primeira) = dica.as_deref().and_then(|d| d.lines().next()) {
        escrito(ui, c, primeira, col.plano, y + 54.0, ancho, Letra::nova(9.5, MEDIO), ("dica", &q.id));
    }

    pecas::ponto(c, col.situacao + 5.0, y + 22.0, cor, trabalha);
    let largura = escrito(ui, c, &rotulo, col.situacao + 18.0, y + 26.0, LARGURA_SITUACAO - 24.0, Letra::nova(11.0, cor).forte(), ("situacao", &q.id));
    if let Some(dica) = dica {
        let _ = ui.interact(c.caixa(col.situacao, y + 8.0, largura + 24.0, 28.0), Id::unique(("plano-agentes-situacao", &q.id)), Sense::hover()).on_hover_text(dica);
    }

    let vazio = |s: &str| if s.is_empty() { "—".to_string() } else { s.to_string() };
    let (dep, destaque) = dependencia(q);
    escrito(ui, c, &vazio(&dep), col.depende, y + 26.0, col.area - col.depende - 8.0, Letra::nova(10.5, if destaque { cor } else { MEDIO }), ("depende", &q.id));
    escrito(ui, c, &vazio(&q.area), col.area, y + 26.0, col.ramo - col.area - 8.0, Letra::nova(10.5, if q.situacao == Situacao::Area { cor } else { MEDIO }), ("area", &q.id));
    match ramo_visivel(q) {
        Some(ramo) => {
            let (corte, _) = c.cortado(ramo, col.tempo - col.ramo - 28.0, &Letra::nova(9.5, TINTA));
            pecas::ficha(c, col.ramo, y + 14.0, &corte, if trabalha { AZUL } else { MEDIO });
        }
        None => {
            c.texto("—", col.ramo, y + 26.0, Letra::nova(10.5, MEDIO.gamma_multiply(0.7)));
        }
    }
    if !matches!(q.situacao, Situacao::Nao | Situacao::Outra) {
        let ha = tempo::escrito(tempo::visto_ha(&ctx, (&q.id, format!("{:?}", q.situacao))));
        c.texto(&ha, col.tempo, y + 26.0, Letra::nova(10.5, TINTA));
        c.texto(tr("nesta janela"), col.tempo, y + 42.0, Letra::nova(9.0, MEDIO.gamma_multiply(0.8)));
        let _ = ui.interact(c.caixa(col.tempo, y + 10.0, LARGURA_TEMPO, 40.0), Id::unique(("plano-agentes-tempo", &q.id)), Sense::hover()).on_hover_text(tr("Contado desde que esta janela viu o plano nesta situação."));
    }
    mov::com_opacidade(ui, t, |ui| botoes(ui, c, q, y, x1)).filter(|_| t > 0.6)
}
