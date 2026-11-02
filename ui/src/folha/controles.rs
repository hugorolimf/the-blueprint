//! Os botões e a busca do canto de cima da folha, que na página são HTML por cima do SVG (`.barra` em web/css/barra.css): com espaço, botões
//! com texto e a busca na segunda fileira; sem espaço (`barra-topo.js`), uma fileira só de ícones de 34 px.

use super::cena::Cena;
use crate::fonte::{familia, Peso};
use crate::i18n::tr;
use crate::planta::{self, botao, Botao};
use crate::telas::{Tela, Vista};
use crate::tema::{rgba, BAIXO, MEDIO, PAPEL, TINTA};
use egui::{pos2, vec2, Pos2, Rect, Response, Sense, Shape, Stroke, Ui};

/// A largura que os botões com texto pedem (`BARRA_CHEIA` em barra-topo.js).
const BARRA_CHEIA: f32 = 760.0;

#[derive(Clone, Copy, PartialEq)]
enum Item {
    Icone(&'static str),
    Texto(&'static str),
}

fn texto_do_botao(nome: &str) -> &'static str {
    match nome {
        "voltar" => "← Workspaces",
        "voltar-bp" => "← Blueprints",
        "planejar" => "  Planejar",
        "compartilhar" => tr("Compartilhar"),
        _ => tr("Apresentar"),
    }
}

/// Desenha uma fileira de itens alinhada à direita, de `direita` para a esquerda, a 8 px um do outro. Devolve o clique, se houve, e o x do primeiro.
fn fileira(ui: &mut Ui, itens: &[Item], direita: f32, y: f32) -> (Option<&'static str>, f32) {
    let p = ui.painter().clone();
    let largura = |i: &Item| match i {
        Item::Icone(_) => 34.0,
        Item::Texto(n) => Botao::barra(texto_do_botao(n)).largura(&p),
    };
    let mut x = direita;
    let mut clique = None;
    for item in itens.iter().rev() {
        x -= largura(item);
        let resposta = match item {
            Item::Icone(nome) => icone(ui, pos2(x, y), nome),
            Item::Texto(nome) => {
                let r = botao(ui, pos2(x, y), &Botao::barra(texto_do_botao(nome)));
                if *nome == "planejar" {
                    estrela(&p, pos2(x + 1.0 + 15.0 + 3.85, y + 17.0), r.hovered() || r.has_focus());
                }
                r
            }
        };
        if resposta.clicked() {
            clique = Some(match item {
                Item::Icone(n) | Item::Texto(n) => *n,
            });
        }
        x -= 8.0;
    }
    (clique, x + 8.0)
}

/// A busca: caixa de 380 x 35 com o texto de exemplo e o contador de resultados à direita; edita `vista.consulta`.
fn busca(ui: &mut Ui, vista: &mut Vista, x: f32, y: f32, dica: &str, resultado: Option<String>) -> Response {
    let p = ui.painter().clone();
    let campo = Rect::from_min_size(pos2(x, y), vec2(380.0, 35.0));
    p.rect_filled(campo, 0, rgba(0x0c3a78, 0xcc));
    planta::borda(&p, campo, MEDIO);
    let editor = egui::TextEdit::singleline(&mut vista.consulta)
        .frame(egui::Frame::NONE)
        .font(egui::FontId::new(11.0, familia(Peso::Normal)))
        .text_color(TINTA)
        .hint_text(egui::RichText::new(dica).size(11.0).color(BAIXO))
        .desired_width(campo.width() - 24.0)
        .align(egui::Align2::LEFT_CENTER);
    let resposta = ui.put(campo.shrink2(vec2(12.0, 1.0)), editor);
    if std::mem::take(&mut vista.focar_busca) {
        resposta.request_focus();
    }
    // o contador de resultados (`busca-n`) à direita do campo, em caixa alta e espaçado
    if let Some(n) = resultado {
        crate::planta::Texto::novo(n.to_uppercase(), 10.0).cor(MEDIO).trilha(0.12).na_linha_dir(&p, campo.max.x - 10.0, campo.min.y, campo.height());
    }
    resposta
}

/// Na barra compacta a busca é uma lupa que abre o campo por cima dos ícones; devolve `true` se o campo está aberto (e então os ícones não aparecem).
fn busca_compacta(ui: &mut Ui, vista: &mut Vista, direita: f32, y: f32, dica: &str, contador: Option<String>) -> bool {
    if !(vista.busca_aberta || !vista.consulta.is_empty()) {
        return false;
    }
    let r = busca(ui, vista, direita - 380.0, y, dica, contador);
    // vazia e sem foco: volta a ser a lupa
    if r.lost_focus() && vista.consulta.is_empty() {
        vista.busca_aberta = false;
    }
    true
}

/// A lista de blueprints do workspace.
pub fn lista(ui: &mut Ui, vista: &mut Vista, c: &Cena, w: f32, id: &str, cabecalho_fim: f32, n_resultados: usize, pesquisando: bool, abrir_web: &mut Option<String>) {
    let (direita, y) = (c.o.x + w - 64.0, c.o.y + 44.0);
    let contador = pesquisando.then(|| if n_resultados == 0 { tr("nenhum resultado").to_string() } else { crate::i18n::trn(n_resultados, "{n} resultado", "{n} resultados") });
    let compacta = (w - 64.0) - cabecalho_fim < BARRA_CHEIA;
    if compacta && busca_compacta(ui, vista, direita, y, tr("Pesquisar blueprints, ações, planos"), contador.clone()) {
        return;
    }
    let itens_cheia = [Item::Texto("voltar"), Item::Icone("tela"), Item::Icone("foco"), Item::Texto("planejar"), Item::Texto("compartilhar")];
    let itens_compacta = [Item::Icone("buscar"), Item::Icone("voltar"), Item::Icone("tela"), Item::Icone("foco"), Item::Icone("planejar"), Item::Icone("compartilhar")];
    let itens: &[Item] = if compacta { &itens_compacta } else { &itens_cheia };
    match fileira(ui, itens, direita, y).0 {
        Some("buscar") => {
            vista.busca_aberta = true;
            vista.focar_busca = true;
        }
        Some("voltar") => {
            vista.tela = Tela::Entrada;
            vista.consulta.clear();
        }
        Some("tela") => tela_cheia(ui),
        Some("planejar" | "compartilhar") => *abrir_web = Some(format!("#/c/{id}")),
        _ => {}
    }
    if !compacta {
        busca(ui, vista, direita - 380.0, y + 42.0, tr("Pesquisar blueprints, ações, planos"), contador);
    }
}

/// O blueprint aberto. `cabecalho_fim`: o x onde termina o título/subtítulo (já com os 24 px de respiro) para decidir entre a barra cheia e a de ícones.
pub fn blueprint(ui: &mut Ui, vista: &mut Vista, c: &Cena, w: f32, id: &str, cabecalho_fim: f32, pendentes: bool, contador: Option<String>, abrir_web: &mut Option<String>) -> (Option<&'static str>, Option<Periodo>) {
    let (direita, y) = (c.o.x + w - 64.0, c.o.y + 44.0);
    let compacta = (w - 64.0) - cabecalho_fim < BARRA_CHEIA;
    let comum = [Item::Icone("expandir"), Item::Icone("recolher"), Item::Icone("tela"), Item::Icone("foco")];
    if compacta && busca_compacta(ui, vista, direita, y, tr("Pesquisar ações, planos e checkpoints"), contador.clone()) {
        return (None, periodo(ui, c, pendentes));
    }
    let clique = if compacta {
        let itens = [Item::Icone("buscar"), Item::Icone("voltar-bp"), comum[0], comum[1], comum[2], comum[3], Item::Icone("planejar"), Item::Icone("compartilhar"), Item::Icone("apresentar")];
        fileira(ui, &itens, direita, y).0
    } else {
        let itens = [Item::Texto("voltar-bp"), comum[0], comum[1], comum[2], comum[3], Item::Texto("planejar"), Item::Texto("compartilhar"), Item::Texto("apresentar")];
        let (clique, _) = fileira(ui, &itens, direita, y);
        busca(ui, vista, direita - 380.0, y + 42.0, tr("Pesquisar ações, planos e checkpoints"), contador);
        clique
    };
    match clique {
        Some("buscar") => {
            vista.busca_aberta = true;
            vista.focar_busca = true;
        }
        Some("voltar-bp") => {
            vista.tela = Tela::Workspace(id.to_string());
            vista.consulta.clear();
        }
        Some("tela") => tela_cheia(ui),
        Some("planejar" | "compartilhar" | "apresentar") => *abrir_web = Some(format!("#/c/{id}")),
        _ => {}
    }
    (clique.filter(|n| matches!(*n, "expandir" | "recolher")), periodo(ui, c, pendentes))
}

/// O que os botões de período pedem.
#[derive(Clone, Copy, PartialEq)]
pub enum Periodo {
    Todo,
    Hoje,
    Pendentes,
}

/// Os filtros de período. «Em andamento» (que depende dos agentes) ainda não filtra. `pendentes`: o filtro está ligado.
fn periodo(ui: &mut Ui, c: &Cena, pendentes: bool) -> Option<Periodo> {
    let mut comando = None;
    let p = ui.painter().clone();
    let (mut x, y) = (c.o.x + 70.0, c.o.y + 124.0);
    for rotulo in ["Todo o blueprint", "Hoje", "Pendentes", "Em andamento"] {
        let r = if rotulo == "Em andamento" { format!("{} · 0", tr(rotulo)) } else { tr(rotulo).to_string() };
        let b = Botao { texto: &r, tam: 10.0, cheio: rotulo == "Pendentes" && pendentes, ativo: true, altura: 27.0, pad_x: 12.0 };
        let largura = b.largura(&p);
        if botao(ui, pos2(x, y), &b).clicked() {
            comando = match rotulo {
                "Todo o blueprint" => Some(Periodo::Todo),
                "Hoje" => Some(Periodo::Hoje),
                "Pendentes" => Some(Periodo::Pendentes),
                _ => None,
            };
        }
        x += largura + 14.0;
    }
    comando
}

fn tela_cheia(ui: &Ui) {
    let cheia = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!cheia));
}

