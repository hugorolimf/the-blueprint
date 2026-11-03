//! As esteiras entre as ações, na folha: quem espera um plano de outra ação recebe uma, saindo do pé da obra de lá e entrando por baixo na obra daqui. A folha inteira vira uma linha de montagem, como no Factorio.
//! Só a esteira mais curta: se esta ação espera a anterior, e a anterior espera a primeira, não precisa de outra vinda da primeira. Elas correm por baixo das linhas, sem riscar nome nenhum, e passam por baixo do que está escrito no caminho.
//! Quem liga a quem, as pontas e o que cada uma cruza só mudam com a folha e a escada: são medidos uma vez (`Folha::geracao`), e a cada quadro só entram as alturas das linhas.

use super::bloco::{faixa_do_texto, ALTO_MINI, W_MINI};
use super::Pincel;
use crate::plano::acao;
use crate::plano::escala::Escala;
use crate::plano::folha::estilo::com_alfa;
use crate::plano::modelo::{Acao, Folha, Situacao};
use crate::tema::{MEDIO, VERDE};
use egui::{vec2, Id, Pos2, Rect, Shape, Stroke, Vec2};
use std::collections::HashMap;
use std::sync::Arc;

/// O que a esteira não risca em volta do meio de um cabeçalho de projeto e de um texto de ação.
const SOBE_CABECA: f32 = 38.0;
const DESCE_CABECA: f32 = 26.0;
const SOBE_TEXTO: f32 = 34.0;
const DESCE_TEXTO: f32 = 36.0;
/// Onde a esteira corre, abaixo do meio da linha: logo embaixo da miniatura, onde não há texto.
const DESCE_PISTA: f32 = 52.0;
const TAM_SETA: f32 = 5.0;

/// Uma esteira direta, medida uma vez por folha. As pontas valem para qualquer altura da linha: o `x` na página e o quanto ficam abaixo do meio dela.
#[derive(Clone, Debug)]
struct Esteira {
    /// a linha que espera e a que entrega
    para: usize,
    de: usize,
    x0: f32,
    desce0: f32,
    x1: f32,
    desce1: f32,
    /// tudo o que ela traz já chegou
    ja: bool,
    /// as linhas com o texto no caminho do pé que sai: ele passa por baixo delas
    cobrem: Vec<usize>,
}

/// A folha, a escada e a escala da tela em que as esteiras foram medidas.
type Chave = (u64, u32, u32, usize, u32);

/// `de` chega em `ate` seguindo as dependências?
fn alcanca(deps: &[Vec<usize>], de: usize, ate: usize) -> bool {
    let mut vistos = vec![false; deps.len()];
    let mut pilha = vec![de];
    while let Some(i) = pilha.pop() {
        if i == ate {
            return true;
        }
        if !std::mem::replace(&mut vistos[i], true) {
            pilha.extend(&deps[i]);
        }
    }
    false
}

/// As dependências entre as ações, por linha, sem as que outro caminho já cobre.
fn so_as_diretas(deps: &[Vec<usize>]) -> Vec<Vec<usize>> {
    deps.iter().map(|v| v.iter().copied().filter(|d| !v.iter().any(|m| m != d && alcanca(deps, *m, *d))).collect()).collect()
}

fn dependencias(acoes: &[&Acao]) -> Vec<Vec<usize>> {
    // a dona de cada plano é a primeira ação que o tem
    let mut dona: HashMap<&str, usize> = HashMap::new();
    for (i, a) in acoes.iter().enumerate() {
        for q in &a.planos {
            dona.entry(q.referencia.as_str()).or_insert(i);
        }
    }
    acoes.iter().enumerate().map(|(i, a)| {
        let mut v: Vec<usize> = a.planos.iter().flat_map(|q| &q.depende_de).filter_map(|r| dona.get(r.as_str()).copied()).filter(|d| *d != i).collect();
        v.sort_unstable();
        v.dedup();
        v
    }).collect()
}

