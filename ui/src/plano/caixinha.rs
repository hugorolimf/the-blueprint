//! A caixinha de cada plano (`design/planejar/acao-planos.mjs:15`): o desenho sai da situação que o servidor calculou; o clique só marca para iniciar quem está em `Nao`.
//! Plano em execução leva à linha em Agentes; marcar como feito à mão fica no menu, porque a caixinha chama o agente.

use super::folha::estilo::{com_alfa, pulso, sobre, ANDANDO, FEITO};
use super::linha::Pincel;
use super::modelo::{Plano, Situacao};
use super::Estado;
use crate::fonte::{Peso, SUBIDA};
use crate::i18n::{tr, tr_com};
use crate::planta::{botao, Botao, Texto};
use crate::tema::{AMBAR, MEDIO, PAPEL, TINTA, VERMELHO};
use egui::{vec2, Color32, Id, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui};
use serde_json::{json, Value};

pub const TEXTOS: &[(&str, &str)] = &[
    ("Iniciar os {n}", "Start all {n}"),
    ("Chamar o agente neste plano", "Call the agent on this plan"),
    ("Ver em Agentes", "See in Agents"),
];

const LADO: f32 = 13.0;
/// O período em que o plano em execução respira.
const PISCA_S: f32 = 1.5;
/// O canto da caixinha.
const CANTO: f32 = 3.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Desenho {
    Vazia,
    Check,
    Tracejada,
    Piscando,
    Ponto,
    Xis,
}

pub fn desenho_de(s: Situacao) -> Desenho {
    match s {
        Situacao::Pronto => Desenho::Check,
        Situacao::Revisao => Desenho::Tracejada,
        Situacao::Trabalhando => Desenho::Piscando,
        Situacao::Fila | Situacao::Dependencia | Situacao::Area => Desenho::Ponto,
        Situacao::Bloqueado | Situacao::Parado => Desenho::Xis,
        Situacao::Nao | Situacao::Outra => Desenho::Vazia,
    }
}

/// O que um clique na caixinha pede.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clique {
    /// `gesto/marcar` deste plano
    Marcar,
    /// abrir a linha do plano em Agentes
    Agentes,
}

pub fn clique_de(plano: &Plano) -> Option<Clique> {
    if plano.situacao == Situacao::Nao && !plano.feito {
        Some(Clique::Marcar)
    } else if !plano.execucao_id.is_empty() {
        Some(Clique::Agentes)
    } else {
        None
    }
}

/// Os planos que «Iniciar os N» marca, pelo id.
pub fn livres(planos: &[Plano]) -> Vec<&str> {
    planos.iter().filter(|p| clique_de(p) == Some(Clique::Marcar)).map(|p| p.id.as_str()).collect()
}

pub fn corpo_marcar(ws: &str, bp: &str, planos: &[&str]) -> Value {
    json!({"workspaceId": ws, "blueprintId": bp, "planos": planos})
}

/// Pede ao servidor para iniciar estes planos.
pub fn marcar(ctx: &egui::Context, estado: &Estado, servidor: &str, ws: &str, bp: &str, planos: &[&str]) {
    estado.gesto(ctx, servidor, "marcar", &corpo_marcar(ws, bp, planos));
}

/// O lápis das peças: traço e texto em coordenadas da tela do Planejar, a partir da origem `o`. Quem desenha com `Cena` passa `Lapis { p: c.p, o: c.o }`.
#[derive(Clone, Copy)]
pub struct Lapis<'a> {
    pub p: &'a Painter,
    pub o: Pos2,
}

impl Lapis<'_> {
    pub fn pt(&self, x: f32, y: f32) -> Pos2 {
        self.o + vec2(x, y)
    }

    pub fn retangulo(&self, x: f32, y: f32, w: f32, h: f32, fundo: Option<Color32>, traco: Option<(Color32, f32)>) {
        let r = Rect::from_min_size(self.pt(x, y), vec2(w, h));
        if let Some(f) = fundo {
            self.p.rect_filled(r, 0, f);
        }
        if let Some((cor, largura)) = traco {
            self.p.rect_stroke(r, 0, Stroke::new(largura, cor), egui::StrokeKind::Middle);
        }
    }

    pub fn fio(&self, cor: Color32, largura: f32, de: (f32, f32), ate: (f32, f32)) {
        self.p.line_segment([self.pt(de.0, de.1), self.pt(ate.0, ate.1)], Stroke::new(largura, cor));
    }

    /// Um retângulo só de contorno, tracejado com `traco` cheio e `traco` vazio.
    pub fn tracejado(&self, cor: Color32, largura: f32, x: f32, y: f32, w: f32, h: f32, traco: f32) {
        let pontos = [self.pt(x, y), self.pt(x + w, y), self.pt(x + w, y + h), self.pt(x, y + h), self.pt(x, y)];
        self.p.extend(egui::Shape::dashed_line(&pontos, Stroke::new(largura, cor), traco, traco));
    }

    /// Escreve `s` com a linha de base em `y`, sem o contorno do papel (é sobre fundo cheio).
    pub fn texto(&self, s: &str, x: f32, y: f32, tam: f32, peso: Peso, cor: Color32) {
        Texto::novo(s.to_string(), tam).trilha(0.06).peso(peso).cor(cor).em(self.p, self.pt(x, y - SUBIDA * tam));
    }
}

