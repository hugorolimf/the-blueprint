//! O desenho do menu, com as medidas de `web/css/menu-contextual.css`: o cabeçalho com o alvo, os itens em grupos separados por linha, o ícone à
//! esquerda e o «excluir» em tom de alerta. Os ícones são os traços de 16 px de `contexto-icones.js`.

use super::{Comando, Icone, Menu};
use crate::planta::Texto;
use crate::tema::{rgba, ERRO, MEDIO, PAPEL, TENUE, TINTA};
use egui::epaint::Shadow;
use egui::{pos2, vec2, Area, Color32, Context, CursorIcon, Id, Order, Painter, Pos2, Rect, ScrollArea, Sense, Stroke, Ui, UiBuilder};

const LARGURA_MIN: f32 = 240.0;
const LARGURA_MAX: f32 = 360.0;
const BORDA: f32 = 1.0;
const FOLGA: f32 = 5.0;
const MARGEM: f32 = 8.0;
const ITEM: f32 = 34.0;
const VAO: f32 = 1.0;
const SEPARADOR: f32 = 9.0;
const TOPO: f32 = 31.0;
const ANTES_DO_TEXTO: f32 = BORDA + 12.0 + 16.0 + 10.0;
/// O texto escuro sobre o fundo de alerta (`.perigo:hover`).
const SOBRE_ERRO: Color32 = Color32::from_rgb(0x3b, 0x0d, 0x06);

fn rotulo<'a>(s: &'a str, cor: Color32) -> Texto<'a> {
    Texto::novo(s, 11.0).trilha(0.04).cor(cor).linhas(1)
}

fn cabecalho<'a>(s: &'a str) -> Texto<'a> {
    Texto::novo(s, 10.0).trilha(0.08).cor(MEDIO).linhas(1)
}

/// Desenha o menu na camada de cima e devolve o comando do item clicado.
pub fn mostrar(ctx: &Context, m: &mut Menu, ocupado: bool) -> Option<Comando> {
    let id = Id::unique("menu-contexto");
    let medida = ctx.layer_painter(egui::LayerId::new(Order::Foreground, id));
    let titulo = m.titulo.to_uppercase();
    let conteudo = m.itens.iter().map(|i| ANTES_DO_TEXTO + rotulo(&i.rotulo, TINTA).largura(&medida) + 13.0).fold(cabecalho(&titulo).largura(&medida) + 24.0, f32::max);
    let tela = ctx.content_rect();
    let largura = (conteudo + 2.0 * (FOLGA + BORDA)).clamp(LARGURA_MIN, LARGURA_MAX.min(tela.width() - 16.0).max(LARGURA_MIN)).ceil();
    let separadores = m.itens.windows(2).filter(|p| p[0].grupo != p[1].grupo).count() as f32;
    let corpo = TOPO + VAO + m.itens.len() as f32 * (ITEM + VAO) + separadores * (SEPARADOR + VAO);
    let altura = (corpo + 2.0 * (FOLGA + BORDA)).min(tela.height() - 16.0).floor();
    let x = m.pos.x.min(tela.max.x - largura - MARGEM).max(tela.min.x + MARGEM).round();
    let y = m.pos.y.min(tela.max.y - altura - MARGEM).max(tela.min.y + MARGEM).round();
    let caixa = Rect::from_min_size(pos2(x, y), vec2(largura, altura));
    m.caixa = caixa;
    let moveu = ctx.input(|i| i.pointer.delta() != egui::Vec2::ZERO);
    let mut escolhido = None;
    Area::new(id).order(Order::Foreground).fixed_pos(caixa.min).constrain(false).show(ctx, |ui| {
        ui.expand_to_include_rect(caixa);
        let p = ui.painter();
        p.add(Shadow { offset: [0, 12], blur: 36, spread: 0, color: rgba(0x02132e, 0xcc) }.as_shape(caixa, 0));
        p.rect_filled(caixa, 0, rgba(0x082b5e, 0xf5));
        p.rect_stroke(caixa, 0, Stroke::new(BORDA, TINTA), egui::StrokeKind::Inside);
        let dentro = caixa.shrink(FOLGA + BORDA);
        ui.scope_builder(UiBuilder::new().max_rect(dentro), |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            ScrollArea::vertical().max_height(dentro.height()).auto_shrink([false, false]).show(ui, |ui| {
                topo(ui, &titulo, dentro.width());
                for i in 0..m.itens.len() {
                    if i > 0 && m.itens[i - 1].grupo != m.itens[i].grupo {
                        separador(ui, dentro.width());
                    }
                    if linha(ui, m, i, dentro.width(), moveu, ocupado) {
                        escolhido = m.itens[i].comando.take();
                    }
                    ui.add_space(VAO);
                }
            });
        });
    });
    escolhido
}