/// Tudo o que `de` entrega a `para` já chegou?
fn chegou(de: &Acao, para: &Acao) -> bool {
    para.planos.iter().flat_map(|q| &q.depende_de).filter_map(|d| de.planos.iter().find(|q| q.referencia == *d)).all(|q| q.feito || q.situacao == Situacao::Pronto)
}

fn traco_de(p: &Pincel, pontos: [(f32, f32); 2], traco: Stroke, cheia: bool) {
    let linha = [p.pt(pontos[0].0, pontos[0].1), p.pt(pontos[1].0, pontos[1].1)];
    if cheia {
        p.p.add(Shape::line(linha.to_vec(), traco));
    } else {
        p.p.extend(Shape::dashed_line(&linha, traco, 6.0, 4.0));
    }
}

/// Um trecho de pé, pulando as `faixas` que ele não pode riscar.
fn de_pe(p: &Pincel, x: f32, y0: f32, y1: f32, faixas: &mut [(f32, f32)], traco: Stroke, cheia: bool) {
    faixas.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut y = y0;
    for (a, b) in faixas.iter().filter(|(a, b)| *b > y0 && *a < y1) {
        if *a > y {
            traco_de(p, [(x, y), (x, *a)], traco, cheia);
        }
        y = y.max(*b);
    }
    if y < y1 {
        traco_de(p, [(x, y), (x, y1)], traco, cheia);
    }
}

/// As esteiras diretas, na ordem em que são desenhadas. O `y` cresce com a linha (`empilhar`), então «a de lá fica acima» é só o índice menor.
/// As pontas saem da miniatura com o canto em zero: a câmera dela só anda junto com a linha.
fn medir(p: &Pincel, e: &Escala, f: &Folha, acoes: &[&Acao]) -> Vec<Esteira> {
    let deps = dependencias(acoes);
    if deps.iter().all(Vec::is_empty) {
        return Vec::new();
    }
    let textos: Vec<Option<(f32, f32)>> = acoes.iter().map(|a| faixa_do_texto(p, f, e, a)).collect();
    let caixa = Rect::from_min_size(Pos2::ZERO, vec2(W_MINI, ALTO_MINI));
    let mut pontas: Vec<Option<Option<(f32, Vec2)>>> = vec![None; acoes.len()];
    let mut ponta = |k: usize| *pontas[k].get_or_insert_with(|| e.trecho(acoes[k]).map(|(xa, _)| (xa, acao::pe(acoes[k], caixa) - caixa.min)));
    let mut lista = Vec::new();
    for (i, diretas) in so_as_diretas(&deps).iter().enumerate() {
        let acima: Vec<usize> = diretas.iter().copied().filter(|d| *d < i).collect();
        if acima.is_empty() {
            continue;
        }
        let Some((x_para, chega)) = ponta(i) else { continue };
        for d in acima {
            let Some((x_de, sai)) = ponta(d) else { continue };
            let x0 = x_de + sai.x;
            let cobrem = textos.iter().enumerate().filter(|(_, t)| t.is_some_and(|(a, b)| x0 > a - 4.0 && x0 < b + 4.0)).map(|(k, _)| k).collect();
            let (desce0, desce1) = (sai.y - ALTO_MINI / 2.0, chega.y - ALTO_MINI / 2.0);
            lista.push(Esteira { para: i, de: d, x0, desce0, x1: x_para + chega.x, desce1, ja: chegou(acoes[d], acoes[i]), cobrem });
        }
    }
    lista
}

