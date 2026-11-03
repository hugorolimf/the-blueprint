//! A linha de uma ação na folha, com o bloco dela na escada (`linha_bloco.rs`), e o cabeçalho do projeto, o mesmo da folha original (`folha/secao.rs`).
//! A barra fina de «O que chegou» e as formas ficam em `barra_*.rs`.

use super::escala::{Escala, LINHA_FOLHA};
use super::folha::estilo::{com_alfa, sobre};
use super::modelo::{Acao, Folha, Projeto};
use crate::dominio::Situacao;
use crate::folha::cena::{Ancora, Cena};
use crate::folha::pagina::corta;
use crate::fonte::Peso;
use crate::menu::{self, Alvo};
use crate::tema::{BAIXO, MEDIO, TINTA};
use egui::{Id, Sense, Stroke, Ui};

#[path = "linha_bloco.rs"]
pub mod bloco;
#[path = "linha_esteiras.rs"]
pub mod esteiras;
#[path = "barra_cronograma.rs"]
mod cronograma;
#[path = "barra_formas.rs"]
mod formas;
#[path = "folha_figura.rs"]
pub mod figura;
#[path = "linha_pincel.rs"]
mod pincel;
pub use cronograma::barra;
pub use pincel::{Letra, Pincel};

pub const TEXTOS: &[(&str, &str)] = &[
    ("Aguardando plano", "Awaiting plan"),
    ("×{n}", "×{n}"),
];

/// A altura do cabeçalho de um projeto: os 58 px até o fio e o respiro até a primeira ação.
pub const ALTO_PROJETO: f32 = 76.0;

/// O cabeçalho de um projeto: o círculo com o número, o nome em caixa alta e o fio. `y` é o meio do cabeçalho.
pub fn cabecalho(p: &Pincel, regua: &Pincel, e: &Escala, projeto: &Projeto, nome_padrao: &str, numero: usize, y: f32) {
    let c = Cena { p: p.p, o: p.o };
    let (l, topo) = (e.x0, y - ALTO_PROJETO / 2.0);
    c.p.circle_stroke(c.pt(l + 14.0, topo + 20.0), 14.0, Stroke::new(1.0, TINTA));
    c.texto(&format!("{numero:02}"), l + 14.0, topo + 25.0, 13.0, Peso::Negrito, TINTA, Ancora::Meio, false);
    let nome = if projeto.nome.is_empty() { nome_padrao } else { &projeto.nome };
    c.texto(&corta(nome, 48), l + 40.0, topo + 19.0, 17.0, Peso::Negrito, TINTA, Ancora::Inicio, true);
    regua.fio(l, topo + 58.0, e.x1, topo + 58.0, MEDIO, 1.0);
}

/// A situação da ação para o menu do botão direito: a guardada, ou a que os planos dizem.
pub fn situacao(a: &Acao) -> Situacao {
    let n = &a.numeros;
    match a.status.as_str() {
        "concluido" => Situacao::Concluido,
        "andamento" => Situacao::Andamento,
        _ if n.planos > 0 && n.prontos >= n.planos => Situacao::Concluido,
        _ if n.prontos > 0 => Situacao::Andamento,
        _ => Situacao::Planejado,
    }
}

/// O alvo do menu de uma ação.
pub fn alvo(ws: &str, bp: &str, a: &Acao) -> Alvo {
    let feitos = a.numeros.prontos as usize + a.checkpoints.iter().filter(|k| k.feito).count();
    let total = a.numeros.planos as usize + a.checkpoints.len();
    Alvo::Acao { ws: ws.into(), bp: bp.into(), id: a.id.clone(), rotulo: a.referencia.clone(), status: situacao(a), itens: (feitos, total) }
}

/// Uma ação, centrada em `y`: toda a linha responde ao mouse. Devolve se pediu para abrir ou fechar.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, f: &Folha, e: &Escala, a: &Acao, servidor: &str, ws: &str, bp: &str, y: f32, aberta: f32) -> bool {
    let (topo, largura) = (y - LINHA_FOLHA / 2.0, e.x1 - e.x0);
    let area = c.caixa(e.x0, topo, largura, LINHA_FOLHA);
    let resposta = ui.interact(area, ui.scope_id().with(("plano-linha", a.id.as_str())), Sense::click());
    menu::marcar(ui, area, || alvo(ws, bp, a));
    let luz = sobre(ui.ctx(), Id::unique(("plano-linha-sobre", a.id.as_str())), resposta.hovered());
    let na_peca = bloco::desenhar(ui, c, f, e, a, servidor, y, luz);
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let clicou = resposta.clicked();
    // o `on_hover_text` do egui 0.37, com a dica escrita só quando aparece: montar o texto em toda linha a cada quadro custava, e a acessibilidade continua com ele quando há nó
    if ui.ctx().has_accesskit_node(resposta.id) {
        let _ = resposta.on_hover_text(bloco::dica(a));
    } else {
        let _ = resposta.on_hover_ui(|ui| {
            ui.set_max_width(ui.spacing().tooltip_width);
            ui.add(egui::Label::new(bloco::dica(a)));
        });
    }
    // o clique numa peça abre a ação, mas nunca fecha a que já está aberta
    clicou || (na_peca && aberta <= 0.5)
}

/// A divisória entre duas ações de «O que chegou»: um fio de cabelo.
pub fn separador(c: &Pincel, e: &Escala, y: f32) {
    c.fio(e.x0, y, e.x1, y, com_alfa(BAIXO, 0.22), 1.0);
}
