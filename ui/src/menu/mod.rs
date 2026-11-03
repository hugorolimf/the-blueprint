//! O menu do botão direito (`contexto.js` da página). Cada tela marca o que está sob o cursor (`marcar`), e o que aparece sai de todos os alvos
//! marcados naquele ponto: um plano está dentro de uma ação, que está dentro da folha. Os itens saem de `itens.rs` e o desenho de `desenho.rs`.

mod desenho;
mod itens;

pub use itens::Alvo;

use crate::telas::{Acao, Novo, NovoBp, Tela, Vista};
use egui::{Context, Id, Key, Modifiers, Order, Pos2, Rect, Ui, Vec2};

/// Os grupos, na ordem em que aparecem; uma linha separa um do outro.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Grupo {
    Abrir,
    Editar,
    Estado,
    Perigo,
    Geral,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Icone {
    Abrir,
    Alvo,
    Doc,
    Externo,
    Check,
    Circulo,
    Lixo,
    Voltar,
    TelaCheia,
    Relogio,
    Mais,
    Recolher,
    Expandir,
}

/// O que um item faz: muda os dados (`Dados`, quem aplica é o `app`) ou só mexe na tela.
#[derive(Debug)]
pub enum Comando {
    Dados(Acao),
    AbrirWorkspace(String),
    Entrada,
    CriarWorkspace,
    ExcluirWorkspace(String),
    AbrirBlueprint(String, String),
    NovoBlueprint(String),
    /// a pergunta antes de apagar: (workspace, blueprint, ação; sem ação, o blueprint inteiro)
    ExcluirItem(String, String, Option<String>),
    IrPara(String),
    Agente(String),
    Link(String),
    /// abre na versão web o que a interface nativa ainda não tem
    Web(String),
    /// troca o blueprint entre a folha antiga (`true`) e o Planejar (`false`)
    FolhaAntiga(String, bool),
    /// abre ou recolhe estas seções
    Secoes(Vec<String>, bool),
    FecharNovo,
    TelaCheia,
}

#[derive(Debug)]
pub struct Item {
    pub(crate) grupo: Grupo,
    pub(crate) icone: Icone,
    pub(crate) rotulo: String,
    pub(crate) comando: Option<Comando>,
    pub(crate) perigo: bool,
    /// por que o item está desligado (sem comando)
    pub(crate) motivo: Option<&'static str>,
}

pub struct Menu {
    pos: Pos2,
    titulo: String,
    itens: Vec<Item>,
    foco: Option<usize>,
    caixa: Rect,
    /// o tamanho da janela ao abrir: mudou, o menu fecha (como na página)
    tela: Vec2,
}

fn chave() -> Id {
    Id::unique("menu-alvos")
}

/// A tela diz que `area` é este alvo. Só monta o alvo no quadro do clique direito, e só se o cursor estiver ali.
pub fn marcar(ui: &Ui, area: Rect, alvo: impl FnOnce() -> Alvo) {
    let ctx = ui.ctx();
    let (direito, pos) = ctx.input(|i| (i.pointer.secondary_clicked(), i.pointer.interact_pos()));
    if direito && pos.is_some_and(|p| area.contains(p)) {
        let alvo = alvo();
        ctx.data_mut(|d| d.get_temp_mut_or_default::<Vec<Alvo>>(chave()).push(alvo));
    }
}

/// Com o menu aberto, as teclas são dele (antes das telas): setas, Home, End, Enter, Esc e Tab, como `contexto.js`.
pub fn teclado(ctx: &Context, vista: &mut Vista) -> Option<Comando> {
    let m = vista.menu.as_mut()?;
    let tecla = |k: Key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k));
    if tecla(Key::Escape) || tecla(Key::Tab) {
        vista.menu = None;
        return None;
    }
    let n = m.itens.len();
    if tecla(Key::ArrowDown) {
        m.foco = Some(m.foco.map_or(0, |f| (f + 1) % n));
    }
    if tecla(Key::ArrowUp) {
        m.foco = Some(m.foco.map_or(n - 1, |f| (f + n - 1) % n));
    }
    if tecla(Key::Home) {
        m.foco = Some(0);
    }
    if tecla(Key::End) {
        m.foco = Some(n - 1);
    }
    if tecla(Key::Enter) {
        let comando = m.foco.and_then(|f| m.itens.get_mut(f)).and_then(|i| i.comando.take());
        if comando.is_some() {
            vista.menu = None;
        }
        return comando;
    }
    None
}

