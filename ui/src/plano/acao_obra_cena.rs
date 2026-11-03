//! A cena da obra num instante: a chapa, os patamares, as construções na altura em que estão, os dutos da dependência e, no mergulho, as peças por dentro.
//! A planta fala sem texto: tracejado é o que ainda não existe, cheio é o que está de pé, o duto diz quem espera quem, e o cadeado na bandeira, quem ainda não pode subir.

use super::super::acao_estilo::{alfa as com_transparencia, mix};
use super::super::acao_movimento::{cascata, clarao, desde_que_abriu, desligado, hover, mistura, pulso};
use super::super::modelo::{Acao, Folha, Plano, Situacao};
use super::blocos::Bloco;
use super::dutos;
use super::figura::{feito, Aparencia, Peca, LADO};
use super::forma::indice;
use super::obra_figura::{chave_duto, chave_modulo, chave_peca, Modulo, Obra};
use super::obra_pecas::{por_peca, por_peca_com, Extra, Extras, PecaObra};
use super::obra_fora::{self as fora, Fora};
use super::obra_formas::Papel;
use super::obra_letreiro::Letreiro;
use super::obra_tela::{self as tela, Composicao};
use super::obra_regra;
use super::obra_sumico::{chave_se_clica, Sumico, SUMIU};
use super::obra_tempo;
use crate::anatomia::{Camada, Camera, Cena, Malha, Tinta};
use crate::tema::{AZUL, CARTAO, DESTAQUE, MEDIO, PAPEL, TINTA};
use egui::{Color32, Context, Rect, Stroke};

const FOLGA_PATAMAR: f32 = 0.35;
const ESPESSURA_CHAPA: f32 = 0.35;
const SEG_SUBIR: f32 = 0.9;
const CHEGADA: f32 = 1.6;
const ALTO_MASTRO: f32 = 1.05;

/// Tudo o que a cena precisa saber neste quadro.
pub struct Quadro<'q, 'a> {
    pub f: &'q Folha,
    pub de_fora: &'q [Fora],
    pub o: &'q Obra,
    pub planos: &'q [&'q Plano],
    pub lista: &'q [(&'q Plano, &'q Bloco<'a>)],
    pub pecas: &'q [Vec<PecaObra<'a>>],
    pub aceso: Option<usize>,
    pub dentro: Option<usize>,
    pub t_dentro: f32,
    pub tau: Option<f32>,
    pub peca: Option<&'q str>,
    /// a construção do próximo passo
    pub proximo: Option<usize>,
    /// a peça que o mouse acendeu, aqui ou na lista
    pub acesa: Option<String>,
    pub fase: f32,
    /// a câmera deste quadro: o sumiço mede na tela
    pub cam: &'q Camera,
    /// o que a coluna de texto cobre
    pub sumico: Sumico,
    /// o quanto cada construção fica além da coluna, de 0 a 1: a vista expandida apaga as outras e troca a escolhida pelas camadas
    pub apagar: &'q [f32],
}

impl Quadro<'_, '_> {
    /// O quanto a construção `i` aparece neste quadro: a coluna por cima e a vista expandida.
    pub fn fica(&self, i: usize) -> f32 {
        let Some(m) = self.o.modulos.get(i) else { return 1.0 };
        let (x, y, z) = (m.x, m.y, m.base);
        let lote = [[x, y, z], [x + LADO, y, z], [x, y + LADO, z], [x + LADO, y + LADO, z], [x + LADO * 0.5, y + LADO * 0.5, z + m.h]];
        self.sumico.fica(self.cam, &lote) * self.apagar.get(i).copied().unwrap_or(1.0)
    }

    /// O mesmo para o que se pinta por cima da cena, medido pela caixa dele na tela: a bandeira e a placa da construção `i`.
    pub fn fica_na_tela(&self, i: usize, r: Rect) -> f32 {
        self.sumico.fica_rect(r) * self.apagar.get(i).copied().unwrap_or(1.0)
    }

