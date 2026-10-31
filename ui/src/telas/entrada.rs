//! «Seus workspaces»: a primeira tela depois do nome, desenhada no papel de planta da página (`web/css/entrada.css`, `primeiro-uso.css`).
//! Tudo é posicionado em pixels do CSS a partir do canto da página, com as medidas tiradas da própria tela web.

use super::{Acao, Tela, Vista};
use crate::documento::{Documento, Resumo, RAIZ};
use crate::emblema;
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com, trn};
use crate::planta::{self, botao, canto, Botao, Papel, Texto};
use crate::tema::{rgba, BAIXO, ERRO, MEDIO, PAPEL, TENUE, TINTA};
use egui::{pos2, vec2, Align2, Pos2, Rect, Sense, Ui};

const PAD_X: f32 = 64.0;
const CARTAO: egui::Vec2 = vec2(320.0, 360.0);
const FOLGA: f32 = 24.0;

/// O que aparece entre o título e a grade.
enum Bloco {
    Nada,
    /// a linha «O workspace X é o exemplo»
    Exemplo(Resumo),
    /// os três caminhos de quem abre o app vazio
    Passos(PassosMedidos),
}

/// A altura dos três cartões de primeiros passos depende do quanto o texto quebra na largura da janela.
struct PassosMedidos {
    largura: f32,
    altura_cartao: f32,
}

pub fn tela(ui: &mut Ui, vista: &mut Vista, doc: &Documento, servidor: &str, ocupado: bool) -> Option<Acao> {
    let mut acao = None;
    let viewport = ui.available_height();
    let largura = ui.available_width();
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        let util = (largura - 2.0 * PAD_X).max(320.0);
        let ws = doc.workspaces();
        let p = ui.painter().clone();
        let bloco = if !doc.tem_blueprint() { Bloco::Passos(medir_passos(&p, util)) } else if let Some(ex) = doc.exemplo() { Bloco::Exemplo(ex) } else { Bloco::Nada };
        // alturas, de cima para baixo (medidas da tela web)
        let (topo_bloco, altura_bloco) = match &bloco {
            Bloco::Nada => (0.0, 0.0),
            Bloco::Exemplo(_) => (322.6, 72.0),
            Bloco::Passos(m) => (322.6, 24.0 + 1.0 + 16.0 + 8.0 + 32.0 + 20.0 + m.altura_cartao + 31.2 + 20.0 + 1.0),
        };
        let status_y = if matches!(bloco, Bloco::Nada) { 314.6 } else { topo_bloco + altura_bloco + 32.0 };
        let grade_y = status_y + 22.4 + 14.0;
        let colunas = (((util + FOLGA) / (CARTAO.x + FOLGA)) as usize).max(1);
        let total = ws.len() + 1;
        let linhas = total.div_ceil(colunas);
        let fim = grade_y + linhas as f32 * CARTAO.y + (linhas - 1) as f32 * FOLGA + 80.0;
        let (pagina, _) = ui.allocate_exact_size(vec2(largura, fim.max(viewport)), Sense::hover());
        planta::papel(&p, pagina, Papel::Entrada);
        let o = pagina.min;
        cabecalho(ui, vista, doc, servidor, o, largura, &mut acao);
        Texto::novo(tr("Seus workspaces"), 42.0).peso(Peso::Medio).trilha(-0.04).na_linha(&p, o.x + PAD_X, o.y + 163.0, 67.2);
        Texto::novo(tr("Escolha um workspace para continuar."), 14.0).cor(MEDIO).na_linha(&p, o.x + PAD_X, o.y + 244.2, 22.4);
        match &bloco {
            Bloco::Nada => {}
            Bloco::Exemplo(ex) => {
                if faixa_do_exemplo(ui, vista, ex, pos2(o.x + PAD_X, o.y + topo_bloco), util, ocupado) {
                    vista.excluir = Some(ex.id.clone());
                }
            }
            Bloco::Passos(m) => {
                if passos(ui, pos2(o.x + PAD_X, o.y + topo_bloco), util, m, ocupado) {
                    acao = Some(Acao::AbrirExemplo);
                }
            }
        }
        let (estado, cor) = if !vista.erro.is_empty() { (vista.erro.clone(), ERRO) } else if !vista.aviso.is_empty() { (vista.aviso.clone(), MEDIO) } else { (trn(ws.len(), "{n} workspace · arraste para reordenar (Alt + setas)", "{n} workspaces · arraste para reordenar (Alt + setas)"), MEDIO) };
        Texto::novo(estado, 14.0).cor(cor).na_linha(&p, o.x + PAD_X, o.y + status_y, 22.4);
        grade(ui, vista, &ws, pos2(o.x + PAD_X, o.y + grade_y), colunas, ocupado, &mut acao);
    });
    acao
}

