//! Onde cada ação cai na folha do blueprint aberto: a largura da barra, o bloco do título e dos planos, a linha em que ela fica e a altura de
//! cada linha (`acoes` em web/js/render.js, `linhas` em dados.js, `disposicaoFolha` e `linhasDoPlano` em projetos.js). Só contas: nada de desenho.

use crate::dominio::{Acao, Janela};
use crate::i18n::tr;

/// A largura de uma letra de 11 px no tipo mono, e a mínima de uma barra aberta.
pub const CHAR: f32 = 6.9;
pub const BARRA_MIN: f32 = 220.0;
/// Altura de uma linha de ações aberta, do cabeçalho da seção até as ações e a folga de baixo.
pub const RH: f32 = 124.0;
pub const PH: f32 = 96.0;
pub const FOLGA: f32 = 76.0;

pub struct Posicao {
    /// o índice da ação na lista dada
    pub i: usize,
    pub rotulo: String,
    /// o começo e o fim da ação na régua (a barra recolhida usa estes)
    pub x0: f32,
    pub x1: f32,
    pub bw: f32,
    pub bx0: f32,
    pub lx: f32,
    pub lw: f32,
    pub bloco_w: f32,
    pub linhas_planos: Vec<Vec<String>>,
    pub alt: f32,
    pub linha: usize,
}

pub struct Disposicao {
    pub posicoes: Vec<Posicao>,
    /// o y de cada linha, a partir do topo do corpo da seção
    pub linha_y: Vec<f32>,
    /// a altura da seção inteira (cabeçalho, linhas e folga)
    pub alto: f32,
}

/// Quebra o texto sem cortar palavras e sem limite de linhas; palavra maior que a linha é partida (`linhasDoPlano`).
pub fn linhas_do_plano(texto: &str, largura: usize) -> Vec<String> {
    let limite = largura.max(1);
    let mut linhas = Vec::new();
    for paragrafo in texto.split('\n') {
        let mut linha = String::new();
        for palavra in paragrafo.split_whitespace() {
            let letras: Vec<char> = palavra.chars().collect();
            let mut inicio = 0;
            while letras.len() - inicio > limite {
                if !linha.is_empty() {
                    linhas.push(std::mem::take(&mut linha));
                }
                linhas.push(letras[inicio..inicio + limite].iter().collect());
                inicio += limite;
            }
            let resto: String = letras[inicio..].iter().collect();
            if !linha.is_empty() && linha.chars().count() + 1 + (letras.len() - inicio) > limite {
                linhas.push(std::mem::take(&mut linha));
            }
            if !linha.is_empty() {
                linha.push(' ');
            }
            linha += &resto;
        }
        linhas.push(linha);
    }
    linhas
}