    /// A construção é o ponto de início ou o ponto em que a obra está: leva a cor de destaque, de longe e de perto.
    pub fn em_destaque(&self, i: usize) -> bool {
        self.tau.is_none() && (self.proximo == Some(i) || obra_regra::em_obra(self.planos[i]))
    }

    fn acende(&self, p: &PecaObra) -> bool {
        self.peca == Some(p.chave.as_str()) || self.acesa.as_deref() == Some(p.chave.as_str())
    }
}

/// O chão aceso da construção em destaque.
fn chao_aceso(respira: f32) -> Tinta {
    Tinta::de(mix(PAPEL, DESTAQUE, 0.16), mix(PAPEL, DESTAQUE, 0.5 + 0.12 * respira), DESTAQUE)
}

/// O quanto o plano está de pé de verdade.
pub fn altura_real(q: &Plano) -> f32 {
    if feito(q) {
        1.0
    } else if matches!(q.situacao, Situacao::Trabalhando | Situacao::Parado | Situacao::Bloqueado) {
        0.55
    } else {
        0.0
    }
}

/// O quanto o módulo `i` aparece de pé neste quadro: o instante da régua, ou a obra de verdade.
pub fn subiu_visto(ctx: &Context, a: &Acao, q: &Quadro, i: usize) -> f32 {
    if let Some(tau) = q.tau {
        return obra_tempo::subida(tau, q.o.modulos[i].nivel);
    }
    let alvo = altura_real(q.planos[i]);
    if desligado() {
        return alvo;
    }
    ctx.animate_value_with_time(egui::Id::unique(("plano-obra-subiu", &a.referencia, &q.planos[i].referencia)), alvo, SEG_SUBIR)
}

/// Quem está ligado a `i`, para cima e para baixo na dependência.
pub fn ligados(q: &[&Plano], i: usize) -> Vec<bool> {
    let mut v = vec![false; q.len()];
    v[i] = true;
    let mut pilha = vec![i];
    while let Some(k) = pilha.pop() {
        for d in q[k].depende_de.iter().filter_map(|d| indice(q, d)) {
            if !std::mem::replace(&mut v[d], true) {
                pilha.push(d);
            }
        }
    }
    let mut pilha = vec![i];
    let mut descendo = vec![false; q.len()];
    while let Some(k) = pilha.pop() {
        for (j, p) in q.iter().enumerate() {
            if !descendo[j] && p.depende_de.iter().any(|d| *d == q[k].referencia) {
                descendo[j] = true;
                v[j] = true;
                pilha.push(j);
            }
        }
    }
    v
}

fn chapa(cena: &mut Cena, o: &Obra) {
    let (x0, x1, y0, y1) = o.limites;
    cena.solido(Camada::Base, Malha::laje([x0, y0, -ESPESSURA_CHAPA], [x1 - x0, y1 - y0, ESPESSURA_CHAPA], 0.2), Tinta::chao(), 0);
    let d = 0.5;
    let anel = vec![[x0 + d, y0 + d, 0.0], [x1 - d, y0 + d, 0.0], [x1 - d, y1 - d, 0.0], [x0 + d, y1 - d, 0.0], [x0 + d, y0 + d, 0.0]];
    cena.fio(Camada::Base, anel, Stroke::new(0.7, MEDIO.gamma_multiply(0.4)), None);
    for (x, y) in [(x0 + 1.0, y0 + 1.0), (x1 - 1.0, y0 + 1.0), (x1 - 1.0, y1 - 1.0), (x0 + 1.0, y1 - 1.0)] {
        cena.solido(Camada::Base, Malha::cilindro((x, y), 0.22, 0.0, 0.03), Tinta::chao(), 0);
    }
}