/// O botão pequeno (`mini` do protótipo): 24 de altura, 9 dos lados, letra de 10.
pub fn botao_mini(texto: &str) -> Botao<'_> {
    Botao { texto, tam: 10.0, cheio: false, ativo: true, altura: 24.0, pad_x: 9.0 }
}

/// Desenha a caixinha com o canto de cima à esquerda em `(x, y)`.
pub fn desenhar(l: Lapis, x: f32, y: f32, s: Situacao) {
    desenhar_sobre(l, x, y, s, 0.0);
}

/// A caixinha com o mouse em cima (`luz` de 0 a 1): feita é verde-água com o visto, em execução respira em azul, esperando revisão é âmbar tracejada.
fn desenhar_sobre(l: Lapis, x: f32, y: f32, s: Situacao, luz: f32) {
    let (p, caixa) = (Pincel { p: l.p, o: l.o }, Rect::from_min_size(l.pt(x, y), vec2(LADO, LADO)));
    let contorno = |cor: Color32| l.p.rect_stroke(caixa, CANTO, Stroke::new(1.2, cor), StrokeKind::Inside);
    match desenho_de(s) {
        Desenho::Check => {
            l.p.rect_filled(caixa, CANTO, com_alfa(FEITO, 0.9));
            p.visto(x + 6.5, y + 6.5, 8.0, PAPEL, 1.7);
        }
        Desenho::Tracejada => {
            p.suave_tracejado(x, y, LADO, LADO, CANTO, TINTA, 2.5, 2.0);
        }
        Desenho::Piscando => {
            let respira = pulso(l.p.ctx(), PISCA_S);
            l.p.rect_filled(caixa, CANTO, com_alfa(ANDANDO, 0.12 + 0.18 * respira));
            contorno(ANDANDO);
            l.p.rect_filled(caixa.shrink(3.5), 1.5, com_alfa(ANDANDO, 0.35 + 0.65 * respira));
        }
        Desenho::Ponto => {
            contorno(com_alfa(MEDIO, 0.9));
            l.p.circle_filled(l.pt(x + 6.5, y + 6.5), 2.0, AMBAR);
        }
        Desenho::Xis => {
            contorno(com_alfa(VERMELHO, 0.8));
            l.fio(VERMELHO, 1.3, (x + 3.5, y + 3.5), (x + 9.5, y + 9.5));
            l.fio(VERMELHO, 1.3, (x + 9.5, y + 3.5), (x + 3.5, y + 9.5));
        }
        Desenho::Vazia => {
            l.p.rect_filled(caixa, CANTO, com_alfa(TINTA, 0.1 * luz));
            contorno(com_alfa(TINTA, 0.65 + 0.35 * luz));
        }
    }
}

/// A caixinha com o clique: quem chama faz o que voltar (`marcar`, ou `Sub::Agentes`), porque a folha que ele percorre está emprestada do mesmo `Estado`.
pub fn caixinha(l: Lapis, ui: &mut Ui, x: f32, y: f32, plano: &Plano) -> Option<Clique> {
    let Some(pede) = clique_de(plano) else {
        desenhar(l, x, y, plano.situacao);
        return None;
    };
    let r = Rect::from_min_size(l.pt(x, y), vec2(LADO, LADO)).expand(3.0);
    let resposta = ui.interact(r, ui.scope_id().with(("plano-caixinha", &plano.id)), Sense::click());
    desenhar_sobre(l, x, y, plano.situacao, sobre(ui.ctx(), Id::unique(("plano-caixinha-sobre", &plano.id)), resposta.hovered()));
    let resposta = resposta.on_hover_text(match pede {
        Clique::Marcar => tr("Chamar o agente neste plano"),
        Clique::Agentes => tr("Ver em Agentes"),
    });
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resposta.clicked().then_some(pede)
}

/// A altura de «Iniciar os N», para quem mede antes de empilhar.
pub const ALTURA_INICIAR: f32 = 24.0;

/// «Iniciar os N», só com mais de um livre na ação: marca todos de uma vez. Devolve se apareceu.
pub fn iniciar_os_n(ui: &mut Ui, pos: Pos2, estado: &Estado, servidor: &str, ws: &str, bp: &str, planos: &[Plano]) -> bool {
    let ids = livres(planos);
    if ids.len() < 2 {
        return false;
    }
    let texto = tr_com("Iniciar os {n}", &[("n", &ids.len().to_string())]);
    let ctx = ui.ctx().clone();
    if botao(ui, pos, &botao_mini(&texto)).clicked() {
        marcar(&ctx, estado, servidor, ws, bp, &ids);
    }
    true
}
