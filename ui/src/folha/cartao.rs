//! O cartão de um blueprint na lista (`capa` em web/js/render.js): a arte «em construção», o nome, a descrição, o andamento em texto, a barra
//! com uma célula por ação, a linha de cota e os checkpoints em losango.

use super::cena::{Ancora, Cena, Fio};
use super::pagina::corta;
use crate::dominio::{self, Acao, Resumo, Situacao};
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com};
use crate::tema::{rgba, BAIXO, MEDIO, PAPEL, TINTA};
use egui::Color32;

/// As células da barra: se cabem (9 px cada), uma por ação; senão, um bloco por situação (`capaCelulas`).
struct Celula {
    status: Situacao,
    pct: f32,
    peso: f32,
}

fn celulas(fs: &[&Acao], largura: f32) -> Vec<Celula> {
    if fs.len() as f32 * 9.0 <= largura {
        return fs.iter().map(|f| Celula { status: f.status, pct: f.pct, peso: f.itens.1.max(1) as f32 }).collect();
    }
    let mut blocos: Vec<(Situacao, f32, f32)> = Vec::new();
    for f in fs {
        let peso = f.itens.1.max(1) as f32;
        match blocos.last_mut() {
            Some(b) if b.0 == f.status => {
                b.1 += peso;
                b.2 += f.pct * peso;
            }
            _ => blocos.push((f.status, peso, f.pct * peso)),
        }
    }
    blocos.into_iter().map(|(status, peso, soma)| Celula { status, pct: soma / peso, peso }).collect()
}

/// Os checkpoints que se tocariam (menos de 8 px do primeiro do grupo) viram um losango só, no meio deles (`capaMarcos`).
fn marcos(mut m: Vec<(f32, bool)>) -> Vec<(f32, bool)> {
    m.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut grupos: Vec<(f32, usize, usize, f32)> = Vec::new();
    for (x, feito) in m {
        match grupos.last_mut() {
            Some(g) if x - g.0 < 8.0 => {
                g.1 += 1;
                g.2 += usize::from(feito);
                g.3 += x;
            }
            _ => grupos.push((x, 1, usize::from(feito), x)),
        }
    }
    grupos.into_iter().map(|(_, n, feitos, soma)| (soma / n as f32, feitos == n)).collect()
}