/// O topo: «WORKSPACES» à esquerda; à direita o avatar, «Olá, nome» e os botões. A linha fina fecha o topo.
fn cabecalho(ui: &mut Ui, vista: &mut Vista, doc: &Documento, servidor: &str, o: Pos2, largura: f32, acao: &mut Option<Acao>) {
    let p = ui.painter().clone();
    let (y, direita) = (o.y + 44.0, o.x + largura - PAD_X);
    Texto::novo(tr("Workspaces").to_uppercase(), 13.0).peso(Peso::Negrito).trilha(0.2).na_linha(&p, o.x + PAD_X, y, 46.0);
    let _ = acao;
    let sair = Botao::entrada(tr("Trocar nome"));
    let roadmap = Botao::entrada(tr("Roadmap"));
    let x_sair = direita - sair.largura(&p);
    if botao(ui, pos2(x_sair, y), &sair).clicked() {
        vista.nome = doc.nome_usuario().to_string();
        vista.trocando_nome = true;
    }
    let x_roadmap = x_sair - 12.0 - roadmap.largura(&p);
    if botao(ui, pos2(x_roadmap, y), &roadmap).clicked() {
        ui.ctx().open_url(egui::OpenUrl::new_tab(format!("{servidor}/#/roadmap")));
    }
    let nome = doc.nome_usuario();
    let ola = Texto::novo(tr("Olá,"), 12.0).cor(MEDIO);
    let forte = Texto::novo(nome, 12.0).peso(Peso::Negrito);
    let (l_ola, l_nome) = (Texto::novo(format!("{} ", tr("Olá,")), 12.0).largura(&p), forte.largura(&p));
    let x_ola = x_roadmap - 12.0 - l_ola - l_nome;
    ola.na_linha(&p, x_ola, y, 46.0);
    forte.na_linha(&p, x_ola + l_ola, y, 46.0);
    let avatar = pos2(x_ola - 12.0 - 20.0, y + 23.0);
    p.circle_stroke(avatar, 19.5, egui::Stroke::new(1.0, MEDIO));
    let inicial: String = nome.chars().take(1).flat_map(char::to_uppercase).collect();
    Texto::novo(inicial, 12.0).centro(&p, avatar);
    p.rect_filled(Rect::from_min_size(pos2(o.x + PAD_X, y + 70.0), vec2(direita - o.x - PAD_X, 1.0)), 0, BAIXO);
}

/// «O workspace X é o exemplo…» com o botão de remover. Devolve `true` no clique do botão.
fn faixa_do_exemplo(ui: &mut Ui, _vista: &Vista, ex: &Resumo, pos: Pos2, util: f32, ocupado: bool) -> bool {
    let p = ui.painter().clone();
    let r = Rect::from_min_size(pos, vec2(util, 72.0));
    p.rect_filled(r, 0, rgba(0x082b5e, 0x99));
    planta::tracejado(&p, r, BAIXO);
    canto(&p, r);
    let frase = tr_com("O workspace «{nome}» é o exemplo: dados fictícios, separados dos seus.", &[("nome", &ex.nome)]);
    let t = Texto::novo(frase, 12.0).cor(MEDIO);
    t.na_linha(&p, r.min.x + 19.0, r.min.y + 13.0, 46.0);
    let b = Botao::entrada(tr("Remover exemplo")).tam(10.0).ativo(!ocupado);
    botao(ui, pos2(r.min.x + 19.0 + t.largura(&p) + 12.0, r.min.y + 13.0), &b).clicked()
}

fn medir_passos(p: &egui::Painter, util: f32) -> PassosMedidos {
    let largura = (util - 2.0 * 29.0 - 2.0 * 16.0) / 3.0;
    let descricoes = [tr(PASSOS[0].2), tr(PASSOS[1].2), tr(PASSOS[2].2)];
    let linhas = descricoes.iter().map(|d| Texto::novo(*d, 12.0).lh(19.2).caixa(largura - 38.0).altura(p)).fold(0.0f32, f32::max);
    // borda 1 + padding 18 + número 24.6 + 8 + título 27.2 + 8 + texto + 8 + seta 22.4 + padding 18 + borda 1
    PassosMedidos { largura, altura_cartao: 2.0 + 36.0 + 24.6 + 8.0 + 27.2 + 8.0 + linhas + 8.0 + 22.4 }
}

const PASSOS: [(&str, &str, &str); 3] = [
    ("01", "Abrir o exemplo", "Um workspace com dois blueprints fictícios, para ver o app cheio. Os seus dados não mudam e ele sai com um clique."),
    ("02", "Conectar seu agente", "Registre o MCP no Claude Code ou no Codex e peça um plano: a proposta aparece aqui para você aplicar."),
    ("03", "Criar um blueprint", "Escreva o primeiro blueprint à mão, com ações, planos e checkpoints."),
];