/// Dispõe as ações de uma seção. `aberto`: com planos e checkpoints (senão só as barras compactas); `numero` dá o `1.2` de cada ação; `colw` é a largura de uma coluna de planos.
pub fn dispor(acoes: &[&Acao], aberto: bool, numero: impl Fn(&Acao) -> String, j: &Janela, l: f32, r: f32, colw: f32) -> Disposicao {
    let x = |d: i64| l + (d - j.t0) as f32 / (j.t1 - j.t0).max(1) as f32 * (r - l);
    let mut posicoes: Vec<Posicao> = acoes.iter().enumerate().map(|(i, f)| {
        let rotulo = format!("{} · {}", numero(f), if f.nome.is_empty() { tr("SEM NOME") } else { &f.nome });
        let lw = rotulo.chars().count() as f32 * (CHAR + 0.35) + 34.0;
        let (x0, x1) = (x(f.idx as i64), x(f.idx as i64 + 1));
        let (nt, nm) = (f.planos.len(), f.checkpoints.len());
        let ncol = usize::from(nt > 0) + usize::from(nm > 0);
        let bloco_w = if ncol > 0 { ncol as f32 * colw + (ncol - 1) as f32 * 30.0 } else { 0.0 };
        let bw = (x1 - x0).max(BARRA_MIN);
        let bx0 = l.max(x0.min(r - bw));
        // título, planos e checkpoints formam um bloco alinhado à esquerda, ao centro ou à direita, conforme o terço da folha em que a barra está
        let bloco3 = lw.max(bloco_w).min(r - l - 8.0);
        let terco = (bx0 + bw / 2.0 - l) / (r - l);
        let lx = if terco < 1.0 / 3.0 { l + 4.0 } else if terco > 2.0 / 3.0 { r - bloco3 } else { (l + r - bloco3) / 2.0 };
        let linhas_planos: Vec<Vec<String>> = f.planos.iter().map(|t| linhas_do_plano(if t.texto.is_empty() { tr("sem texto") } else { &t.texto }, ((colw - 28.0) / 7.3).floor() as usize)).collect();
        let altura_planos: f32 = linhas_planos.iter().map(|ls| (ls.len() as f32 * 14.0 + 8.0).max(22.0)).sum();
        let alt = if ncol > 0 { 100.0 + 34.0 + altura_planos.max(nm as f32 * 22.0) + 44.0 } else { RH };
        Posicao { i, rotulo, x0, x1: x1.max(x0), bw, bx0, lx, lw, bloco_w, linhas_planos, alt, linha: 0 }
    }).collect();
    // a escada: cada ação na sua própria linha, na ordem do blueprint
    posicoes.iter_mut().enumerate().for_each(|(k, p)| p.linha = k);
    let mut alturas = vec![if aberto { RH } else { 24.0 }; posicoes.len()];
    if aberto {
        for p in &posicoes {
            alturas[p.linha] = alturas[p.linha].max(p.alt);
        }
    }
    let mut linha_y = Vec::new();
    let mut corpo = 0.0;
    for h in &alturas {
        linha_y.push(corpo);
        corpo += h;
    }
    Disposicao { posicoes, linha_y, alto: if alturas.is_empty() { PH + 36.0 + FOLGA } else { PH + corpo + if aberto { 0.0 } else { 10.0 } + FOLGA } }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn quebra_palavras_sem_cortar_e_parte_as_longas() {
        assert_eq!(linhas_do_plano("Desenhar as telas principais", 12), ["Desenhar as", "telas", "principais"]);
        assert_eq!(linhas_do_plano("abcdefghij", 4), ["abcd", "efgh", "ij"]);
        assert_eq!(linhas_do_plano("a\n\nb", 10), ["a", "", "b"]);
        assert_eq!(linhas_do_plano("", 5), [""]);
    }

    /// O exemplo na janela de 1280: uma ação por linha, e a primeira linha mede 100 + 34 + 3 planos de 22 + 44 = 244 (conferido na tela web).
    #[test]
    fn exemplo_dispoe_uma_acao_por_linha() {
        let dados: serde_json::Value = serde_json::from_str(include_str!("../../../data.exemplo.json")).unwrap();
        let acoes = crate::dominio::acoes_de(&dados["blueprints"][0]);
        let j = crate::dominio::janela(&acoes);
        let refs: Vec<&Acao> = acoes.iter().collect();
        let d = dispor(&refs, true, |f| format!("1.{}", f.idx + 1), &j, 70.0, 1194.0, 430.0);
        let fechada = dispor(&refs, false, |f| format!("1.{}", f.idx + 1), &j, 70.0, 1194.0, 430.0);
        assert_eq!(fechada.alto, PH + 24.0 * fechada.linha_y.len() as f32 + 10.0 + FOLGA, "recolhida: linhas de 24 px");
        assert_eq!(d.posicoes.iter().map(|p| p.linha).collect::<Vec<_>>(), [0, 1, 2]);
        assert_eq!(d.linha_y[..2], [0.0, 244.0]);
        assert!(d.posicoes.iter().all(|p| p.bw >= BARRA_MIN && p.bx0 >= 70.0 && p.bx0 + p.bw <= 1194.0 + 0.01));
        assert_eq!(d.posicoes[0].rotulo, "1.1 · Protótipo navegável");
    }
}