/// A construção `i`: o patamar, o tracejado do que falta e o sólido do que já subiu. O monitor sem tela própria devolve o letreiro do plano, que toma o lugar do esboço no vidro.
fn construcao<'q>(ctx: &Context, a: &Acao, cena: &mut Cena, q: &Quadro<'q, '_>, i: usize, realce: f32, alfa: f32) -> Option<Letreiro<'q>> {
    let (plano, m) = (q.planos[i], &q.o.modulos[i]);
    let fica = q.fica(i);
    let chave = chave_se_clica(chave_modulo(i), fica);
    let (x, y, z) = (m.x, m.y, m.base + 0.12);
    let patamar = Malha::caixa([x - FOLGA_PATAMAR, y - FOLGA_PATAMAR, 0.0], [LADO + FOLGA_PATAMAR * 2.0, LADO + FOLGA_PATAMAR * 2.0, z]);
    let aqui = q.em_destaque(i);
    let respira = if desligado() { 1.0 } else { pulso(ctx, 1.8) };
    cena.solido(Camada::Chao, patamar, if aqui { chao_aceso(respira) } else { Tinta::chao() }.com_alfa(alfa.max(0.5) * fica), chave);
    let subiu = subiu_visto(ctx, a, q, i);
    let aberto = if q.dentro == Some(i) { q.t_dentro } else { 0.0 };
    let vivo = if plano.situacao == Situacao::Trabalhando && q.tau.is_none() { pulso(ctx, 1.6) } else { 0.0 };
    let simulado = q.tau.is_some() && !feito(plano);
    let base_cor = if simulado { AZUL } else if aqui { DESTAQUE } else { Aparencia::de(plano, 0.0).traco };
    let cor = mix(base_cor, TINTA, (realce * 0.7 + vivo * 0.5).min(1.0));
    let comp = tela::composicao(plano, !q.lista[i].1.telas.is_empty());
    // por dentro, a construção só clareia o bastante para as peças aparecerem; sem peças, fica inteira: de perto ela precisa continuar à vista
    let tem_pecas = q.pecas[i].iter().any(|p| p.visivel && p.tipo != Peca::Tela);
    let some = if tem_pecas { 0.45 } else { 0.0 };
    let visto = alfa * fica * (1.0 - aberto * some);
    let letreiro = Letreiro::do_plano(plano, comp, (x, y, z), m.h, subiu, visto);
    corpo_de(cena, plano, comp, (x, y, z), m.h, subiu, cor, 1.3 + 0.5 * realce, visto, chave, letreiro.is_none());
    letreiro
}

/// A construção em si, na composição dela: o tracejado do que falta e o sólido do que já subiu.
#[allow(clippy::too_many_arguments)]
pub fn corpo(cena: &mut Cena, plano: &Plano, comp: Composicao, xyz: (f32, f32, f32), h: f32, subiu: f32, cor: Color32, largura: f32, alfa: f32, chave: u32) {
    corpo_de(cena, plano, comp, xyz, h, subiu, cor, largura, alfa, chave, true);
}

/// O mesmo corpo; sem `esboco`, o vidro do monitor fica liso para o letreiro.
#[allow(clippy::too_many_arguments)]
fn corpo_de(cena: &mut Cena, plano: &Plano, comp: Composicao, (x, y, z): (f32, f32, f32), h: f32, subiu: f32, cor: Color32, largura: f32, alfa: f32, chave: u32, esboco: bool) {
    if subiu < 1.0 {
        for (malha, papel) in tela::partes(comp, plano, x, y, z, h) {
            if papel == Papel::Corpo {
                cena.solido(Camada::Pecas, malha, Tinta::fantasma(cor).com_traco(cor, largura.min(1.6)).com_alfa((alfa * (1.0 - subiu * 0.6)).min(1.0)), chave);
            }
        }
    }
    if subiu > 0.0 {
        let parede = Tinta::parede().com_traco(cor, largura).com_alfa(alfa);
        let detalhe = Tinta::de(CARTAO, PAPEL, cor.gamma_multiply(0.7)).com_alfa(alfa);
        for (malha, papel) in tela::partes(comp, plano, x, y, z, h * subiu).into_iter().filter(|(_, p)| esboco || *p != Papel::Esboco) {
            cena.solido(Camada::Pecas, malha, if papel == Papel::Corpo { parede } else { detalhe }, chave);
        }
    }
}

