//! As peças de um plano no desenho: o que se decide, se responde, se vê ou se confere. Cada uma tem um número que segue o do plano: no plano 1.1, são 1.1.1, 1.1.2 e assim por diante.
//! A mesma peça aparece por dentro da construção (o mergulho) e na vista expandida da parte: as duas pintam daqui.

use super::blocos::Bloco;
use super::figura::{Marca, Peca};
use super::obra_figura::Modulo;
use super::obra_letreiro::Letreiro;
use super::obra_tela::{self as tela, Composicao, Lugar};
use crate::anatomia::{imagem_na_face, Camada, Camera, Cena, Malha, Tinta, V3};
use crate::plano::modelo::Detalhe;
use crate::tema::{AZUL, MEDIO, TINTA, VERDE, VERMELHO};
use egui::{Color32, Context, Painter};

/// Quantas peças de cada tipo cabem no desenho; a lista do lado traz todas.
const CABEM: usize = 5;

/// O que só o palco sabe pintar, na ordem da cena: a figura de uma tela.
pub enum Extra<'a> {
    Imagem(&'a Detalhe, [V3; 4], f32),
}

/// Onde os letreiros começam no número que a cena devolve: abaixo dele, o número é o de uma figura.
const PRIMEIRO_LETREIRO: usize = 1 << 24;

/// O que o palco pinta na ordem da cena: as figuras das telas e os letreiros dos monitores sem tela. O letreiro fica fora de `Extra` porque a miniatura, o mergulho na tela e a vista expandida só pintam figuras.
#[derive(Default)]
pub struct Extras<'a> {
    pub imagens: Vec<Extra<'a>>,
    pub letreiros: Vec<Letreiro<'a>>,
}

impl<'a> Extras<'a> {
    /// Põe o letreiro na cena, na vez do vidro dele. Não responde ao clique: a construção já responde.
    pub fn letreiro(&mut self, cena: &mut Cena, l: Letreiro<'a>) {
        cena.extra(Camada::Pecas, l.centro(), PRIMEIRO_LETREIRO + self.letreiros.len(), 0, 0.0);
        self.letreiros.push(l);
    }

    /// Pinta o extra `n`, quando a cena chega nele.
    pub fn pintar(&self, n: usize, ctx: &Context, servidor: &str, cam: &Camera, p: &Painter) {
        match n.checked_sub(PRIMEIRO_LETREIRO) {
            Some(k) => {
                if let Some(l) = self.letreiros.get(k) {
                    l.pintar(cam, p);
                }
            }
            None => {
                let Extra::Imagem(d, cantos, alfa) = &self.imagens[n];
                pintar_imagem(ctx, servidor, d, cam, p, cantos, *alfa);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PecaObra<'a> {
    pub tipo: Peca,
    pub chave: String,
    pub nome: String,
    pub marca: Marca,
    pub tela: Option<&'a Detalhe>,
    /// o lugar da tela, em pé; as outras peças só têm o canto
    pub lugar: Option<Lugar>,
    pub numero: String,
    /// se cabe no desenho
    pub visivel: bool,
    /// o canto, relativo à laje
    pub u: f32,
    pub v: f32,
}

impl PecaObra<'_> {
    pub fn centro(&self, m: &Modulo) -> V3 {
        if let Some(l) = self.lugar {
            return [m.x + l.u + l.w * 0.5, m.y + l.v, m.base + 0.12 + l.chao + l.pe + l.h * 0.5];
        }
        [m.x + self.u + 0.3, m.y + self.v + 0.3, m.base + 0.4]
    }
}

/// As peças de um bloco, na ordem em que são numeradas: decisões, perguntas, telas e checkpoints. Por dentro, cada tipo tem a sua fileira, com os checkpoints na frente; a tela não entra aí: é parte da construção (`obra_tela`).
pub fn pecas_de<'a>(b: &Bloco<'a>) -> Vec<PecaObra<'a>> {
    let dono = b.plano.as_ref().map_or("", |q| q.referencia.as_str());
    let comp = b.plano.as_ref().map_or(Composicao::Parede, |q| tela::composicao(q, !b.telas.is_empty()));
    let lugares = tela::fileira(&b.telas, comp);
    let mut v: Vec<PecaObra<'a>> = Vec::new();
    let mut por = |tipo, chave, nome: &str, marca, detalhe, lugar: Option<Lugar>, i: usize, cabem: usize, u: f32, fila: f32| {
        let numero = format!("{dono}.{}", v.len() + 1);
        v.push(PecaObra { tipo, chave, nome: nome.to_string(), marca, tela: detalhe, lugar, numero, visivel: i < cabem, u, v: fila });
    };
    for (i, d) in b.decisoes.iter().enumerate() {
        let nome = if d.rotulo.is_empty() { &d.texto } else { &d.rotulo };
        por(Peca::Decisao, format!("d:{}", d.id), nome, Marca::de_revisao(d.pede_aprovacao, d.revisao.as_ref()), None, None, i, CABEM, 0.3 + i as f32 * 0.74, 1.3);
    }
    for (i, q) in b.perguntas.iter().enumerate() {
        let marca = if q.resposta.is_empty() { Marca::Espera } else { Marca::Ok };
        por(Peca::Pergunta, format!("q:{}", q.id), &q.texto, marca, None, None, i, CABEM, 0.4 + i as f32 * 0.74, 2.4);
    }
    for (i, d) in b.telas.iter().enumerate() {
        // a que não coube ganha o lugar do último monitor, onde ela entra quando for escolhida
        let lugar = lugares.get(i).copied().or_else(|| tela::fileira(&[b.telas[0], b.telas[1], *d], comp).pop());
        let (u, fila) = lugar.map_or((0.0, 0.0), |l| (l.u, l.v));
        por(Peca::Tela, format!("t:{}", d.id), &d.titulo, Marca::de_revisao(d.pede_aprovacao, d.revisao.as_ref()), Some(*d), lugar, i, tela::MAX, u, fila);
    }
    for (i, k) in b.checkpoints.iter().enumerate() {
        por(Peca::Checkpoint, format!("k:{}", k.id), &k.nome, if k.feito { Marca::Ok } else { Marca::Neutra }, None, None, i, CABEM, 0.3 + i as f32 * 0.74, 3.2);
    }
    v
}

