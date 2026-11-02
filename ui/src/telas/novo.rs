//! «Novo workspace»: uma prancha. O nome é escrito grande direto na folha e o que se digita aparece no carimbo, ao lado da figura (`novo_palco.rs`).
//! Protótipo aprovado em `design/novo-workspace/`. Não há escolha de arte: todo workspace novo nasce com a figura do Blueprint.

use super::{novo_palco, Acao, Vista};
use crate::fonte::{familia, Peso};
use crate::i18n::tr;
use crate::planta::{self, botao, Botao, Papel, Texto};
use crate::tema::{rgba, BAIXO, ERRO, MEDIO, TINTA};
use egui::{pos2, vec2, CursorIcon, FontId, Rect, Sense, Stroke, Ui};

/// A marca de todo workspace criado aqui (`icone` nos dados).
const ICONE: &str = "blueprint";
/// Abaixo desta largura só o formulário aparece.
const COM_PALCO: f32 = 960.0;
const COLUNA: f32 = 444.0;
const ALTURA_MIN: f32 = 620.0;
/// As alturas do rótulo ao botão: rótulo, nome, descrição e botão com os vãos.
const NOME_Y: f32 = 23.0;
const NOME_H: f32 = 52.0;
const DESC_Y: f32 = 89.0;
const DESC_H: f32 = 52.0;
const BOTAO_Y: f32 = 177.0;
const BLOCO_H: f32 = 223.0;
const MONTAGEM: f64 = 0.7;

pub struct Novo {
    pub nome: String,
    pub descricao: String,
    pub erro: String,
    /// quando a tela abriu, para a figura se montar
    inicio: Option<f64>,
}

impl Novo {
    pub fn vazio() -> Novo {
        Novo { nome: String::new(), descricao: String::new(), erro: String::new(), inicio: None }
    }

    /// Para o roteiro de teste, que só existe no programa nativo.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub fn com_nome(nome: &str) -> Novo {
        Novo { nome: nome.to_string(), ..Novo::vazio() }
    }
}

pub fn tela(ui: &mut Ui, vista: &mut Vista, ocupado: bool) -> Option<Acao> {
    let mut acao = None;
    let (viewport, largura) = (ui.available_height(), ui.available_width());
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let altura = viewport.max(ALTURA_MIN);
        let (pagina, _) = ui.allocate_exact_size(vec2(largura, altura), Sense::hover());
        let (p, o) = (ui.painter().clone(), pagina.min);
        planta::papel(&p, pagina, Papel::Entrada);
        let com_palco = largura >= COM_PALCO;
        let x = o.x + if largura > 600.0 { 64.0 } else { 24.0 };
        let w = COLUNA.min(largura - 2.0 * (x - o.x));
        if barra(ui, x, o.y + 24.0, w) || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            vista.novo = None;
            return;
        }
        let Some(novo) = vista.novo.as_mut() else { return };
        let agora = ui.input(|i| i.time);
        let primeiro = novo.inicio.is_none();
        let inicio = *novo.inicio.get_or_insert(agora);
        let y = (o.y + ((altura - BLOCO_H) / 2.0).round()).max(o.y + 72.0);
        Texto::novo(tr("Novo workspace").to_uppercase(), 10.0).cor(MEDIO).trilha(0.14).na_linha(&p, x, y, 13.0);
        let (resposta, enviar) = nome(ui, Rect::from_min_size(pos2(x, y + NOME_Y), vec2(w, NOME_H)), novo);
        if primeiro {
            resposta.request_focus();
        }
        descricao(ui, Rect::from_min_size(pos2(x, y + DESC_Y), vec2(w, DESC_H)), &mut novo.descricao);
        let criar = Botao::entrada(tr("Criar workspace")).cheio(true).ativo(!ocupado);
        let pos = pos2(x, y + BOTAO_Y);
        let clicou = botao(ui, pos, &criar).clicked();
        let depois = pos.x + criar.largura(&p) + 14.0;
        if novo.erro.is_empty() {
            Texto::novo(tr("Enter"), 10.0).cor(BAIXO).trilha(0.08).na_linha(&p, depois, pos.y, criar.altura);
        } else {
            Texto::novo(&*novo.erro, 12.0).cor(ERRO).na_linha(&p, depois, pos.y, criar.altura);
        }
        if (clicou || enviar) && !ocupado {
            if novo.nome.trim().is_empty() {
                novo.erro = tr("Digite o nome do workspace.").to_string();
                resposta.request_focus();
            } else {
                acao = Some(Acao::CriarWorkspace { nome: novo.nome.trim().to_string(), descricao: novo.descricao.trim().to_string(), icone: ICONE.to_string() });
            }
        }
        if com_palco {
            let palco = Rect::from_min_max(pos2(x + w + 40.0, o.y + 40.0), pos2(o.x + largura - 64.0, o.y + altura - 40.0));
            let t = ((agora - inicio) / MONTAGEM).clamp(0.0, 1.0) as f32;
            if t < 1.0 {
                ui.ctx().request_repaint();
            }
            novo_palco::desenhar(ui, palco, &novo.nome, &novo.descricao, t);
        }
    });
    acao
}