fn topo(ui: &mut Ui, titulo: &str, largura: f32) {
    let (r, _) = ui.allocate_exact_size(vec2(largura, TOPO), Sense::hover());
    let p = ui.painter();
    cabecalho(titulo).caixa(largura - 24.0).na_linha(p, r.min.x + 12.0, r.min.y + 6.0, 13.0);
    p.hline(r.x_range(), r.max.y - 4.0, Stroke::new(1.0, TENUE));
    ui.add_space(VAO);
}

fn separador(ui: &mut Ui, largura: f32) {
    let (r, _) = ui.allocate_exact_size(vec2(largura, SEPARADOR), Sense::hover());
    ui.painter().hline((r.min.x + 6.0)..=(r.max.x - 6.0), r.center().y, Stroke::new(1.0, TENUE));
    ui.add_space(VAO);
}

/// Um item. Passar o mouse leva o foco até ele, para o teclado continuar daí; a posição parada do cursor não rouba o foco das setas.
fn linha(ui: &mut Ui, m: &mut Menu, i: usize, largura: f32, moveu: bool, ocupado: bool) -> bool {
    let item = &m.itens[i];
    let bloqueado = ocupado && matches!(item.comando, Some(Comando::Dados(_)));
    let ativo = item.comando.is_some() && !bloqueado;
    let (r, mut resposta) = ui.allocate_exact_size(vec2(largura, ITEM), if ativo { Sense::click() } else { Sense::hover() });
    if resposta.hovered() {
        if moveu {
            m.foco = Some(i);
        }
        ui.ctx().set_cursor_icon(if ativo { CursorIcon::PointingHand } else { CursorIcon::NotAllowed });
    }
    let foco = m.foco == Some(i);
    let item = &m.itens[i];
    let p = ui.painter();
    let (fundo, cor, cor_icone) = match (foco && ativo, item.perigo) {
        (true, true) => (Some(ERRO), SOBRE_ERRO, SOBRE_ERRO),
        (true, false) => (Some(TINTA), PAPEL, PAPEL),
        (false, true) => (None, ERRO, ERRO),
        (false, false) => (None, TINTA, MEDIO),
    };
    if let Some(f) = fundo {
        p.rect_filled(r, 0, f);
    } else if foco {
        p.rect_stroke(r, 0, Stroke::new(1.0, TENUE), egui::StrokeKind::Inside);
    }
    let apagar = |c: Color32| if ativo { c } else { c.gamma_multiply(0.45) };
    icone(p, pos2(r.min.x + BORDA + 12.0, r.center().y - 8.0), item.icone, apagar(cor_icone));
    rotulo(&item.rotulo, apagar(cor)).caixa(largura - ANTES_DO_TEXTO - 13.0).na_linha(p, r.min.x + ANTES_DO_TEXTO, r.min.y, ITEM);
    if let Some(motivo) = item.motivo {
        resposta = resposta.on_hover_text(motivo);
    }
    resposta.clicked()
}

type Traco = &'static [(f32, f32)];

