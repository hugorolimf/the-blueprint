//! «Novo blueprint»: o editor da página aberto para um blueprint que ainda não existe (`#painel-novo-manual` de blueprint.html):
//! a barra de cima e uma faixa com o nome, a descrição curta e «Criar blueprint vazio». Criado, ele abre na folha.

use super::novo::campo_linha;
use super::{Acao, Vista};
use crate::fonte::Peso;
use crate::i18n::tr;
use crate::planta::{self, botao, Botao, Papel, Texto};
use crate::tema::{rgba, BAIXO, ERRO, MEDIO, TINTA};
use egui::{pos2, vec2, Rect, Sense, Ui};

pub struct NovoBp {
    pub workspace: String,
    pub nome: String,
    pub descricao: String,
    pub erro: String,
}

impl NovoBp {
    pub fn em(workspace: &str) -> NovoBp {
        NovoBp { workspace: workspace.to_string(), nome: String::new(), descricao: String::new(), erro: String::new() }
    }
}

pub fn tela(ui: &mut Ui, vista: &mut Vista, ocupado: bool) -> Option<Acao> {
    let mut acao = None;
    let (altura, largura) = (ui.available_height(), ui.available_width());
    let (pagina, _) = ui.allocate_exact_size(vec2(largura, altura), Sense::hover());
    let (p, o) = (ui.painter().clone(), pagina.min);
    planta::papel(&p, pagina, Papel::Entrada);
    barra(ui, vista, o, largura, ocupado);
    let Some(novo) = vista.novo_bp.as_mut() else { return None };
    // a faixa `.nv`: 1fr 1.5fr auto, 14 px entre as colunas, 16 x 18 de respiro, encostada embaixo
    let faixa = Rect::from_min_size(pos2(o.x + 24.0, o.y + 64.0 + 18.0), vec2(largura - 48.0, 92.0));
    p.rect_filled(faixa, 0, rgba(0x0c3a78, 0x70));
    planta::borda(&p, faixa, BAIXO);
    let criar = Botao::barra(tr("Criar blueprint vazio")).ativo(!ocupado);
    let largura_criar = criar.largura(&p);
    let (x0, topo) = (faixa.min.x + 19.0, faixa.min.y + 17.0);
    let livre = (faixa.width() - 36.0 - 2.0 - largura_criar - 28.0).max(200.0);
    let (w_nome, w_desc) = (livre / 2.5, livre * 1.5 / 2.5);
    let x_desc = x0 + w_nome + 14.0;
    let rotulo = |t: &str, x: f32| Texto::novo(t.to_uppercase(), 10.0).cor(MEDIO).trilha(0.14).escrita().na_linha(&p, x, topo, 13.0);
    rotulo(tr("Nome do blueprint"), x0);
    rotulo(tr("O que ele é? (descrição curta)"), x_desc);
    let y_campo = topo + 13.0 + 8.0;
    let enter_nome = campo_linha(ui, Rect::from_min_size(pos2(x0, y_campo), vec2(w_nome, 37.0)), &mut novo.nome, tr("Ex.: Portal do fornecedor"));
    let enter_desc = campo_linha(ui, Rect::from_min_size(pos2(x_desc, y_campo), vec2(w_desc, 37.0)), &mut novo.descricao, tr("Para que serve"));
    let clicou = botao(ui, pos2(x_desc + w_desc + 14.0, y_campo + 3.0), &criar).clicked();
    if (clicou || enter_nome || enter_desc) && !ocupado {
        if novo.nome.trim().is_empty() {
            novo.erro = tr("Defina o nome do blueprint antes de criar.").to_string();
        } else {
            acao = Some(Acao::CriarBlueprint { workspace: novo.workspace.clone(), nome: novo.nome.clone(), descricao: novo.descricao.clone() });
        }
    }
    if !novo.erro.is_empty() {
        Texto::novo(&*novo.erro, 12.0).cor(ERRO).em(&p, pos2(faixa.min.x, faixa.max.y + 14.0));
    }
    acao
}

/// A barra de cima (`.ed-topo`): «← VOLTAR AO BLUEPRINT», o rótulo e o nome que vai sendo digitado, o estado e «TELA CHEIA».
fn barra(ui: &mut Ui, vista: &mut Vista, o: egui::Pos2, largura: f32, ocupado: bool) {
    let p = ui.painter().clone();
    p.rect_filled(Rect::from_min_size(o, vec2(largura, 64.0)), 0, rgba(0x082b5e, 0xe8));
    p.rect_filled(Rect::from_min_size(pos2(o.x, o.y + 62.0), vec2(largura, 2.0)), 0, TINTA);
    let voltar = Botao::barra(tr("← Voltar ao blueprint"));
    if botao(ui, pos2(o.x + 24.0, o.y + 14.0), &voltar).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        vista.novo_bp = None;
        return;
    }
    let x = o.x + 24.0 + voltar.largura(&p) + 14.0;
    let nome = vista.novo_bp.as_ref().map(|n| n.nome.trim().to_string()).filter(|n| !n.is_empty()).unwrap_or_else(|| tr("Novo blueprint").to_string());
    Texto::novo(tr("Novo blueprint").to_uppercase(), 10.0).cor(MEDIO).trilha(0.16).na_linha(&p, x, o.y + 13.0, 13.0);
    let tela_cheia = Botao::barra("∷ Tela cheia");
    let x_tela = o.x + largura - 24.0 - tela_cheia.largura(&p);
    let estado = if ocupado { tr("salvando…") } else if vista.novo_bp.as_ref().is_some_and(|n| !n.erro.is_empty()) { tr("erro") } else { tr("pronto") };
    let t = Texto::novo(estado.to_uppercase(), 10.0).cor(MEDIO).trilha(0.12);
    let r_estado = t.na_linha_dir(&p, x_tela - 14.0, o.y + 14.0, 34.0);
    let caixa = (r_estado.min.x - 14.0 - x).max(40.0);
    Texto::novo(nome.to_uppercase(), 17.0).peso(Peso::Negrito).trilha(0.06).caixa(caixa).linhas(1).na_linha(&p, x, o.y + 26.0, 22.0);
    if botao(ui, pos2(x_tela, o.y + 14.0), &tela_cheia).clicked() {
        let cheia = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!cheia));
    }
}