/// A estrela de quatro pontas do «✦ Planejar» (a fonte não tem esse glifo).
pub(crate) fn estrela(p: &egui::Painter, c: Pos2, invertido: bool) {
    let (fora, dentro) = (5.0, 1.4);
    let pts = (0..8).map(|i| {
        let ang = std::f32::consts::FRAC_PI_4 * i as f32 - std::f32::consts::FRAC_PI_2;
        c + vec2(ang.cos(), ang.sin()) * if i % 2 == 0 { fora } else { dentro }
    }).collect();
    p.add(Shape::convex_polygon(pts, if invertido { PAPEL } else { TINTA }, Stroke::NONE));
}

/// Os botões quadrados de 34 px, com o desenho (simplificado) de cada ícone.
fn icone(ui: &mut Ui, pos: Pos2, nome: &str) -> Response {
    let p = ui.painter().clone();
    let r = Rect::from_min_size(pos, vec2(34.0, 34.0));
    let resposta = ui.interact(r, ui.scope_id().with(("icone", nome)), Sense::click());
    let invertido = resposta.hovered() || resposta.has_focus();
    p.rect_filled(r, 0, if invertido { TINTA } else { rgba(0x0c3a78, 0xcc) });
    planta::borda(&p, r, MEDIO);
    let cor = if invertido { PAPEL } else { TINTA };
    let (cx, cy) = (r.center().x, r.center().y);
    let traco = Stroke::new(1.5, cor);
    let linha = |a: (f32, f32), b: (f32, f32)| p.line_segment([pos2(cx + a.0, cy + a.1), pos2(cx + b.0, cy + b.1)], traco);
    match nome {
        "voltar" | "voltar-bp" => {
            linha((6.0, 0.0), (-6.0, 0.0));
            linha((-6.0, 0.0), (-2.0, -4.0));
            linha((-6.0, 0.0), (-2.0, 4.0));
        }
        "buscar" => {
            p.circle_stroke(pos2(cx - 1.5, cy - 1.5), 5.0, traco);
            linha((2.0, 2.0), (7.0, 7.0));
        }
        "expandir" | "recolher" => {
            let s = if nome == "expandir" { 1.0 } else { -1.0 };
            for dy in [-4.0f32, 2.0] {
                linha((-4.0, (dy - 1.0) * s), (0.0, (dy + 3.0) * s));
                linha((0.0, (dy + 3.0) * s), (4.0, (dy - 1.0) * s));
            }
        }
        "tela" => {
            for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                linha((sx * 7.0, sy * 7.0), (sx * 3.0, sy * 7.0));
                linha((sx * 7.0, sy * 7.0), (sx * 7.0, sy * 3.0));
            }
        }
        "foco" => {
            p.circle_stroke(pos2(cx, cy), 5.0, traco);
            p.circle_filled(pos2(cx, cy), 1.5, cor);
            for (dx, dy) in [(0.0f32, -1.0f32), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)] {
                linha((dx * 5.0, dy * 5.0), (dx * 9.0, dy * 9.0));
            }
        }
        "planejar" => estrela(&p, pos2(cx, cy), invertido),
        "compartilhar" => {
            for (dx, dy) in [(-5.0f32, 0.0f32), (5.0, -5.0), (5.0, 5.0)] {
                p.circle_stroke(pos2(cx + dx, cy + dy), 2.2, traco);
            }
            linha((-3.0, -1.0), (3.0, -4.0));
            linha((-3.0, 1.0), (3.0, 4.0));
        }
        _ => {
            p.rect_stroke(Rect::from_center_size(pos2(cx, cy - 1.0), vec2(14.0, 10.0)), 0, traco, egui::StrokeKind::Middle);
            linha((-3.0, 7.0), (3.0, 7.0));
            linha((-1.0, -4.0), (3.0, -1.0));
            linha((3.0, -1.0), (-1.0, 2.0));
        }
    }
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resposta
}