/// A bandeira em cima de quem espera uma decisão: de longe, diz onde a pessoa é chamada.
fn bandeira(cena: &mut Cena, m: &Modulo, chave: u32, alfa: f32) {
    let (x, y, z0) = (m.x + LADO * 0.5, m.y + LADO * 0.5, m.base + 0.12 + m.h);
    let z1 = z0 + ALTO_MASTRO;
    cena.fio(Camada::Acima, vec![[x, y, z0], [x, y, z1]], Stroke::new(1.3, AZUL.gamma_multiply(alfa)), None);
    let pano = vec![[x, y, z1], [x + 0.52, y - 0.52, z1 - 0.2], [x, y, z1 - 0.4]];
    cena.solido(Camada::Acima, Malha::de_faces(pano, vec![vec![0, 1, 2], vec![2, 1, 0]]), Tinta::estado(AZUL).com_alfa(alfa), chave);
}

/// O tracejado que anda em volta da construção em destaque: de longe, diz por onde começar ou onde a obra está.
fn proximo_passo(cena: &mut Cena, m: &Modulo, fase: f32, respira: f32, alfa: f32) {
    let d = FOLGA_PATAMAR + 0.28;
    let (x0, y0, x1, y1) = (m.x - d, m.y - d, m.x + LADO + d, m.y + LADO + d);
    let anel = vec![[x0, y0, 0.03], [x1, y0, 0.03], [x1, y1, 0.03], [x0, y1, 0.03], [x0, y0, 0.03]];
    cena.fio(Camada::Base, anel, Stroke::new(1.8, com_transparencia(DESTAQUE, alfa * (0.6 + 0.4 * respira))), Some((7.0, 5.0, -fase * 12.0)));
}

/// As telas da construção: de pé o tempo todo, de longe e de perto, no mesmo lugar. Dá para clicar nelas sem antes entrar.
fn telas_de_pe<'q>(cena: &mut Cena, extras: &mut Vec<Extra<'q>>, q: &Quadro<'q, '_>, i: usize, ctx: &Context, a: &Acao, alfa: f32) {
    let m = &q.o.modulos[i];
    let fica = q.fica(i);
    let escolhida_fora =q.pecas[i].iter().enumerate().find(|(_, p)| p.tipo == Peca::Tela && !p.visivel && q.dentro == Some(i) && q.peca == Some(p.chave.as_str()));
    for (n, (j, p)) in q.pecas[i].iter().enumerate().filter(|(_, p)| p.tipo == Peca::Tela && p.visivel).enumerate() {
        // a tela que não coube entra no último monitor enquanto estiver escolhida
        let (j, p) = if n + 1 == tela::MAX { escolhida_fora.unwrap_or((j, p)) } else { (j, p) };
        let sel = q.dentro == Some(i) && q.acende(p);
        let sobe = mistura(ctx, ("plano-obra-peca-sel", &a.referencia, &p.chave), sel, 0.2) * 0.14;
        let chao = p.lugar.map_or(0.0, |l| l.chao);
        por_peca(cena, extras, p, (m.x + p.u, m.y + p.v, m.base + 0.12 + chao + sobe), alfa * fica, sel, chave_se_clica(chave_peca(i, j), fica));
    }
}