/// O painel «Primeiros passos». Devolve `true` no clique de «Abrir o exemplo» (os outros dois ainda não existem nesta interface).
fn passos(ui: &mut Ui, pos: Pos2, util: f32, m: &PassosMedidos, ocupado: bool) -> bool {
    let p = ui.painter().clone();
    let altura = 24.0 + 1.0 + 16.0 + 8.0 + 32.0 + 20.0 + m.altura_cartao + 31.2 + 20.0 + 1.0;
    let r = Rect::from_min_size(pos, vec2(util, altura));
    p.rect_filled(r, 0, rgba(0x082b5e, 0x99));
    planta::tracejado(&p, r, BAIXO);
    canto(&p, r);
    let x = r.min.x + 29.0;
    Texto::novo(tr("Primeiros passos").to_uppercase(), 10.0).cor(MEDIO).trilha(0.16).na_linha(&p, x, r.min.y + 25.0, 16.0);
    Texto::novo(tr("Nenhum blueprint ainda. Por onde começar?"), 20.0).peso(Peso::Medio).trilha(-0.02).na_linha(&p, x, r.min.y + 49.0, 32.0);
    let topo = r.min.y + 101.0;
    let mut abrir = false;
    for (i, (n, titulo, descricao)) in PASSOS.iter().enumerate() {
        let c = Rect::from_min_size(pos2(x + i as f32 * (m.largura + 16.0), topo), vec2(m.largura, m.altura_cartao));
        let ativo = i == 0 && !ocupado;
        let resposta = ui.interact(c, ui.scope_id().with(("passo", i)), if ativo { Sense::click() } else { Sense::hover() });
        let realce = ativo && (resposta.hovered() || resposta.has_focus());
        p.rect_filled(c, 0, if realce { rgba(0x124482, 0xff) } else { rgba(0x0a3069, 0xdd) });
        planta::borda(&p, c, if realce { TINTA } else { BAIXO });
        let ix = c.min.x + 19.0;
        Texto::novo(*n, 11.0).cor(MEDIO).trilha(0.14).na_linha(&p, ix, c.min.y + 19.0, 17.6);
        planta::linha(&p, pos2(ix, c.min.y + 19.0 + 24.6), pos2(c.max.x - 19.0, c.min.y + 19.0 + 24.6), TENUE);
        Texto::novo(tr(titulo), 17.0).peso(Peso::Medio).na_linha(&p, ix, c.min.y + 52.6, 27.2);
        Texto::novo(tr(descricao), 12.0).cor(MEDIO).lh(19.2).caixa(m.largura - 38.0).em(&p, pos2(ix, c.min.y + 87.8));
        Texto::novo("→", 14.0).cor(MEDIO).na_linha_dir(&p, c.max.x - 19.0, c.max.y - 19.0 - 22.4, 22.4);
        if ativo && resposta.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if ativo && resposta.clicked() {
            abrir = true;
        }
    }
    abrir
}

/// Os cartões dos workspaces e o «Criar workspace», em linhas que quebram como o `flex-wrap` da página.
fn grade(ui: &mut Ui, vista: &mut Vista, ws: &[Resumo], origem: Pos2, colunas: usize, ocupado: bool, acao: &mut Option<Acao>) {
    let slot = |i: usize| Rect::from_min_size(origem + vec2((i % colunas) as f32 * (CARTAO.x + FOLGA), (i / colunas) as f32 * (CARTAO.y + FOLGA)), CARTAO);
    let mut soltar: Option<(String, usize)> = None;
    for (i, w) in ws.iter().enumerate() {
        let r = slot(i);
        let resposta = ui.interact(r, ui.scope_id().with(("ws", &w.id)), Sense::click_and_drag());
        let arrastando = resposta.dragged() && resposta.drag_delta().length() >= 0.0 && ui.input(|i| i.pointer.is_decidedly_dragging());
        let realce = ui.ctx().animate_bool_with_time(resposta.id, resposta.hovered() || resposta.has_focus(), 0.2);
        cartao(ui, r, w, realce, arrastando);
        if resposta.clicked() {
            vista.tela = Tela::Workspace(w.id.clone());
        }
        if !ocupado {
            if resposta.drag_stopped() {
                if let Some(ponto) = ui.input(|i| i.pointer.interact_pos()) {
                    if let Some(alvo) = (0..ws.len()).find(|j| slot(*j).contains(ponto)).filter(|j| *j != i) {
                        soltar = Some((w.id.clone(), alvo));
                    }
                }
            }
            if resposta.has_focus() {
                let (alt, esq, dir) = ui.input(|i| (i.modifiers.alt, i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowDown)));
                if alt && (esq || dir) {
                    *acao = Some(Acao::Mover(w.id.clone(), if esq { -1 } else { 1 }));
                }
            }
        }
        if w.id != RAIZ {
            resposta.context_menu(|ui| {
                if ui.add_enabled(!ocupado, egui::Button::new(tr("Excluir workspace…"))).clicked() {
                    vista.excluir = Some(w.id.clone());
                    ui.close();
                }
            });
        }
        if arrastando {
            if let Some(ponto) = ui.input(|i| i.pointer.hover_pos()) {
                let fantasma = Texto::novo(format!("↔ {}", w.nome), 12.0);
                let caixa = Align2::LEFT_TOP.anchor_size(ponto + vec2(12.0, 12.0), vec2(fantasma.largura(ui.painter()) + 20.0, 28.0));
                let p = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, ui.scope_id().with("fantasma")));
                p.rect_filled(caixa, 0, PAPEL);
                planta::borda(&p, caixa, TINTA);
                fantasma.na_linha(&p, caixa.min.x + 10.0, caixa.min.y, 28.0);
            }
        }
    }
    if let Some((id, alvo)) = soltar {
        *acao = Some(Acao::MoverPara(id, alvo));
    }
    let r = slot(ws.len());
    let resposta = ui.interact(r, ui.scope_id().with("novo"), Sense::click());
    adicionar(ui, r, ui.ctx().animate_bool_with_time(resposta.id, resposta.hovered() || resposta.has_focus(), 0.2));
    if resposta.clicked() && !ocupado {
        vista.novo = Some(super::Novo::vazio());
    }
}

