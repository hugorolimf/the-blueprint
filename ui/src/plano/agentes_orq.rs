//! O cartão de destaque do orquestrador na tela de agentes: a figura sob um foco de luz que respira, o estado (Desligado, Ligado, Trabalhando) com o ponto que pulsa,
//! a conversa em balões e o campo de pedir com o botão de enviar dentro dele, o único botão cheio do cartão. Só desenha e devolve o pedido: quem manda o gesto é `agentes::tela`.

use super::super::agentes_movimento as mov;
use super::super::agentes_pecas as pecas;
use super::super::caixinha::botao_mini;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::Orquestrador;
use super::super::orquestrador::{corpo_ligado, corpo_mensagem, figura_svg_alfa, Pedido};
use super::{conversa, figuras};
use crate::fonte::{familia, Peso};
use crate::i18n::{tr, tr_com};
use crate::planta::{self, botao, Botao, Texto};
use crate::tema::{AZUL, BAIXO, CAMPO, CARTAO, MEDIO, TINTA, VERDE};
use egui::{vec2, Color32, FontId, Id, Rect, Sense, Stroke, StrokeKind, TextEdit, Ui};
use serde_json::Value;

pub const ALTURA: f32 = 218.0;
const ZONA_ESQUERDA: f32 = 346.0;
const FIGURA: (f32, f32) = (132.0, 102.0);
const ALTO_CONVERSA: f32 = 118.0;
const ALTO_CAMPO: f32 = 34.0;

/// Como o orquestrador está: o que a pílula escreve e a cor do ponto.
#[derive(Clone, Copy, PartialEq)]
enum Estado {
    Desligado,
    Ligado,
    Trabalhando,
}

fn estado_de(o: &Orquestrador) -> Estado {
    match (o.ligado, o.acordado) {
        (_, true) => Estado::Trabalhando,
        (true, false) => Estado::Ligado,
        (false, false) => Estado::Desligado,
    }
}

impl Estado {
    fn cor(self) -> Color32 {
        match self {
            Estado::Desligado => MEDIO,
            Estado::Ligado => VERDE,
            Estado::Trabalhando => AZUL,
        }
    }

    fn rotulo(self) -> &'static str {
        match self {
            Estado::Desligado => tr("Desligado"),
            Estado::Ligado => tr("Ligado"),
            Estado::Trabalhando => tr("Trabalhando"),
        }
    }

    /// O que o estado conta debaixo da pílula.
    fn explicacao(self, trabalhando: u32) -> (String, Color32) {
        match self {
            Estado::Trabalhando if trabalhando == 1 => (tr("Coordenando 1 agente").to_string(), AZUL),
            Estado::Trabalhando => (tr_com("Coordenando {n} agentes", &[("n", &trabalhando.to_string())]), AZUL),
            Estado::Ligado => (tr("Dormindo. Acorda quando um plano termina, pede ajuda ou quando você manda uma mensagem.").to_string(), MEDIO),
            Estado::Desligado => (tr("Ligue, ou mande uma mensagem: mandar também liga.").to_string(), MEDIO),
        }
    }
}

/// O foco de luz atrás da figura: círculos sobrepostos que clareiam o centro; respira enquanto ele trabalha.
fn foco(c: &Pincel, centro: (f32, f32), cor: Color32, est: Estado) {
    let respira = if est == Estado::Trabalhando { mov::onda(mov::fase(c.p.ctx(), 3.2)) } else { 0.0 };
    let base = match est {
        Estado::Trabalhando => 0.034 + 0.016 * respira,
        Estado::Ligado => 0.02,
        Estado::Desligado => 0.0,
    };
    for k in 0..7 {
        c.p.circle_filled(c.pt(centro.0, centro.1), 76.0 - k as f32 * 10.0, cor.gamma_multiply(base));
    }
}

fn figura(ui: &Ui, c: &Pincel, o: &Orquestrador, x: f32, y: f32) {
    let ms = if mov::ligado() { (mov::agora(ui.ctx()) * 1000.0) as u64 } else { 0 };
    let ((nome, gz), proximo) = figuras::do_orquestrador(o.acordado, ms);
    if let Some(espera) = proximo {
        ui.ctx().request_repaint_after(espera);
    }
    let alfa = mov::suave(ui.ctx(), "orq-figura", if o.ligado || o.acordado { 1.0 } else { 0.55 });
    figura_svg_alfa(ui, c.caixa(x, y, FIGURA.0, FIGURA.1), nome, gz, alfa);
}

/// A pílula do estado: o ponto, a palavra e, no mouse, a explicação inteira. Devolve a largura.
fn pilula(ui: &Ui, c: &Pincel, est: Estado, cor: Color32, dica: &str, x: f32, y: f32) -> f32 {
    let letra = Letra::nova(9.5, cor).forte().trilha(0.14);
    let w = (c.largura(&est.rotulo().to_uppercase(), &letra) + 36.0).round();
    c.ret(x, y, w, 20.0, Some(cor.gamma_multiply(0.12)), Some((cor.gamma_multiply(0.6), 1.0)));
    if est == Estado::Desligado {
        pecas::ponto_vazado(c, x + 11.0, y + 10.0, cor);
    } else {
        pecas::ponto(c, x + 11.0, y + 10.0, cor, est == Estado::Trabalhando);
    }
    c.rotulo(est.rotulo(), x + 23.0, y + 14.0, letra);
    let _ = ui.interact(c.caixa(x, y, w, 20.0), ui.scope_id().with("orq-pilula"), Sense::hover()).on_hover_text(dica);
    w
}

