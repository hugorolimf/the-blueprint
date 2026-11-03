//! O desenho explodido de uma construção: a mesma construção da planta separada em camadas. Embaixo a base, depois os checkpoints, as perguntas, as decisões e as telas; em cima, a construção.
//! Não é outra tela: é o nível do meio do zoom num plano, no próprio palco. `acao_obra_explosao.rs` diz quando e onde; aqui ficam as camadas, a altura de cada uma conforme a separação e como elas entram na cena do palco, subindo do lote da construção. As chamadas de cada camada estão em `acao_partes_chamada.rs`.

use super::super::acao_movimento::mistura;
use super::super::modelo::{Acao, Plano};
use super::blocos::Bloco;
use super::figura::{Peca, LADO};
use super::obra_cena::corpo;
use super::obra_figura::ALTO;
use super::obra_mergulho as mergulho;
use super::obra_pecas::{por_peca, Extra, PecaObra};
use super::obra_tela::{Composicao, Lugar};
use crate::anatomia::{Camada, Cena, Malha, Tinta, V3};
use crate::tema::{MEDIO, PAPEL, TINTA};
use egui::{Color32, Stroke};

#[path = "acao_partes_chamada.rs"]
pub mod chamada;

const SEG_EXPLODE: f32 = 0.75;
const ESPESSURA: f32 = 0.07;
/// A borda da base em volta do lote: a mesma do patamar da construção.
pub const FOLGA: f32 = 0.35;
/// Na bandeja a tela fica num pé curto, e não no alto da construção.
pub const PE_NA_BANDEJA: f32 = 0.12;
/// Abaixo disto a camada ainda está chegando e não responde ao clique.
const TOCA_DESDE: f32 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Nivel {
    Base,
    Checkpoints,
    Perguntas,
    Decisoes,
    Telas,
    Obra,
}

impl Nivel {
    /// O vão até a camada de cima, com tudo separado.
    fn vao(self) -> f32 {
        match self {
            Nivel::Base | Nivel::Checkpoints => 1.0,
            Nivel::Perguntas => 1.6,
            Nivel::Telas => 2.5,
            Nivel::Decisoes => 1.2,
            Nivel::Obra => ALTO,
        }
    }

    pub fn peca(self) -> Option<Peca> {
        match self {
            Nivel::Checkpoints => Some(Peca::Checkpoint),
            Nivel::Perguntas => Some(Peca::Pergunta),
            Nivel::Decisoes => Some(Peca::Decisao),
            Nivel::Telas => Some(Peca::Tela),
            _ => None,
        }
    }
}

/// As camadas do plano: só entra a que tem peça. A base e a construção estão sempre lá.
pub fn niveis(b: &Bloco) -> Vec<Nivel> {
    let mut v = vec![Nivel::Base];
    v.extend(b.checkpoints.first().map(|_| Nivel::Checkpoints));
    v.extend(b.perguntas.first().map(|_| Nivel::Perguntas));
    v.extend(b.decisoes.first().map(|_| Nivel::Decisoes));
    v.extend(b.telas.first().map(|_| Nivel::Telas));
    v.push(Nivel::Obra);
    v
}

/// A altura de cada camada acima do lote: `t` 0 é tudo junto, como na planta; 1 é separado.
pub fn alturas(n: &[Nivel], t: f32) -> Vec<f32> {
    let mut z = 0.0;
    n.iter().enumerate().map(|(i, k)| {
        let junto = i as f32 * 0.1;
        let aqui = junto + (z - junto) * t;
        z += k.vao();
        aqui
    }).collect()
}

/// Quanto as camadas estão separadas. Precisa ser lido a cada quadro, aberta ou não, para a abertura andar.
pub fn separacao(ctx: &egui::Context, a: &Acao, q: &str, aberta: bool) -> f32 {
    mistura(ctx, ("plano-parte-explode", &a.referencia, q), aberta, SEG_EXPLODE)
}

/// Abre a vista expandida de `plano`: as camadas se separam ali mesmo, e a câmera do palco voa até a pilha.
pub fn abrir(ctx: &egui::Context, a: &Acao, plano: &str) {
    mergulho::expandir(ctx, a, plano);
}

fn bandeja() -> Tinta {
    Tinta { tons: [PAPEL.gamma_multiply(0.16); 5], traco: Stroke::new(1.0, TINTA.gamma_multiply(0.6)), vinco: Stroke::NONE, tracejado: false, alfa: 1.0 }
}

/// Onde a peça `k` de `n` começa, para a fileira ficar no meio da bandeja.
fn coluna(tipo: Peca, k: usize, n: usize) -> f32 {
    let (lado, passo) = match tipo {
        Peca::Tela => (1.12, 1.25),
        Peca::Pergunta => (0.0, 0.74),
        _ => (0.55, 0.74),
    };
    (LADO - lado - passo * (n as f32 - 1.0)) * 0.5 + passo * k as f32
}

/// A fileira de peças em cima da bandeja, centrada no fundo dela.
pub fn fila_de(tipo: Peca) -> f32 {
    match tipo {
        Peca::Tela => LADO * 0.5 - 0.1,
        Peca::Pergunta => LADO * 0.5,
        _ => LADO * 0.5 - 0.27,
    }
}

