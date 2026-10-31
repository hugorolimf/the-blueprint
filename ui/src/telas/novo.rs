//! «Novo workspace»: a tela cheia de criação, com o formulário à esquerda e a prévia da arte à direita (`#workspace-nova` em entrada.css).
//! Só a arte «Marca pronta» existe aqui; o texto (sigla) e as peças em JSON dependem de desenhar peças isométricas, que vêm com o kit.

use super::{Acao, Vista};
use crate::emblema::{self, Tamanho};
use crate::fonte::{familia, Peso};
use crate::i18n::tr;
use crate::planta::{self, botao, Botao, Papel, Texto};
use crate::tema::{rgba, BAIXO, ERRO, MEDIO, TINTA};
use egui::{pos2, vec2, Rect, Sense, Ui};

/// As marcas prontas (as de `MARCAS_PRONTAS` em arte.js).
pub const MARCAS: [(&str, &str); 5] = [("blueprint", "Blueprint (padrão)"), ("moedas", "Moedas"), ("livros", "Livros"), ("chip", "Chip"), ("caminhao", "Caminhão")];
const MODOS: [(&str, bool); 3] = [("Marca pronta", true), ("Texto (sigla)", false), ("Peças em texto (JSON)", false)];

pub struct Novo {
    pub nome: String,
    pub descricao: String,
    pub icone: usize,
    pub erro: String,
}

impl Novo {
    pub fn vazio() -> Novo {
        Novo { nome: String::new(), descricao: String::new(), icone: 0, erro: String::new() }
    }
}

pub fn tela(ui: &mut Ui, vista: &mut Vista, ocupado: bool) -> Option<Acao> {
    let mut acao = None;
    let (viewport, largura) = (ui.available_height(), ui.available_width());
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let altura_card = 566.4;
        let topo_card = 64.0 + 28.0f32.max((viewport - 64.0 - altura_card) / 2.0);
        let (pagina, _) = ui.allocate_exact_size(vec2(largura, viewport.max(topo_card + altura_card + 28.0)), Sense::hover());
        let (p, o) = (ui.painter().clone(), pagina.min);
        planta::papel(&p, pagina, Papel::Entrada);
        barra(ui, vista, o, largura, ocupado);
        let Some(novo) = vista.novo.as_mut() else { return };
        // o cartão do formulário
        let card = Rect::from_min_size(pos2(o.x + 24.0, o.y + topo_card), vec2(largura - 48.0, altura_card));
        p.rect_filled(card, 0, rgba(0x0c3a78, 0x70));
        planta::borda(&p, card, BAIXO);
        let (gx, gy) = (card.min.x + 35.0, card.min.y + 31.0);
        let esquerda = (card.width() - 70.0 - 560.0 - 36.0).max(240.0);
        // rótulo + campo, de cima para baixo (as posições vêm da tela web)
        let rotulo = |ui: &Ui, texto: &str, y: f32| {
            Texto::novo(texto.to_uppercase(), 10.0).cor(MEDIO).trilha(0.14).na_linha(ui.painter(), gx, gy + y, 13.0);
        };
        rotulo(ui, tr("Nome do workspace"), 0.0);
        let nome = Rect::from_min_size(pos2(gx, gy + 21.0), vec2(esquerda, 37.0));
        let enviar = campo(ui, nome, &mut novo.nome, tr("Nome do workspace"), false);
        rotulo(ui, tr("Descrição (opcional)"), 74.0);
        campo(ui, Rect::from_min_size(pos2(gx, gy + 95.0), vec2(esquerda, 88.0)), &mut novo.descricao, tr("Breve descrição do workspace"), true);
        rotulo(ui, tr("Arte do projeto"), 199.0);
        seletor(ui, Rect::from_min_size(pos2(gx, gy + 220.0), vec2(esquerda, 39.0)), "modo", &MODOS.iter().map(|(t, ok)| (tr(t).to_string(), *ok)).collect::<Vec<_>>(), &mut 0);
        rotulo(ui, tr("Marca"), 275.0);
        seletor(ui, Rect::from_min_size(pos2(gx, gy + 296.0), vec2(esquerda, 39.0)), "marca", &MARCAS.iter().map(|(_, t)| (tr(t).to_string(), true)).collect::<Vec<_>>(), &mut novo.icone);
        // a prévia
        let previa = Rect::from_min_size(pos2(gx + esquerda + 36.0, gy), vec2(560.0, 448.4));
        p.rect_filled(previa, 0, rgba(0x082b5e, 0x99));
        planta::tracejado(&p, previa, BAIXO);
        emblema::pintar(ui, Rect::from_min_size(previa.min + vec2(23.0, 23.0), vec2(514.0, 375.6)), MARCAS[novo.icone].0, Tamanho::Previa);
        Texto::novo(tr("Prévia da arte").to_uppercase(), 10.0).cor(MEDIO).trilha(0.14).centro(&p, pos2(previa.center().x, previa.max.y - 30.0));
        // criar
        let criar = Botao::barra(tr("Criar workspace")).cheio(true).ativo(!ocupado);
        let largura_criar = criar.largura(&p);
        let clicou = botao(ui, pos2(card.center().x - largura_criar / 2.0, gy + 470.4), &criar).clicked();
        if (clicou || enviar) && !ocupado {
            if novo.nome.trim().is_empty() {
                novo.erro = tr("Digite o nome do workspace.").to_string();
            } else {
                acao = Some(Acao::CriarWorkspace { nome: novo.nome.clone(), descricao: novo.descricao.clone(), icone: MARCAS[novo.icone].0.to_string() });
            }
        }
        if !novo.erro.is_empty() {
            Texto::novo(&*novo.erro, 12.0).cor(ERRO).centro(&p, pos2(card.center().x, gy + 470.4 + 34.0 + 18.0));
        }
    });
    acao
}

