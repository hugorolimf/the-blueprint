//! Os sinais da planta, desenhados em traço isométrico como o resto da obra: cadeado, iniciar, visto, parado, fila e construção. A cor diz o estado, e a dica escreve o que a cor diz.
//! Aqui também mora a casa: a peça, com a contagem do lado e o clique que leva até ela.

use super::super::acao_estilo::{alfa, mix};
use super::super::linha::{Letra, Pincel};
use super::super::vazio_iso::{Iso, Tons};
use super::figura::{self, Marca, Peca};
use super::obra_pecas::cor_da_marca;
use super::obra_regra::Casa;
use crate::i18n::{tr, tr_com};
use crate::tema::{CAMPO, TINTA};
use egui::{pos2, vec2, Color32, CursorIcon, Painter, Pos2, Sense, Shape, Stroke, Ui};

pub const LADO: f32 = 16.0;
const K_PECA: f32 = 15.0;
const VAO_NUMERO: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Icone {
    Cadeado,
    Iniciar,
    Visto,
    Parou,
    Fila,
    Obra,
    /// a placa vazia de uma escolha; com o visto, é a escolhida
    Opcao,
    Peca(Peca, Marca),
}

fn tons(cor: Color32) -> Tons {
    Tons { topo: mix(CAMPO, cor, 0.75), esq: mix(CAMPO, cor, 0.36), dir: mix(CAMPO, cor, 0.2), traco: Stroke::new(1.0, cor) }
}

/// De (u, v) na placa para o mundo: u anda para a direita na tela, v para a frente.
fn na_placa(u: f32, v: f32, z: f32) -> (f32, f32, f32) {
    (u + v, v - u, z)
}

fn risco(iso: &Iso, pontos: &[(f32, f32)], z: f32, cor: Color32, largura: f32) {
    let linha: Vec<Pos2> = pontos.iter().map(|(u, v)| iso.ponto(na_placa(*u, *v, z))).collect();
    iso.p.add(Shape::line(linha, Stroke::new(largura, cor)));
}

fn placa(iso: &Iso, cor: Color32) {
    let q = 0.46;
    let cantos = [(-q, -q), (q, -q), (q, q), (-q, q)].map(|(x, y)| (x, y, 0.0));
    iso.face(&cantos, alfa(cor, 0.18), Stroke::new(1.0, alfa(cor, 0.85)));
}

/// O mesmo cadeado da esteira, de frente para quem olha: o corpo de cantos redondos com a faixa do topo atrás dele, a alça em U entrando no meio dessa faixa e o buraco da chave.
/// De frente, na placa, `u` anda só no x da tela e a profundidade só desce: o topo é o mesmo contorno subido, e o corpo não vira um cubo com alça. A alça alta é o que diz «cadeado» na letra pequena.
fn cadeado(iso: &Iso, cor: Color32) {
    let k = iso.k;
    let meio = iso.o - vec2(0.0, k * 0.5);
    let (meia, alto, faixa, canto) = (0.6 * k, 0.74 * k, 0.24 * k, 0.2 * k);
    let (raio, perna) = (0.32 * k, 0.2 * k);
    let pe = meio.y + (raio + perna + faixa * 0.5 + alto) * 0.5;
    let quina = pe - alto;
    let (x0, x1, y0, y1) = (meio.x - meia + canto, meio.x + meia - canto, quina + canto, pe - canto);
    let arco = |cx: f32, cy: f32, de: f32| -> [Pos2; 6] {
        std::array::from_fn(|i| {
            let a = (de + 18.0 * i as f32).to_radians();
            pos2(cx + canto * a.cos(), cy + canto * a.sin())
        })
    };
    // no sentido do relógio na tela; os cantos de baixo ficam na frente e os de cima sobem `sobe`: com `faixa`, é o contorno do corpo com o topo
    let contorno = |sobe: f32| -> Vec<Pos2> { [arco(x1, y1, 0.0), arco(x0, y1, 90.0), arco(x0, y0 - sobe, 180.0), arco(x1, y0 - sobe, 270.0)].concat() };
    let traco = Stroke::new(1.0, cor);
    iso.p.add(Shape::convex_polygon(contorno(faixa), mix(CAMPO, cor, 0.85), traco));
    iso.p.add(Shape::convex_polygon(contorno(0.0), mix(CAMPO, cor, 0.55), traco));
    let chave = pos2(meio.x, quina + alto * 0.42);
    iso.p.circle_filled(chave, 0.14 * k, CAMPO);
    let fenda = [chave + vec2(-0.06 * k, 0.0), chave + vec2(0.06 * k, 0.0), chave + vec2(0.1 * k, 0.28 * k), chave + vec2(-0.1 * k, 0.28 * k)];
    iso.p.add(Shape::convex_polygon(fenda.to_vec(), CAMPO, Stroke::NONE));
    // a alça por cima do corpo: as pernas terminam no meio da faixa do topo, onde entram nele
    let (entra, volta) = (quina - faixa * 0.5, quina - faixa * 0.5 - perna);
    let mut alca = vec![pos2(meio.x - raio, entra)];
    alca.extend((0..=12).map(|i| {
        let a = std::f32::consts::PI * (1.0 + i as f32 / 12.0);
        pos2(meio.x + raio * a.cos(), volta + raio * a.sin())
    }));
    alca.push(pos2(meio.x + raio, entra));
    iso.p.add(Shape::line(alca.clone(), Stroke::new(2.6, cor)));
    iso.p.add(Shape::line(alca, Stroke::new(1.0, mix(cor, TINTA, 0.7))));
}