/// Depois das telas: abre ou fecha o menu e o desenha. Devolve o que a pessoa escolheu.
pub fn passo(ctx: &Context, vista: &mut Vista, ocupado: bool) -> Option<Comando> {
    let alvos = ctx.data_mut(|d| {
        let alvos = d.get_temp::<Vec<Alvo>>(chave());
        d.remove_temp::<Vec<Alvo>>(chave());
        alvos
    });
    let (direito, pos, apertou, rolou) = ctx.input(|i| (i.pointer.secondary_clicked(), i.pointer.interact_pos(), i.pointer.any_pressed(), i.smooth_scroll_delta() != Vec2::ZERO));
    let tela = ctx.content_rect().size();
    if let Some(m) = &vista.menu {
        let fora = pos.is_none_or(|p| !m.caixa.contains(p));
        if ((apertou || rolou) && fora) || m.tela != tela {
            vista.menu = None;
        }
    }
    if let (true, Some(p)) = (direito, pos) {
        if livre(ctx, vista, p) {
            let agentes = crate::agentes::ativos(&vista.agentes, &vista.agente_ws(), crate::agentes::agora_ms(), vista.agentes_vida, &vista.vistos);
            let cheia = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            let criando = vista.novo.is_some() || vista.novo_bp.is_some();
            let alvos = alvos.unwrap_or_default();
            let (titulo, mut itens) = itens::montar(&alvos, &agentes, cheia, criando);
            // sem a rota do Planejar a folha antiga é a única que há: não tem o que trocar
            if let (Tela::Blueprint(_, bp), false, false) = (&vista.tela, criando, vista.plano.folha_falhou) {
                if alvos.iter().any(|a| matches!(a, Alvo::Folha { .. })) {
                    itens::trocar_folha(&mut itens, bp, vista.folha_antiga.contains(bp));
                }
            }
            vista.menu = Some(Menu { pos: p, titulo, itens, foco: None, caixa: Rect::NOTHING, tela });
        }
    }
    let m = vista.menu.as_mut()?;
    let comando = desenho::mostrar(ctx, m, ocupado);
    if comando.is_some() {
        vista.menu = None;
    }
    comando
}

/// O clique caiu na tela, e não em cima de uma folha ou de uma pergunta aberta.
fn livre(ctx: &Context, vista: &Vista, p: Pos2) -> bool {
    vista.nota.is_none() && vista.excluir.is_none() && vista.excluir_item.is_none() && ctx.layer_id_at(p).is_none_or(|l| l.order == Order::Background)
}

/// Faz o que o item pede; o que muda os dados volta como `Acao`.
pub fn executar(comando: Comando, ctx: &Context, vista: &mut Vista, servidor: &str) -> Option<Acao> {
    match comando {
        Comando::Dados(acao) => return Some(acao),
        Comando::AbrirWorkspace(id) => vista.tela = Tela::Workspace(id),
        Comando::Entrada => vista.tela = Tela::Entrada,
        Comando::CriarWorkspace => vista.novo = Some(Novo::vazio()),
        Comando::ExcluirWorkspace(id) => vista.excluir = Some(id),
        Comando::AbrirBlueprint(ws, bp) => vista.tela = Tela::Blueprint(ws, bp),
        Comando::NovoBlueprint(ws) => vista.novo_bp = Some(NovoBp::em(&ws)),
        Comando::ExcluirItem(ws, bp, acao) => vista.excluir_item = Some((ws, bp, acao)),
        Comando::IrPara(acao) => vista.ir_para = Some(acao),
        Comando::Agente(nome) => vista.agente_aberto = Some(nome),
        Comando::Link(url) => ctx.open_url(egui::OpenUrl::new_tab(url)),
        Comando::Web(destino) => ctx.open_url(egui::OpenUrl::new_tab(format!("{servidor}/{destino}"))),
        Comando::FolhaAntiga(bp, true) => {
            vista.folha_antiga.insert(bp);
        }
        Comando::FolhaAntiga(bp, false) => {
            vista.folha_antiga.remove(&bp);
        }
        Comando::Secoes(chaves, abrir) => {
            for chave in chaves {
                vista.abertas.insert(chave, abrir);
            }
            crate::anima::mexer(ctx, 350.0);
        }
        Comando::FecharNovo => {
            vista.novo = None;
            vista.novo_bp = None;
        }
        Comando::TelaCheia => {
            let cheia = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!cheia));
        }
    }
    None
}