/// Os traços e os círculos `(x, y, raio)` de cada ícone, na grade de 16 px.
fn formas(nome: Icone) -> (&'static [Traco], &'static [(f32, f32, f32)]) {
    match nome {
        Icone::Abrir => (&[&[(6.0, 3.0), (13.0, 3.0), (13.0, 10.0)], &[(13.0, 3.0), (6.0, 10.0)], &[(12.0, 9.0), (12.0, 13.0), (3.0, 13.0), (3.0, 4.0), (7.0, 4.0)]], &[]),
        Icone::Alvo => (&[&[(8.0, 1.0), (8.0, 3.0)], &[(8.0, 13.0), (8.0, 15.0)], &[(1.0, 8.0), (3.0, 8.0)], &[(13.0, 8.0), (15.0, 8.0)]], &[(8.0, 8.0, 5.0), (8.0, 8.0, 1.5)]),
        Icone::Doc => (&[&[(4.0, 2.0), (10.0, 2.0), (13.0, 5.0), (13.0, 14.0), (4.0, 14.0), (4.0, 2.0)], &[(10.0, 2.0), (10.0, 5.0), (13.0, 5.0)], &[(6.0, 8.0), (11.0, 8.0)], &[(6.0, 11.0), (11.0, 11.0)]], &[]),
        Icone::Externo => (&[&[(9.0, 2.0), (14.0, 2.0), (14.0, 7.0)], &[(14.0, 2.0), (7.0, 9.0)], &[(12.0, 10.0), (12.0, 14.0), (2.0, 14.0), (2.0, 4.0), (6.0, 4.0)]], &[]),
        Icone::Check => (&[&[(3.0, 8.5), (6.5, 12.0), (13.0, 4.5)]], &[]),
        Icone::Circulo => (&[], &[(8.0, 8.0, 5.5)]),
        Icone::Lixo => (&[&[(3.0, 4.5), (13.0, 4.5)], &[(6.5, 4.5), (6.5, 3.0), (9.5, 3.0), (9.5, 4.5)], &[(4.5, 4.5), (5.2, 13.5), (10.8, 13.5), (11.5, 4.5)], &[(7.0, 7.0), (7.0, 11.0)], &[(9.0, 7.0), (9.0, 11.0)]], &[]),
        Icone::Voltar => (&[&[(13.0, 8.0), (3.0, 8.0)], &[(7.0, 4.0), (3.0, 8.0), (7.0, 12.0)]], &[]),
        Icone::TelaCheia => (&[&[(2.0, 6.0), (2.0, 2.0), (6.0, 2.0)], &[(10.0, 2.0), (14.0, 2.0), (14.0, 6.0)], &[(14.0, 10.0), (14.0, 14.0), (10.0, 14.0)], &[(6.0, 14.0), (2.0, 14.0), (2.0, 10.0)]], &[]),
        Icone::Relogio => (&[&[(8.0, 5.0), (8.0, 8.0), (10.0, 9.5)]], &[(8.0, 8.0, 5.5)]),
        Icone::Calendario => (&[&[(2.5, 3.5), (13.5, 3.5), (13.5, 13.5), (2.5, 13.5), (2.5, 3.5)], &[(2.5, 7.0), (13.5, 7.0)], &[(5.5, 2.0), (5.5, 5.0)], &[(10.5, 2.0), (10.5, 5.0)]], &[]),
        Icone::Mais => (&[&[(8.0, 3.0), (8.0, 13.0)], &[(3.0, 8.0), (13.0, 8.0)]], &[]),
        Icone::Recolher => (&[&[(4.0, 10.0), (8.0, 6.0), (12.0, 10.0)]], &[]),
        Icone::Expandir => (&[&[(4.0, 6.0), (8.0, 10.0), (12.0, 6.0)]], &[]),
    }
}

fn icone(p: &Painter, o: Pos2, nome: Icone, cor: Color32) {
    let traco = Stroke::new(1.4, cor);
    let (tracos, circulos) = formas(nome);
    for t in tracos {
        p.add(egui::Shape::line(t.iter().map(|(x, y)| o + vec2(*x, *y)).collect(), traco));
    }
    for (x, y, raio) in circulos {
        p.circle_stroke(o + vec2(*x, *y), *raio, traco);
    }
}