pub fn cor_da_marca(m: Marca) -> Color32 {
    match m {
        Marca::Espera => AZUL,
        Marca::Ok => VERDE,
        Marca::Problema => VERMELHO,
        Marca::Neutra => MEDIO,
    }
}

/// Põe a peça na cena, com o canto em `(x, y, z)`. `chegou` a faz crescer; a escolhida leva o traço forte.
#[allow(clippy::too_many_arguments)]
pub fn por_peca<'a>(cena: &mut Cena, extras: &mut Vec<Extra<'a>>, p: &PecaObra<'a>, xyz: (f32, f32, f32), chegou: f32, sel: bool, chave: u32) {
    por_peca_com(cena, extras, p, xyz, chegou, 1.0, sel, chave);
}

/// O mesmo, com `fica` apagando a peça sem encolher: o sumiço embaixo da coluna é só de tinta.
#[allow(clippy::too_many_arguments)]
pub fn por_peca_com<'a>(cena: &mut Cena, extras: &mut Vec<Extra<'a>>, p: &PecaObra<'a>, (x, y, z): (f32, f32, f32), chegou: f32, fica: f32, sel: bool, chave: u32) {
    let cor = cor_da_marca(p.marca);
    let alfa = chegou * fica;
    let tinta = Tinta::estado(cor).com_traco(if sel { TINTA } else { cor }, if sel { 2.0 } else { 1.1 }).com_alfa(alfa);
    let sobe = chegou.max(0.02);
    match p.tipo {
        Peca::Tela => {
            if let (Some(d), Some(l)) = (p.tela, p.lugar) {
                tela::por(cena, extras, d, &l, (x, y, z), cor, sel, alfa, chave);
            }
        }
        Peca::Decisao => cena.solido_folgado(Camada::Pecas, Malha::caixa([x, y, z], [0.55, 0.55, 0.42 * sobe]), tinta, chave, 0.0),
        Peca::Pergunta => {
            cena.solido(Camada::Pecas, Malha::cilindro((x, y), 0.035, z, 0.62 * sobe), tinta, chave);
            cena.solido_folgado(Camada::Pecas, Malha::esfera([x, y, z + 0.74 * sobe], 0.17), tinta, chave, 14.0);
        }
        Peca::Checkpoint => cena.solido_folgado(Camada::Pecas, Malha::caixa([x, y, z], [0.52, 0.52, 0.04]), tinta, chave, 0.0),
    }
}

/// A figura de verdade na face da tela, em quatro tamanhos: nítida dentro da tela, sem gastar nada de longe.
pub fn pintar_imagem(ctx: &Context, servidor: &str, d: &Detalhe, cam: &Camera, p: &Painter, cantos: &[V3; 4], alfa: f32) {
    let na_tela = cam.ponto(cantos[1]).distance(cam.ponto(cantos[0]));
    let largura = if na_tela <= 320.0 { 320.0 } else if na_tela <= 640.0 { 640.0 } else if na_tela <= 1280.0 { 1280.0 } else { 2560.0 };
    if let Some(t) = super::super::figura::textura(ctx, servidor, d.figura.as_ref(), largura) {
        imagem_na_face(cam, p, t, cantos, alfa);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plano::modelo::{Checkpoint, Plano};

    #[test]
    fn a_peca_segue_o_numero_do_plano() {
        let (k1, k2) = (Checkpoint { id: "a".into(), ..Checkpoint::default() }, Checkpoint { id: "b".into(), ..Checkpoint::default() });
        let b = Bloco { plano: Some(Plano { referencia: "1.1".into(), ..Plano::default() }), checkpoints: vec![&k1, &k2], ..Bloco::default() };
        let v = pecas_de(&b);
        assert_eq!((v[0].numero.as_str(), v[1].numero.as_str()), ("1.1.1", "1.1.2"));
    }
}