/// O triângulo de frente para quem olha, com a espessura para trás, como uma placa: é o «iniciar», e não se confunde com a bandeira da obra. Só a frente leva traço: em quina aguda, o traço fechado vira um espinho comprido.
fn iniciar(p: &Painter, c: Pos2, lado: f32, cor: Color32) {
    let t = tons(cor);
    let r = lado * 0.42;
    let frente = [c + vec2(-r * 0.8, -r), c + vec2(r, 0.0), c + vec2(-r * 0.8, r)];
    let atras = frente.map(|q| q + vec2(-lado * 0.13, -lado * 0.09));
    p.add(Shape::convex_polygon(atras.to_vec(), t.dir, Stroke::NONE));
    p.add(Shape::convex_polygon(frente.to_vec(), t.topo, t.traco));
}

/// Desenha o sinal com o meio em `centro`, cabendo em `lado`.
pub fn desenhar(p: &Painter, centro: Pos2, lado: f32, i: Icone, cor: Color32) {
    let k = lado * 0.62;
    let iso = Iso { p, o: centro, k };
    match i {
        Icone::Cadeado => cadeado(&Iso { p, o: centro + vec2(0.0, k * 0.5), k }, cor),
        Icone::Iniciar => iniciar(p, centro + vec2(lado * 0.04, lado * 0.03), lado, cor),
        Icone::Visto => {
            placa(&iso, cor);
            risco(&iso, &[(-0.32, 0.02), (-0.08, 0.26), (0.36, -0.26)], 0.0, cor, 1.7);
        }
        Icone::Parou => {
            placa(&iso, cor);
            risco(&iso, &[(-0.26, -0.26), (0.26, 0.26)], 0.0, cor, 1.6);
            risco(&iso, &[(0.26, -0.26), (-0.26, 0.26)], 0.0, cor, 1.6);
        }
        Icone::Fila => {
            for u in [-0.6, -0.06, 0.48] {
                let (x, y, _) = na_placa(u, 0.0, 0.0);
                Iso { p, o: centro + vec2(0.0, k * 0.15), k }.caixa((x - 0.14, y - 0.14, 0.0), (0.28, 0.28, 0.28), tons(cor));
            }
        }
        Icone::Obra => Iso { p, o: centro + vec2(0.0, k * 0.34), k }.caixa((-0.42, -0.42, 0.0), (0.84, 0.84, 0.66), tons(cor)),
        Icone::Opcao => placa(&iso, cor),
        Icone::Peca(tipo, marca) => peca(p, centro, tipo, marca),
    }
}

/// A peça do plano com o meio em `centro`, do mesmo desenho que ela tem por dentro da obra.
fn peca(p: &Painter, centro: Pos2, tipo: Peca, marca: Marca) {
    let meio = match tipo {
        Peca::Tela => (0.29, 0.04, 0.41),
        Peca::Decisao => (0.23, 0.23, 0.17),
        Peca::Pergunta => (0.0, 0.0, 0.4),
        Peca::Checkpoint => (0.23, 0.23, 0.02),
    };
    let desvio = Iso { p, o: pos2(0.0, 0.0), k: K_PECA }.ponto(meio).to_vec2();
    figura::peca(&Iso { p, o: centro - desvio, k: K_PECA }, tipo, 0.0, 0.0, marca);
}

/// O número que a casa mostra: o que falta; sem falta, quantas há.
fn numero(casa: &Casa) -> String {
    (if casa.pede > 0 { casa.pede } else { casa.total }).to_string()
}

fn letra(casa: &Casa) -> Letra {
    Letra::nova(11.0, cor_da_marca(casa.marca)).forte()
}

pub fn largura_casa(c: &Pincel, casa: &Casa) -> f32 {
    LADO + VAO_NUMERO + c.largura(&numero(casa), &letra(casa))
}

fn dica(casa: &Casa) -> String {
    let n = [("n", casa.pede.to_string())];
    let n = [(n[0].0, n[0].1.as_str())];
    let total = casa.total.to_string();
    match (casa.tipo, casa.pede) {
        (Peca::Decisao, 0) => tr("Decisões confirmadas").to_string(),
        (Peca::Decisao, _) => tr_com("{n} decisões a confirmar", &n),
        (Peca::Pergunta, 0) => tr("Perguntas respondidas").to_string(),
        (Peca::Pergunta, _) => tr_com("{n} perguntas a responder", &n),
        (Peca::Tela, 0) => tr_com("Telas: {n}", &[("n", &total)]),
        (Peca::Tela, _) => tr_com("{n} telas a aprovar", &n),
        (Peca::Checkpoint, 0) => tr("Checkpoints feitos").to_string(),
        (Peca::Checkpoint, _) => tr_com("{n} checkpoints a conferir", &n),
    }
}

/// A peça e a contagem, com `y` no meio da linha. Devolve se clicaram.
pub fn casa(ui: &mut Ui, c: &Pincel, chave: (&str, &str, usize), casa: &Casa, x: f32, y: f32) -> bool {
    let w = largura_casa(c, casa);
    let area = c.caixa(x - 2.0, y - 11.0, w + 4.0, 22.0);
    let r = ui.interact(area, ui.scope_id().with(("plano-obra-casa", chave)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    if r.hovered() {
        c.p.rect_filled(area, 0, alfa(TINTA, 0.14));
    }
    pintar_casa(c, casa, x, y);
    let clicou = r.clicked();
    let _ = r.on_hover_text(dica(casa));
    clicou
}

/// A peça e a contagem, sem clique: o que vai sumindo embaixo da coluna só se nota.
pub fn pintar_casa(c: &Pincel, casa: &Casa, x: f32, y: f32) {
    desenhar(c.p, c.pt(x + LADO * 0.5, y), LADO, Icone::Peca(casa.tipo, casa.marca), TINTA);
    c.texto(&numero(casa), x + LADO + VAO_NUMERO, y + 4.0, letra(casa));
}