/// Por dentro: o piso, e cada peça em cima dele, grande o bastante para clicar. A tela já está de pé.
fn por_dentro<'q>(cena: &mut Cena, extras: &mut Vec<Extra<'q>>, q: &Quadro<'q, '_>, i: usize, ctx: &Context, a: &Acao) {
    let fica = q.fica(i);
    if fica < SUMIU {
        return;
    }
    let m = &q.o.modulos[i];
    let z = m.base + 0.12;
    let t = q.t_dentro;
    cena.solido(Camada::Chao, Malha::caixa([m.x, m.y, z], [LADO, LADO, 0.05]), Tinta::de(CARTAO, PAPEL, MEDIO).com_alfa(t * fica), chave_se_clica(chave_modulo(i), fica));
    let z = z + 0.05;
    for (j, p) in q.pecas[i].iter().enumerate().filter(|(_, p)| p.visivel && p.tipo != Peca::Tela) {
        let chegou = cascata(ctx, desde_que_abriu(ctx, ("plano-obra-dentro", &a.referencia, &q.planos[i].referencia)), j + 3, 0.0).min(t);
        let sel = q.acende(p);
        let sobe = mistura(ctx, ("plano-obra-peca-sel", &a.referencia, &p.chave), sel, 0.2) * 0.14;
        por_peca_com(cena, extras, p, (m.x + p.u, m.y + p.v, z + sobe), chegou, fica, sel, chave_se_clica(chave_peca(i, j), fica));
    }
}

/// Monta a cena deste quadro. Devolve o que só o palco sabe pintar.
pub fn montar<'q>(ctx: &Context, a: &Acao, q: &Quadro<'q, '_>) -> (Cena, Extras<'q>) {
    let mut cena = Cena::default();
    let mut extras = Extras::default();
    chapa(&mut cena, q.o);
    let ms = desde_que_abriu(ctx, ("acao-obra", &a.referencia));
    let liga = q.aceso.or(q.dentro).map(|i| ligados(q.planos, i));
    let esta_ligado = |i: usize| liga.as_ref().is_none_or(|l| l[i]);
    let vistos: Vec<f32> = (0..q.planos.len()).map(|i| subiu_visto(ctx, a, q, i)).collect();
    for u in &dutos::todos(q.planos) {
        let aceso = hover(ctx, ("plano-obra-duto", &a.referencia, u.n), liga.is_some() && esta_ligado(u.de) && esta_ligado(u.para));
        let alfa = if esta_ligado(u.de) && esta_ligado(u.para) { 1.0 } else { 1.0 - 0.7 * if q.dentro.is_some() { q.t_dentro } else { 1.0 } };
        // a esteira some com as construções das pontas e com o lugar dela na tela: a que passa atrás do texto vai junto
        let pontos = dutos::pontos(q.o, u);
        let fica = q.fica(u.de).min(q.fica(u.para)).min(q.sumico.fica(q.cam, &pontos));
        if fica >= SUMIU {
            dutos::esteira(&mut cena, &pontos, vistos[u.de] >= 1.0, vistos[u.para], aceso, alfa * fica * cascata(ctx, ms, q.planos.len() + u.n, 0.0), q.fase, chave_se_clica(chave_duto(u.n), fica));
        }
    }
    for u in q.de_fora {
        let fica = q.fica(u.para).min(q.sumico.fica(q.cam, &fora::trecho(q.o, u)));
        if fica >= SUMIU {
            fora::por(&mut cena, q.o, u, vistos[u.para], fica * if esta_ligado(u.para) { 1.0 } else { 0.35 }, q.fase, chave_se_clica(chave_modulo(u.para), fica));
        }
    }
    for i in 0..q.planos.len() {
        let plano = q.planos[i];
        let chegou = cascata(ctx, ms, i + 1, 0.0);
        let chave = (a.referencia.as_str(), plano.referencia.as_str());
        let brilho = clarao(ctx, ("plano-obra-chegou", chave.0, chave.1), CHEGADA);
        let realce = hover(ctx, ("plano-obra-sobre", chave), q.aceso == Some(i) || q.dentro == Some(i)).max(brilho);
        let apaga = mistura(ctx, ("plano-obra-apaga", chave), !esta_ligado(i), 0.2);
        let longe = if q.dentro.is_some_and(|d| d != i) { q.t_dentro * 0.45 } else { 0.0 };
        let alfa = chegou * (1.0 - 0.62 * apaga) * (1.0 - longe);
        let fica = q.fica(i);
        if fica < SUMIU {
            continue;
        }
        if q.em_destaque(i) {
            proximo_passo(&mut cena, &q.o.modulos[i], q.fase, if desligado() { 1.0 } else { pulso(ctx, 1.8) }, chegou * fica);
        }
        if let Some(l) = construcao(ctx, a, &mut cena, q, i, realce, alfa) {
            extras.letreiro(&mut cena, l);
        }
        telas_de_pe(&mut cena, &mut extras.imagens, q, i, ctx, a, alfa);
        if q.lista[i].1.a_decidir().0 > 0 && q.tau.is_none() {
            bandeira(&mut cena, &q.o.modulos[i], chave_se_clica(chave_modulo(i), fica), alfa * fica * (1.0 - q.t_dentro));
        }
    }
    if let Some(i) = q.dentro.filter(|_| q.t_dentro > 0.02) {
        por_dentro(&mut cena, &mut extras.imagens, q, i, ctx, a);
    }
    (cena, extras)
}