/// As esteiras desta folha, medidas na primeira vez e guardadas no contexto. Folha sem geração não guarda.
fn guardadas(p: &Pincel, e: &Escala, f: &Folha, linhas: &[(usize, usize, f32)]) -> Arc<Vec<Esteira>> {
    let acoes = || linhas.iter().map(|l| &f.projetos[l.0].acoes[l.1]).collect::<Vec<&Acao>>();
    if f.geracao == 0 {
        return Arc::new(medir(p, e, f, &acoes()));
    }
    let (id, ctx) = (Id::unique("plano-esteiras"), p.p.ctx());
    let chave: Chave = (f.geracao, e.x0.to_bits(), e.x1.to_bits(), e.total(), p.p.pixels_per_point().to_bits());
    if let Some((c, v)) = ctx.data(|d| d.get_temp::<(Chave, Arc<Vec<Esteira>>)>(id)) {
        if c == chave {
            return v;
        }
    }
    let v = Arc::new(medir(p, e, f, &acoes()));
    ctx.data_mut(|d| d.insert_temp(id, (chave, v.clone())));
    v
}

/// Desenha as esteiras. Uma ação aberta no caminho interrompe a esteira, em vez de ela cruzar o palco. `cabecas`: o meio do cabeçalho de cada projeto.
/// `abertas`: se a ação de cada linha está aberta (ou abrindo), na ordem de `linhas`.
pub fn desenhar(p: &Pincel, e: &Escala, f: &Folha, linhas: &[(usize, usize, f32)], cabecas: &[f32], abertas: &[bool]) {
    let esteiras = guardadas(p, e, f, linhas);
    if esteiras.is_empty() {
        return;
    }
    // quantas linhas abertas vêm antes de cada uma: entre a de lá e a daqui há uma aberta se a conta muda
    let mut antes = Vec::with_capacity(linhas.len() + 1);
    antes.push(0usize);
    for k in 0..linhas.len() {
        antes.push(antes[k] + usize::from(abertas.get(k).copied().unwrap_or(false)));
    }
    let cabeca: Vec<(f32, f32)> = cabecas.iter().map(|c| (c - SOBE_CABECA, c + DESCE_CABECA)).collect();
    let visivel = p.p.clip_rect();
    for s in esteiras.iter() {
        if antes[s.para] != antes[s.de] {
            continue;
        }
        let (y, y_de) = (linhas[s.para].2, linhas[s.de].2);
        let (x0, y0, x1, y1, pista) = (s.x0, y_de + s.desce0, s.x1, y + s.desce1, y + DESCE_PISTA);
        // o que ela risca: os dois pés, a pista e a seta, com a grossura do traço; fora da tela, nada aparece
        let (topo, fundo) = (y0.min(y1).min(pista), pista.max(y1 + TAM_SETA).max(y0));
        let area = Rect::from_min_max(p.pt(x0.min(x1) - TAM_SETA - 2.0, topo - 2.0), p.pt(x0.max(x1) + TAM_SETA + 2.0, fundo + 2.0));
        if !visivel.intersects(area) {
            continue;
        }
        let cor = if s.ja { com_alfa(VERDE, 0.8) } else { com_alfa(MEDIO, 0.6) };
        let traco = Stroke::new(1.3, cor);
        let mut faixas = cabeca.clone();
        faixas.extend(s.cobrem.iter().map(|k| (linhas[*k].2 - SOBE_TEXTO, linhas[*k].2 + DESCE_TEXTO)));
        de_pe(p, x0, y0, pista, &mut faixas, traco, s.ja);
        traco_de(p, [(x0, pista), (x1, pista)], traco, s.ja);
        traco_de(p, [(x1, pista), (x1, y1)], traco, true);
        p.fio(x1 - TAM_SETA, y1 + TAM_SETA, x1, y1, cor, 1.5);
        p.fio(x1 + TAM_SETA, y1 + TAM_SETA, x1, y1, cor, 1.5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn so_fica_a_esteira_que_nenhum_outro_caminho_cobre() {
        // a terceira espera a primeira e a segunda; a segunda já espera a primeira
        let deps = vec![vec![], vec![0], vec![0, 1]];
        assert_eq!(so_as_diretas(&deps), vec![vec![], vec![0], vec![1]]);
    }

    #[test]
    fn um_ciclo_nao_trava() {
        let deps = vec![vec![1], vec![0]];
        assert!(alcanca(&deps, 0, 1));
        assert!(!alcanca(&deps, 0, 5) || deps.len() > 5);
    }
}