/// Uma construção explodida neste quadro, pronta para entrar na cena do palco.
pub struct Pilha<'p, 'a> {
    pub q: &'p Plano,
    pub pecas: &'p [PecaObra<'a>],
    pub comp: Composicao,
    pub niveis: &'p [Nivel],
    /// a altura de cada camada acima do lote
    pub zs: &'p [f32],
    /// o quanto cada camada aparece, de 0 a 1
    pub alfas: Vec<f32>,
    /// o canto do lote, no chão da construção, em coordenadas do mundo do palco
    pub lote: V3,
    pub subiu: f32,
    pub cor: Color32,
    /// a peça escolhida e a acesa: sobem um pouco e levam o traço forte
    pub acesas: [Option<&'p str>; 2],
    /// a chave da construção de cima; a da peça `j` é `chave + j + 1`. Zero: nada responde
    pub chave: u32,
}

/// Põe as camadas na cena, cada uma num grupo acima do anterior: pintam depois de tudo o que já está nela, e o clique as acha primeiro.
pub fn montar<'a>(ctx: &egui::Context, a: &Acao, cena: &mut Cena, extras: &mut Vec<Extra<'a>>, p: &Pilha<'_, 'a>) {
    let [x0, y0, z0] = p.lote;
    let topo = p.zs.last().copied().unwrap_or(0.0);
    let guia = p.alfas.iter().copied().fold(0.0, f32::max);
    // os quatro fios dos cantos do lote: dizem de onde as camadas saíram
    for (x, y) in [(0.0, 0.0), (LADO, 0.0), (LADO, LADO), (0.0, LADO)] {
        cena.fio(Camada::Chao, vec![[x0 + x, y0 + y, z0], [x0 + x, y0 + y, z0 + topo]], Stroke::new(0.8, MEDIO.gamma_multiply(0.7 * guia)), Some((5.0, 4.0, 0.0)));
    }
    for (i, (n, dz)) in p.niveis.iter().zip(p.zs).enumerate() {
        let alfa = p.alfas.get(i).copied().unwrap_or(0.0);
        if alfa <= 0.01 {
            continue;
        }
        let toca = |c: u32| if alfa >= TOCA_DESDE && p.chave != 0 { c } else { 0 };
        let z = z0 + dz;
        cena.grupo(i as i32 + 1);
        match n {
            // a base é o próprio patamar do lote, do chão da chapa até o piso da construção
            Nivel::Base => cena.solido(Camada::Chao, Malha::caixa([x0 - FOLGA, y0 - FOLGA, 0.0], [LADO + FOLGA * 2.0, LADO + FOLGA * 2.0, z.max(0.05)]), Tinta::chao().com_alfa(alfa), 0),
            Nivel::Obra => corpo(cena, p.q, p.comp, (x0, y0, z), ALTO, p.subiu, p.cor, 1.2, alfa, toca(p.chave)),
            _ => {
                // a bandeja é só o contorno e um véu: dá para ver as peças das camadas de baixo
                cena.solido(Camada::Chao, Malha::laje([x0, y0, z - ESPESSURA], [LADO, LADO, ESPESSURA], 0.12), bandeja().com_alfa(alfa), 0);
                let da_camada: Vec<(usize, &PecaObra)> = p.pecas.iter().enumerate().filter(|(_, q)| n.peca() == Some(q.tipo) && q.visivel).collect();
                for (k, (j, q)) in da_camada.iter().enumerate() {
                    let sel = p.acesas.contains(&Some(q.chave.as_str()));
                    let sobe = mistura(ctx, ("plano-parte-peca-sel", &a.referencia, &q.chave), sel, 0.2) * 0.16;
                    let lugar = q.lugar.map(|l| Lugar { pe: PE_NA_BANDEJA, ..l });
                    let x = lugar.map_or_else(|| coluna(q.tipo, k, da_camada.len()), |l| l.u);
                    por_peca(cena, extras, &PecaObra { lugar, ..(*q).clone() }, (x0 + x, y0 + fila_de(q.tipo), z + sobe), alfa, sel, toca(p.chave + *j as u32 + 1));
                }
            }
        }
    }
    cena.grupo(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn juntas_na_planta_e_separadas_na_vista_expandida() {
        let n = [Nivel::Base, Nivel::Decisoes, Nivel::Obra];
        assert_eq!(alturas(&n, 1.0), vec![0.0, 1.0, 2.2]);
        assert!(alturas(&n, 0.0)[2] < 0.3);
    }

    #[test]
    fn a_fileira_fica_no_meio_da_bandeja() {
        assert!((coluna(Peca::Decisao, 0, 1) - (LADO - 0.55) * 0.5).abs() < 1e-5);
        let (a, b) = (coluna(Peca::Tela, 0, 2), coluna(Peca::Tela, 1, 2));
        assert!((a - (LADO - (b + 1.12))).abs() < 1e-5);
    }

    #[test]
    fn so_entra_a_camada_que_tem_peca() {
        assert_eq!(niveis(&Bloco::default()), vec![Nivel::Base, Nivel::Obra]);
    }
}