/// A barra de cima: «← WORKSPACES», o título, o estado e «TELA CHEIA».
fn barra(ui: &mut Ui, vista: &mut Vista, o: egui::Pos2, largura: f32, ocupado: bool) {
    let p = ui.painter().clone();
    let r = Rect::from_min_size(o, vec2(largura, 64.0));
    p.rect_filled(r, 0, rgba(0x082b5e, 0xe8));
    p.rect_filled(Rect::from_min_size(pos2(o.x, o.y + 62.0), vec2(largura, 2.0)), 0, TINTA);
    if botao(ui, pos2(o.x + 24.0, o.y + 14.0), &Botao::barra(&format!("← {}", tr("Workspaces")))).clicked() {
        vista.novo = None;
    }
    Texto::novo(tr("Novo workspace").to_uppercase(), 16.0).peso(Peso::Negrito).trilha(0.1).na_linha(&p, o.x + 162.4, o.y + 14.0, 34.0);
    let tela_cheia = Botao::barra("∷ Tela cheia");
    let x = o.x + largura - 24.0 - tela_cheia.largura(&p);
    if botao(ui, pos2(x, o.y + 14.0), &tela_cheia).clicked() {
        let cheia = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!cheia));
    }
    let estado = if ocupado { tr("salvando…") } else if vista.novo.as_ref().is_some_and(|n| !n.erro.is_empty()) { tr("erro") } else { tr("pronto") };
    let t = Texto::novo(estado.to_uppercase(), 10.0).cor(MEDIO).trilha(0.12);
    t.na_linha_dir(&p, x - 14.0, o.y + 14.0, 34.0);
}

/// Um campo de texto no estilo da página: fundo escuro, borda fina, letra de 13 px com 0,03em e em caixa alta quando é de uma linha.
/// Devolve `true` se a pessoa apertou Enter nele.
fn campo(ui: &mut Ui, r: Rect, texto: &mut String, dica: &str, multilinha: bool) -> bool {
    let p = ui.painter().clone();
    p.rect_filled(r, 0, rgba(0x082b5e, 0xcc));
    planta::borda(&p, r, BAIXO);
    let fonte = egui::FontId::new(13.0, familia(Peso::Normal));
    let dica = egui::RichText::new(if multilinha { dica.to_string() } else { dica.to_uppercase() }).size(13.0).color(BAIXO);
    // o nome aparece em caixa alta, como na página (`text-transform: uppercase`); o valor guardado fica como foi digitado
    let mut maiusculo = |ui: &egui::Ui, t: &dyn egui::TextBuffer, _largura: f32| {
        let mut job = egui::text::LayoutJob::simple(t.as_str().to_uppercase(), egui::FontId::new(13.0, familia(Peso::Normal)), TINTA, f32::INFINITY);
        job.wrap.max_width = f32::INFINITY;
        ui.fonts_mut(|f| f.layout_job(job))
    };
    // 11 px dos lados e 9 em cima e embaixo, mais a borda de 1 px
    let interno = r.shrink2(vec2(12.0, 10.0));
    let editor = if multilinha { egui::TextEdit::multiline(texto).desired_rows(3) } else { egui::TextEdit::singleline(texto).char_limit(80).layouter(&mut maiusculo) };
    let editor = editor.frame(egui::Frame::NONE).font(fonte).text_color(TINTA).hint_text(dica).desired_width(interno.width()).align(egui::Align2::LEFT_TOP);
    let resposta = ui.put(interno, editor);
    resposta.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !multilinha
}

/// O `<select>`: o valor atual e a seta; ao clicar abre a lista. Itens `(texto, disponível)`.
fn seletor(ui: &mut Ui, r: Rect, id: &str, itens: &[(String, bool)], atual: &mut usize) {
    let p = ui.painter().clone();
    let resposta = ui.interact(r, ui.scope_id().with(id), Sense::click());
    p.rect_filled(r, 0, rgba(0x082b5e, 0xcc));
    planta::borda(&p, r, BAIXO);
    Texto::novo(itens.get(*atual).map_or("", |i| i.0.as_str()), 13.0).na_linha(&p, r.min.x + 16.0, r.min.y, r.height());
    let c = pos2(r.max.x - 20.0, r.center().y);
    p.line_segment([c + vec2(-4.0, -2.0), c + vec2(0.0, 2.0)], egui::Stroke::new(1.5, TINTA));
    p.line_segment([c + vec2(0.0, 2.0), c + vec2(4.0, -2.0)], egui::Stroke::new(1.5, TINTA));
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    egui::Popup::menu(&resposta).width(r.width()).show(|ui| {
        for (i, (texto, ok)) in itens.iter().enumerate() {
            if ui.add_enabled(*ok, egui::Button::selectable(*atual == i, texto.as_str())).clicked() {
                *atual = i;
            }
        }
    });
}