/// A cena de quem olha de longe, na linha da folha: sem peças por dentro, sem agente e sem movimento, para não custar nada parada. As telas vão de pé, como no palco.
pub fn miniatura<'q, 'a>(o: &Obra, lista: &[(&Plano, &Bloco<'a>)], pecas: &'q [Vec<PecaObra<'a>>], ponto: Option<&str>) -> (Cena, Vec<Extra<'a>>) {
    let mut cena = Cena::default();
    let mut extras = Vec::new();
    chapa(&mut cena, o);
    let planos: Vec<&Plano> = lista.iter().map(|(q, _)| *q).collect();
    for u in &dutos::todos(&planos) {
        dutos::por(&mut cena, o, u, 0.0, 1.0, altura_real(planos[u.de]), altura_real(planos[u.para]), 0.4);
    }
    for (i, (q, b)) in lista.iter().enumerate() {
        let m = &o.modulos[i];
        let z = m.base + 0.12;
        let patamar = Malha::caixa([m.x - FOLGA_PATAMAR, m.y - FOLGA_PATAMAR, 0.0], [LADO + FOLGA_PATAMAR * 2.0, LADO + FOLGA_PATAMAR * 2.0, z]);
        let aqui = ponto == Some(q.referencia.as_str());
        cena.solido(Camada::Chao, patamar, if aqui { chao_aceso(1.0) } else { Tinta::chao() }, 0);
        corpo(&mut cena, q, tela::composicao(q, !b.telas.is_empty()), (m.x, m.y, z), m.h, altura_real(q), if aqui { DESTAQUE } else { Aparencia::de(q, 0.0).traco }, 1.0, 1.0, 0);
        for p in pecas[i].iter().filter(|p| p.tipo == Peca::Tela && p.visivel) {
            por_peca(&mut cena, &mut extras, p, (m.x + p.u, m.y + p.v, z + p.lugar.map_or(0.0, |l| l.chao)), 1.0, false, 0);
        }
        if b.a_decidir().0 > 0 {
            bandeira(&mut cena, m, 0, 1.0);
        }
    }
    (cena, extras)
}

#[path = "acao_obra_toque.rs"]
mod toque;
pub use toque::{clique_da_chave, duto_da_chave, o_que_e};

#[cfg(test)]
mod tests {
    use super::*;

    fn plano(r: &str, dep: &[&str]) -> Plano {
        Plano { referencia: r.into(), depende_de: dep.iter().map(|d| d.to_string()).collect(), ..Plano::default() }
    }

    #[test]
    fn acender_um_plano_acende_quem_ele_espera_e_quem_espera_por_ele() {
        let p = [plano("1", &[]), plano("2", &["1"]), plano("3", &["1"]), plano("4", &["2"])];
        let q: Vec<&Plano> = p.iter().collect();
        assert_eq!(ligados(&q, 1), vec![true, true, false, true]);
    }
}
