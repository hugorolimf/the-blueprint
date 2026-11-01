//! «Bom ter você por aqui.»: a folha de boas-vindas que pede o nome (`.login` em `web/css/entrada.css`). O nome vai para `usuario.nome`.

use super::{Acao, Vista};
use crate::documento::Documento;
use crate::emblema::{self, Tamanho};
use crate::fonte::Peso;
use crate::i18n::tr;
use crate::planta::{self, Botao, Papel, Texto};
use crate::tema::{rgba, BAIXO, ERRO, MEDIO};
use egui::{pos2, vec2, Align2, Color32, Rect, Sense, Ui};

/// A cor padrão do texto de exemplo dos campos no Chromium.
const PLACEHOLDER: Color32 = Color32::from_rgb(117, 117, 117);

pub fn tela(ui: &mut Ui, vista: &mut Vista, doc: &Documento, ocupado: bool) -> Option<Acao> {
    let mut acao = None;
    let pagina = ui.available_rect_before_wrap();
    ui.allocate_rect(pagina, Sense::hover());
    let p = ui.painter().clone();
    planta::papel(&p, pagina, Papel::Login);
    if vista.nome.is_empty() && !vista.trocando_nome {
        vista.nome = doc.nome_usuario().to_string();
    }
    // a folha: 1000 de largura (menos que isso, a janela inteira) e 488,6 de altura, no meio da tela
    let (largura, altura) = ((pagina.width() - 48.0).min(1000.0), 488.6);
    let folha = Rect::from_center_size(pagina.center(), vec2(largura, altura)).translate(vec2(0.0, 0.0));
    p.add(egui::epaint::Shadow { offset: [0, 28], blur: 90, spread: 0, color: rgba(0x051125, 0x66) }.as_shape(folha, 0));
    p.rect_filled(folha, 0, rgba(0x113360, 0x77));
    planta::borda(&p, folha, MEDIO);
    Texto::novo("+", 25.0).cor(MEDIO).na_linha(&p, folha.min.x - 9.0, folha.min.y - 14.0, 25.0);
    Texto::novo("+", 25.0).cor(MEDIO).na_linha(&p, folha.max.x + 9.0 - 15.0, folha.max.y + 12.0 - 25.0, 25.0);
    let metade = (largura - 2.0) / 2.0;
    let (esq, dir) = (folha.min.x + 1.0, folha.min.x + 1.0 + metade);
    // coluna da ilustração
    p.rect_filled(Rect::from_min_size(pos2(dir - 1.0, folha.min.y + 1.0), vec2(1.0, altura - 2.0)), 0, BAIXO);
    let (ex, ew) = (esq + 32.0, metade - 64.0);
    Texto::novo("THE BLUEPRINT / WORKSPACE", 13.0).trilha(0.2).na_linha(&p, ex, folha.min.y + 33.0, 20.8);
    emblema::pintar(ui, Rect::from_min_size(pos2(ex, folha.min.y + 106.7), vec2(ew, 280.0)), "blueprint", Tamanho::Grande);
    Texto::novo(tr("Fig. 01 · Do plano à próxima entrega").to_uppercase(), 10.0).cor(MEDIO).trilha(0.13).na_linha(&p, ex, folha.min.y + 439.6, 16.0);
    // coluna do formulário
    let (fx, fw) = (dir + 40.0, metade - 80.0);
    Texto::novo(tr("Seu próximo blueprint começa aqui").to_uppercase(), 10.0).cor(MEDIO).trilha(0.16).na_linha(&p, fx, folha.min.y + 59.0, 13.0);
    Texto::novo(tr("Bom ter você por aqui."), 32.0).peso(Peso::Medio).trilha(-0.04).caixa(fw).escrita().na_linha(&p, fx, folha.min.y + 91.4, 41.6);
    Texto::novo(format!("{}\n{}", tr("Como podemos chamar você?"), tr("Depois, é só escolher seu espaço e continuar.")), 12.0).cor(MEDIO).lh(21.6).caixa(fw).escrita().em(&p, pos2(fx, folha.min.y + 149.0));
    Texto::novo(tr("Seu nome"), 11.0).escrita().na_linha(&p, fx, folha.min.y + 225.2, 14.0);
    let campo = Rect::from_min_size(pos2(fx, folha.min.y + 252.6), vec2(fw, 53.0));
    p.rect_filled(campo, 0, rgba(0x0c264b, 0xbb));
    planta::borda(&p, campo, MEDIO);
    // o texto fica 16 px dentro da borda, como o `padding: 15px` + borda da página
    let interno = campo.shrink2(vec2(16.0, 1.0));
    let editor = egui::TextEdit::singleline(&mut vista.nome)
        .frame(egui::Frame::NONE)
        .char_limit(80)
        .font(egui::FontId::new(16.0, crate::fonte::familia(Peso::Normal)))
        .text_color(crate::tema::TINTA)
        .hint_text(egui::RichText::new(tr("Digite seu nome")).size(16.0).color(PLACEHOLDER))
        .desired_width(interno.width())
        .align(Align2::LEFT_CENTER);
    let r = ui.put(interno, editor);
    let enviar = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    // o botão principal ocupa a largura do formulário e está sempre disponível: sem nome, a pergunta é «Digite seu nome.»
    let entrar = Botao { texto: tr("Entrar no meu espaço →"), tam: 11.0, cheio: true, ativo: !ocupado, altura: 46.0, pad_x: 18.0 };
    let clicou = planta::botao_com_largura(ui, pos2(fx, folha.min.y + 325.6), fw, &entrar).clicked();
    if (clicou || enviar) && !ocupado {
        if vista.nome.trim().is_empty() {
            vista.erro = tr("Digite seu nome.").to_string();
        } else {
            vista.erro.clear();
            acao = Some(Acao::DefinirNome(vista.nome.trim().to_string()));
            vista.trocando_nome = false;
        }
    }
    Texto::novo(tr("Seu nome fica salvo junto dos blueprints."), 10.0).cor(MEDIO).na_linha(&p, fx, folha.min.y + 391.6, 16.0);
    if !vista.erro.is_empty() {
        Texto::novo(&*vista.erro, 12.0).cor(ERRO).caixa(fw).em(&p, pos2(fx, folha.min.y + 420.0));
    }
    if vista.trocando_nome && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        vista.trocando_nome = false;
        vista.nome.clear();
    }
    acao
}