/// Quantos pixels a descrição e o nome ocupam (`corta` por tamanho de letra), como na página.
pub struct Medidas {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Desenha o cartão `p` em `m`, com o resumo `r` das ações dele, que a lista já calculou (`dominio::resumo_json`). A lista só chama
/// para o cartão que aparece na janela. `realce`: o mouse está em cima (`.alvo:hover .moldura`). Devolve a largura da arte (para o chamador pintar a imagem).
pub fn desenhar(c: &Cena, m: &Medidas, p: &serde_json::Value, r: &Resumo, realce: bool, achou: Option<(&str, &[String])>) -> ArteCartao {
    let (tx, ty, tw, th) = (m.x, m.y, m.w, m.h);
    let fs = dominio::acoes_de(p);
    // `dominio::concluido`: tem ações e todas concluídas
    let ok = !fs.is_empty() && r.concl == fs.len();
    let nome_json = p.get("nome").and_then(|v| v.as_str()).unwrap_or("");
    let descricao_json = p.get("descricao").and_then(|v| v.as_str()).unwrap_or("");
    let (fundo, traco, largura) = if realce { (rgba(0x17539f, 0xff), TINTA, 1.6) } else if ok { (rgba(0x082b5e, 0xb3), BAIXO, 1.0) } else { (rgba(0x0c3a78, 0xb3), MEDIO, 1.0) };
    c.retangulo(tx, ty, tw, th, Some(fundo), Some((traco, largura)));
    let (art_w, art_x, art_y, art_h) = ((tw * 0.25).round().clamp(76.0, 116.0), tx + 12.0, ty + 26.0, th - 52.0);
    let (texto_x, texto_w) = (art_x + art_w + 14.0, tx + tw - 16.0 - (art_x + art_w + 14.0));
    let nome = corta(if nome_json.is_empty() { &tr("SEM NOME") } else { nome_json }, ((texto_w / 10.2).floor() as usize).max(8));
    let descricao = corta(descricao_json, ((texto_w / 7.5).floor() as usize).max(12));
    let faltam = dominio::faltam(&fs);
    let n = p.get("acoes").and_then(|v| v.as_array()).map_or(0, Vec::len);
    let info = if n == 0 {
        tr("SEM AÇÕES AINDA").to_string()
    } else if ok {
        tr_com("✓ CONCLUÍDO · {n} AÇÕES", &[("n", &n.to_string())])
    } else if r.futuro {
        tr("PLANEJADO").to_string()
    } else {
        tr_com("{concl}/{total} AÇÕES · {pct}% · FALTAM {faltam} ITENS", &[("concl", &r.concl.to_string()), ("total", &n.to_string()), ("pct", &((r.geral * 100.0).round() as i32).to_string()), ("faltam", &faltam.to_string())])
    };
    // pesquisando: o trecho achado no nome e na descrição ganha o realce e, no lugar do andamento, vai onde a consulta casou
    if let Some((_, termos)) = achou {
        c.realce(&nome, texto_x, ty + 44.0, 15.0, Peso::Negrito, 0.06, true, &crate::pesquisa::trechos(&nome, termos));
        c.realce(&descricao, texto_x, ty + 70.0, 11.0, Peso::Normal, 0.06, true, &crate::pesquisa::trechos(&descricao, termos));
    }
    c.texto(&nome, texto_x, ty + 44.0, 15.0, Peso::Negrito, TINTA, Ancora::Inicio, true);
    c.texto(&descricao, texto_x, ty + 70.0, 11.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    match achou {
        Some((onde, _)) => c.texto(&corta(onde, (texto_w / 7.0).floor() as usize), texto_x, ty + 95.0, 10.0, Peso::Normal, egui::Color32::WHITE, Ancora::Inicio, false),
        None => c.texto(&corta(&info, (texto_w / 7.0).floor() as usize), texto_x, ty + 95.0, 10.0, Peso::Normal, MEDIO, Ancora::Inicio, false),
    };
    if !fs.is_empty() {
        barra(c, &fs, texto_x, tx + tw - 16.0, ty);
    }
    ArteCartao { x: art_x, y: art_y, w: art_w, h: art_h, progresso: r.geral }
}

pub struct ArteCartao {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub progresso: f32,
}

/// A barra do blueprint inteiro e, embaixo, a linha de cota com os checkpoints, no fim do trecho de cada ação.
fn barra(c: &Cena, fs: &[Acao], x0: f32, x1: f32, ty: f32) {
    let (yb, yc) = (ty + 106.0, ty + 130.0);
    let mut ordem: Vec<&Acao> = fs.iter().collect();
    ordem.sort_by_key(|f| (f.status as u8, f.idx));
    let cel = celulas(&ordem, x1 - x0);
    let vao = |i: usize| if i > 0 && cel[i].status != cel[i - 1].status { 7.0 } else { 2.0 };
    let total: f32 = cel.iter().map(|f| f.peso).sum();
    let util = x1 - x0 - (1..cel.len()).map(vao).sum::<f32>();
    let mut cx = x0;
    for (i, f) in cel.iter().enumerate() {
        if i > 0 {
            cx += vao(i);
            if vao(i) > 2.0 {
                c.fio(TINTA, 1.0, (cx - 3.5, yb - 3.0), (cx - 3.5, yb + 15.0));
            }
        }
        let w = util * f.peso / total;
        match f.status {
            Situacao::Concluido => {
                c.moldura(cx, yb, w, 12.0, TINTA, 1.0, false);
                c.hachura(cx + 1.0, yb + 1.0, (w - 2.0).max(0.0), 10.0);
            }
            Situacao::Andamento => {
                c.moldura(cx, yb, w, 12.0, TINTA, 1.8, false);
                if f.pct > 0.0 {
                    c.retangulo(cx + 1.0, yb + 1.0, ((w - 2.0) * f.pct).max(0.0), 10.0, Some(TINTA.gamma_multiply(0.9)), None);
                }
            }
            Situacao::Planejado => c.moldura(cx, yb, w, 12.0, MEDIO, 1.0, true),
        }
        cx += w;
    }
    c.fios(Fio::Baixo, &[(x0, yc, x1, yc), (x0, yc - 4.0, x0, yc + 4.0), (x1, yc - 4.0, x1, yc + 4.0)]);
    let trecho = (x1 - x0) / fs.len().max(1) as f32;
    let dentro = fs.iter().enumerate().flat_map(|(i, f)| {
        let (a, b, n) = (x0 + i as f32 * trecho, x0 + (i + 1) as f32 * trecho, f.checkpoints.len());
        f.checkpoints.iter().enumerate().map(move |(k, m)| (super::barra::x_marco(a, b, k, n, 8.0), m.feito))
    }).collect();
    for (x, feito) in marcos(dentro) {
        c.losango(x, yc, feito);
    }
    let _: Color32 = PAPEL;
}