/// `realce` vai de 0 a 1 (a transição de 0,2 s da página): o cartão sobe 3 px, clareia e a borda acende.
fn cartao(ui: &Ui, base: Rect, w: &Resumo, realce: f32, arrastando: bool) {
    let p = ui.painter().clone();
    let r = if arrastando { base } else { base.translate(vec2(0.0, -3.0 * realce)) };
    let opaco = |c: egui::Color32| if arrastando { c.gamma_multiply(0.5) } else { c };
    p.rect_filled(r, 0, opaco(rgba(0x0a3069, 0xdd).lerp_to_gamma(rgba(0x124482, 0xff), realce)));
    planta::borda(&p, r, opaco(BAIXO.lerp_to_gamma(TINTA, realce)));
    canto(&p, r);
    let (x, y, util) = (r.min.x + 25.0, r.min.y + 25.0, CARTAO.x - 50.0);
    Texto::novo(tr("Workspace").to_uppercase(), 10.0).cor(opaco(MEDIO)).trilha(0.14).na_linha(&p, x, y, 16.0);
    emblema::pintar(ui, Rect::from_min_size(pos2(x, y + 28.0), vec2(util, 120.0)), &w.icone, emblema::Tamanho::Cartao);
    Texto::novo(&*w.nome, 19.0).peso(Peso::Medio).cor(opaco(TINTA)).lh(24.7).caixa(util).linhas(2).em(&p, pos2(x, y + 160.0));
    let descricao = if w.descricao.is_empty() { tr("Blueprints e planejamento do workspace").to_string() } else { w.descricao.clone() };
    Texto::novo(descricao, 12.0).cor(opaco(MEDIO)).lh(19.2).caixa(util).linhas(2).em(&p, pos2(x, y + 222.0));
    planta::linha(&p, pos2(x, y + 276.0), pos2(x + util, y + 276.0), opaco(TENUE));
    Texto::novo(trn(w.blueprints, "{n} blueprint", "{n} blueprints"), 11.0).cor(opaco(MEDIO)).na_linha(&p, x, y + 293.0, 17.6);
    Texto::novo(tr("Abrir espaço →"), 11.0).cor(opaco(MEDIO)).na_linha_dir(&p, x + util, y + 293.0, 17.6);
}

fn adicionar(ui: &Ui, r: Rect, realce: f32) {
    let p = ui.painter().clone();
    let r = r.translate(vec2(0.0, -3.0 * realce));
    p.rect_filled(r, 0, rgba(0x0c3a78, 0x66).lerp_to_gamma(rgba(0x124482, 0xff), realce));
    planta::tracejado(&p, r, BAIXO.lerp_to_gamma(TINTA, realce));
    canto(&p, r);
    let cx = r.center().x;
    let caixa = Rect::from_min_size(pos2(cx - 32.0, r.min.y + 98.4), vec2(64.0, 64.0));
    planta::borda(&p, caixa, MEDIO);
    Texto::novo("+", 36.0).centro(&p, caixa.center());
    Texto::novo(tr("Criar workspace"), 19.0).peso(Peso::Medio).centro(&p, pos2(cx, r.min.y + 174.4 + 15.2));
    Texto::novo(tr("Um espaço para agrupar blueprints."), 14.0).lh(22.4).caixa(CARTAO.x - 50.0).centrado().centrado_em(&p, cx, r.min.y + 216.8);
}