/// O campo de pedir ao orquestrador, com `Enviar` dentro. Devolve o corpo do gesto quando a mensagem saiu.
fn campo(ui: &mut Ui, c: &Pincel, o: &Orquestrador, x: f32, y: f32, w: f32, ws: &str, bp: &str) -> Option<Value> {
    let ctx = ui.ctx().clone();
    let id = Id::unique(("plano-orq-mensagem", ws));
    let mut texto = ctx.data(|d| d.get_temp::<String>(id)).unwrap_or_default();
    let pronto = corpo_mensagem(ws, bp, &texto);
    let enviar = Botao { cheio: true, ativo: pronto.is_some(), altura: ALTO_CAMPO - 8.0, ..botao_mini(tr("Enviar")) };
    let bw = enviar.largura(c.p);
    let foco = ctx.animate_bool_with_time(id.with("foco"), ctx.memory(|m| m.has_focus(id.with("campo"))), 0.18);
    let r = c.caixa(x, y, w, ALTO_CAMPO);
    c.p.rect_filled(r, 0, CAMPO);
    c.p.rect_stroke(r, 0, Stroke::new(1.0, BAIXO.lerp_to_gamma(TINTA, foco)), StrokeKind::Inside);
    c.ret(x, y, 2.0 + 2.0 * foco, ALTO_CAMPO, Some(AZUL.gamma_multiply(foco)), None);
    let dica = egui::RichText::new(tr("Pedir ao orquestrador…")).size(10.5).color(MEDIO);
    let editor = TextEdit::singleline(&mut texto).id(id.with("campo")).frame(egui::Frame::NONE).margin(egui::Margin::ZERO).font(FontId::new(10.5, familia(Peso::Normal))).text_color(TINTA).hint_text(dica).desired_width(w - bw - 40.0);
    let resposta = ui.put(Rect::from_min_size(c.pt(x + 14.0, y + 9.0), vec2(w - bw - 40.0, 16.0)), editor);
    let enter = resposta.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let clicou = botao(ui, c.pt(x + w - bw - 4.0, y + 4.0), &enviar).clicked();
    let saiu = pronto.filter(|_| enter || clicou);
    if saiu.is_some() {
        conversa::registrar(&ctx, ws, &texto, &o.agora);
        texto.clear();
        resposta.request_focus();
    }
    ctx.data_mut(|d| d.insert_temp(id, texto));
    saiu
}

/// O cartão em `(x0, y)` até `x1`. Devolve o gesto pedido: ligar, desligar ou a mensagem.
#[allow(clippy::too_many_arguments)]
pub fn faixa(ui: &mut Ui, c: &Pincel, o: &Orquestrador, workspace: &str, trabalhando: u32, x0: f32, y: f32, x1: f32, ws: &str, bp: &str) -> Option<Pedido> {
    let ctx = ui.ctx().clone();
    let t = mov::entrada(&ctx, "agentes", 0);
    let entra = mov::Entra::nova(c, t);
    let c = &entra.pincel();
    let (est, w) = (estado_de(o), x1 - x0);
    let cor = mov::suavizar(&ctx, "orq-cor", est.cor());
    let borda = mov::suavizar(&ctx, "orq-borda", if est == Estado::Trabalhando { AZUL.gamma_multiply(0.75) } else { TINTA.gamma_multiply(0.8) });
    pecas::cartao(c, x0, y, w, ALTURA, CARTAO, borda, Some(cor.gamma_multiply(0.85)));
    planta::canto(c.p, c.caixa(x0, y, w, ALTURA));
    foco(c, (x0 + 16.0 + FIGURA.0 / 2.0, y + ALTURA / 2.0 + 8.0), cor, est);
    figura(ui, c, o, x0 + 16.0, y + (ALTURA - FIGURA.1) / 2.0 + 8.0);

    let xt = x0 + 168.0;
    c.texto(tr("Orquestrador"), xt, y + 44.0, Letra::nova(16.0, TINTA).forte());
    let sub = Letra::nova(10.5, MEDIO);
    c.texto(&c.cortado(&tr_com("Mestre do workspace {nome}", &[("nome", workspace)]), x0 + ZONA_ESQUERDA - xt - 12.0, &sub).0, xt, y + 62.0, sub);
    let (frase, cor_frase) = est.explicacao(trabalhando);
    pilula(ui, c, est, cor, &frase, xt, y + 76.0);
    Texto::novo(frase, 10.0).cor(cor_frase.gamma_multiply(0.9)).caixa(x0 + ZONA_ESQUERDA - xt - 14.0).linhas(4).lh(14.5).em(c.p, c.pt(xt, y + 106.0));
    let chave = botao_mini(if o.ligado { tr("Desligar") } else { tr("Ligar") });
    let liga = mov::com_opacidade(ui, t, |ui| botao(ui, c.pt(xt, y + ALTURA - 38.0), &chave).clicked());
    c.fio(x0 + ZONA_ESQUERDA, y + 18.0, x0 + ZONA_ESQUERDA, y + ALTURA - 18.0, BAIXO.gamma_multiply(0.4), 1.0);

    let (xr, xf) = (x0 + ZONA_ESQUERDA + 24.0, x1 - 18.0);
    c.rotulo(tr("Agora"), xr, y + 28.0, Letra::nova(10.0, MEDIO).trilha(0.14));
    c.fio(xr, y + 36.0, xf, y + 36.0, BAIXO.gamma_multiply(0.45), 1.0);
    conversa::desenhar(ui, c, o, &conversa::enviadas(&ctx, ws), xr, y + 46.0, xf - xr, ALTO_CONVERSA);
    let mensagem = mov::com_opacidade(ui, t, |ui| campo(ui, c, o, xr, y + 46.0 + ALTO_CONVERSA + 12.0, xf - xr, ws, bp));
    mensagem.map(|corpo| ("orquestrador", corpo)).or_else(|| liga.then(|| ("orquestrador", corpo_ligado(ws, !o.ligado))))
}