/// A linha de cima: «← WORKSPACES» e o lembrete do Esc. Devolve `true` no clique em voltar.
fn barra(ui: &mut Ui, x: f32, y: f32, w: f32) -> bool {
    let p = ui.painter().clone();
    let rotulo = format!("← {}", tr("Workspaces")).to_uppercase();
    let medida = Texto::novo(rotulo.as_str(), 11.0).trilha(0.12).largura(&p);
    let r = Rect::from_min_size(pos2(x, y), vec2(medida, 32.0));
    let resposta = ui.interact(r, ui.scope_id().with("voltar"), Sense::click());
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    Texto::novo(rotulo.as_str(), 11.0).trilha(0.12).cor(if resposta.hovered() { TINTA } else { MEDIO }).na_linha(&p, x, y, 32.0);
    Texto::novo(tr("Esc para voltar"), 10.0).cor(BAIXO).trilha(0.08).na_linha_dir(&p, x + w, y, 32.0);
    resposta.clicked()
}

/// O nome: letra grande sobre um fio, que acende com o foco e fica em tom de alerta com erro. Devolve o campo e se apertou Enter.
fn nome(ui: &mut Ui, r: Rect, novo: &mut Novo) -> (egui::Response, bool) {
    let fonte = FontId::new(36.0, familia(Peso::Medio));
    // a dica é pintada aqui porque a do `TextEdit` sai acinzentada, fora do tom da folha
    if novo.nome.is_empty() {
        Texto::novo(tr("Como vai se chamar?"), 36.0).peso(Peso::Medio).cor(BAIXO).em(ui.painter(), r.min);
    }
    let editor = egui::TextEdit::singleline(&mut novo.nome).id(ui.scope_id().with("novo-nome")).char_limit(80).frame(egui::Frame::NONE).margin(egui::Margin::ZERO).font(fonte).text_color(TINTA).desired_width(r.width());
    let resposta = ui.put(r, editor);
    if resposta.changed() {
        novo.erro.clear();
    }
    let traco = if !novo.erro.is_empty() { Stroke::new(2.0, ERRO) } else if resposta.has_focus() { Stroke::new(2.0, TINTA) } else { Stroke::new(1.0, BAIXO) };
    ui.painter().hline(r.x_range(), r.max.y, traco);
    let enviar = resposta.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    (resposta, enviar)
}

/// A descrição: duas linhas, sem caixa, que clareiam com o foco.
fn descricao(ui: &mut Ui, r: Rect, texto: &mut String) {
    let id = ui.scope_id().with("novo-descricao");
    let cor = if ui.memory(|m| m.has_focus(id)) { TINTA } else { MEDIO };
    if texto.is_empty() {
        Texto::novo(tr("Para que serve? (opcional)"), 14.0).cor(BAIXO).em(ui.painter(), r.min);
    }
    let editor = egui::TextEdit::multiline(texto).id(id).char_limit(200).desired_rows(2).frame(egui::Frame::NONE).margin(egui::Margin::ZERO).font(FontId::new(14.0, familia(Peso::Normal))).text_color(cor).desired_width(r.width());
    ui.put(r, editor);
}

/// Um campo de uma linha como foi digitado (os do novo blueprint): fundo escuro, borda fina, letra de 13 px. Devolve `true` se a pessoa apertou Enter nele.
pub(super) fn campo_linha(ui: &mut Ui, r: Rect, texto: &mut String, dica: &str) -> bool {
    let p = ui.painter().clone();
    p.rect_filled(r, 0, rgba(0x082b5e, 0xcc));
    planta::borda(&p, r, BAIXO);
    let fonte = FontId::new(13.0, familia(Peso::Normal));
    let dica = egui::RichText::new(dica).size(13.0).color(BAIXO);
    // 11 px dos lados e 9 em cima e embaixo, mais a borda de 1 px
    let interno = r.shrink2(vec2(12.0, 10.0));
    let editor = egui::TextEdit::singleline(texto).frame(egui::Frame::NONE).font(fonte).text_color(TINTA).hint_text(dica).desired_width(interno.width()).align(egui::Align2::LEFT_TOP);
    let resposta = ui.put(interno, editor);
    resposta.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
}
