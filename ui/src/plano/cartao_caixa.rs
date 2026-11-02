//! A caixinha de cada plano (`design/planejar/acao-planos.mjs:15`) com cantos suaves e movimento: o desenho sai da situação que o servidor calculou,
//! o clique só marca para iniciar quem está livre (`caixinha::clique_de`), e o hover mostra o «play» que chama o agente.

use super::acao_estilo::{alfa, mix, raio};
use super::acao_movimento::{hover, pulso};
use super::caixinha::{clique_de, desenho_de, Clique, Desenho, Lapis};
use super::modelo::{Plano, Situacao};
use crate::i18n::tr;
use crate::tema::{AMBAR, AZUL, CAMPO, TINTA, VERDE, VERMELHO};
use egui::{pos2, vec2, Rect, Sense, Shape, Stroke, StrokeKind, Ui};

pub const LADO: f32 = 14.0;

/// Desenha a caixinha com o canto de cima à esquerda em `(x, y)`. `h` (0 a 1) é o quanto o mouse já está em cima.
pub fn desenhar(l: Lapis, x: f32, y: f32, s: Situacao, h: f32) {
    let r = Rect::from_min_size(l.pt(x, y), vec2(LADO, LADO));
    let c = r.center();
    let desenho = desenho_de(s);
    let traco = |cor| Stroke::new(1.0, cor);
    match desenho {
        Desenho::Tracejada => {
            l.p.rect_stroke(r, 0, Stroke::new(1.3, crate::tema::TINTA), StrokeKind::Middle);
            return;
        }
        Desenho::Check => {
            l.p.rect_filled(r, raio(), alfa(VERDE, 0.16));
            l.p.rect_stroke(r, raio(), traco(alfa(VERDE, 0.85)), StrokeKind::Inside);
            l.p.add(Shape::line(vec![pos2(c.x - 3.6, c.y + 0.2), pos2(c.x - 1.0, c.y + 2.9), pos2(c.x + 3.8, c.y - 3.0)], Stroke::new(1.5, VERDE)));
        }
        Desenho::Piscando => {
            l.p.rect_filled(r, raio(), alfa(AZUL, 0.12));
            l.p.rect_stroke(r, raio(), traco(alfa(AZUL, 0.9)), StrokeKind::Inside);
            let respira = pulso(l.p.ctx(), 1.6);
            l.p.rect_filled(Rect::from_center_size(c, vec2(6.0, 6.0)), 1.0, alfa(AZUL, 0.35 + 0.65 * respira));
        }
        Desenho::Vazia | Desenho::Ponto | Desenho::Xis => {
            let borda = mix(alfa(TINTA, 0.8), TINTA, h);
            l.p.rect_filled(r, raio(), mix(CAMPO, mix(CAMPO, AZUL, 0.3), h));
            l.p.rect_stroke(r, raio(), traco(if desenho == Desenho::Xis { alfa(VERMELHO, 0.9) } else { borda }), StrokeKind::Inside);
            match desenho {
                Desenho::Ponto => {
                    l.p.circle_filled(c, 2.2, AMBAR);
                }
                Desenho::Xis => {
                    l.p.add(Shape::line(vec![pos2(c.x - 3.0, c.y - 3.0), pos2(c.x + 3.0, c.y + 3.0)], Stroke::new(1.3, VERMELHO)));
                    l.p.add(Shape::line(vec![pos2(c.x + 3.0, c.y - 3.0), pos2(c.x - 3.0, c.y + 3.0)], Stroke::new(1.3, VERMELHO)));
                }
                // o «play» de quem ainda não começou: aparece com o mouse em cima
                _ if h > 0.02 => {
                    l.p.add(Shape::convex_polygon(vec![pos2(c.x - 2.0, c.y - 3.4), pos2(c.x + 3.4, c.y), pos2(c.x - 2.0, c.y + 3.4)], alfa(AZUL, h), Stroke::NONE));
                }
                _ => {}
            }
        }
    }
}

/// A caixinha com o clique: quem chama faz o que voltar (`marcar`, ou `Sub::Agentes`), porque a folha que ele percorre está emprestada do mesmo `Estado`.
pub fn caixa(l: Lapis, ui: &mut Ui, x: f32, y: f32, plano: &Plano) -> Option<Clique> {
    let pede = clique_de(plano);
    let r = Rect::from_min_size(l.pt(x, y), vec2(LADO, LADO)).expand(3.0);
    let resposta = ui.interact(r, ui.scope_id().with(("plano-caixinha", &plano.id)), if pede.is_some() { Sense::click() } else { Sense::hover() });
    let h = hover(ui.ctx(), ("caixinha", &plano.id), pede == Some(Clique::Marcar) && resposta.hovered());
    desenhar(l, x, y, plano.situacao, h);
    let pede = pede?;
    let resposta = resposta.on_hover_text(match pede {
        Clique::Marcar => tr("Chamar o agente neste plano"),
        Clique::Agentes => tr("Ver em Agentes"),
    });
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resposta.clicked().then_some(pede)
}
