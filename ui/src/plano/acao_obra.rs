//! O palco: a única tela da ação. A obra ocupa a prancha inteira e fala sozinha, sem parágrafo: o que está de pé, o que falta, quem espera quem, onde a pessoa é chamada. O que antes era aba e mesa agora é figura e clique em cima dela.
//! Embaixo, a régua da ordem de construção (o tempo, a quarta dimensão). Clicar numa construção aproxima o olhar, e o lado direito vira a ficha daquele plano; o título do painel de bordo abre a mesa da ação.
//! Mais um clique, ou mais roda, abre a vista expandida aqui mesmo: o palco não muda de tamanho nem de câmera, a construção se separa em camadas onde está, as outras se apagam (`acao_obra_explosao.rs`), e a coluna da direita é a mesma ficha, só mais completa. A página não pula entre os níveis.
//! Em janela larga a ficha é uma coluna à direita, e a mesa da ação abre bem mais larga, em duas colunas, sempre deixando à obra uma faixa à esquerda. O palco cede o lugar de cada uma no seu próprio ritmo, e trocar de uma para a outra anda em vez de pular.

use super::super::acao_movimento::{com_alfa, mistura};
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Folha, Plano};
use super::super::revisao::Alvo;
use super::blocos::{Bloco, Montagem};
use super::obra_figura::{self, Clique, Obra};
use super::figura::Peca;
use super::obra_pecas::{pecas_de, PecaObra};
use super::obra_sumico::Sumico;
use super::{explodida, mesa_acao, na_tela, obra_bordo, obra_explosao as explosao, obra_mergulho as mergulho, obra_rotulos, obra_tempo, Saida};
use crate::i18n::tr;
use crate::tema::MEDIO;
use egui::{vec2, Rect, Ui};

const LARGURA_DUAS: f32 = 900.0;
/// A ficha de um plano: uma coluna à direita do palco.
const FRACAO_FICHA: f32 = 0.38;
const FICHA_MAX: f32 = 620.0;
const DESLIZE_FICHA: f32 = 40.0;
/// A mesa da ação: larga, para caber em duas colunas, sem tirar da obra a faixa da esquerda.
const FRACAO_MESA: f32 = 0.64;
const MESA_MAX: f32 = 1000.0;
const OBRA_MIN: f32 = 340.0;
/// A mesa entra pela borda do palco e se abre até o lugar: anda esta fração da largura dela.
const DESLIZE_MESA: f32 = 0.35;
/// O tempo que o palco leva para ceder o lugar de um painel, ou passá-lo ao outro.
const SEG_CEDE: f32 = 0.5;
const VAO_COLUNAS: f32 = 28.0;
const VAO: f32 = 14.0;
const LARGURA_NOMES: f32 = 78.0;
/// A altura do palco cresce com a largura, dentro destes limites.
const ALTO_PALCO: (f32, f32) = (480.0, 780.0);
const FRACAO_ALTO: f32 = 0.44;
/// A faixa acima do palco, com o interruptor dos nomes e o próximo passo: nada disso cobre a obra.
const TOPO: f32 = 34.0;
const FOLGA_JANELA: f32 = 70.0;

fn so_planos<'m, 'a>(m: &'m Montagem<'a>) -> Vec<(&'m Plano, &'m Bloco<'a>)> {
    m.blocos.iter().filter_map(|b| b.plano.as_ref().map(|q| (q, b))).collect()
}

/// Nunca mais alto que a janela: dentro de uma tela, ela e a barra precisam caber juntas.
fn alto_palco(ctx: &egui::Context, w: f32) -> f32 {
    (w * FRACAO_ALTO).clamp(ALTO_PALCO.0, ALTO_PALCO.1).min((ctx.content_rect().height() - FOLGA_JANELA).max(ALTO_PALCO.0))
}

/// O que sobra do palco com a ficha aberta: é nisso que o mergulho enquadra.
pub fn largura_livre(w: f32) -> f32 {
    if w >= LARGURA_DUAS { w - largura_ficha(w) - VAO_COLUNAS } else { w }
}

fn largura_ficha(w: f32) -> f32 {
    if w >= LARGURA_DUAS { (w * FRACAO_FICHA).min(FICHA_MAX) } else { w }
}

/// A mesa da ação em janela larga: nunca mais estreita que a ficha, e a obra fica com pelo menos `OBRA_MIN` à esquerda.
fn largura_mesa_acao(w: f32) -> f32 {
    if w >= LARGURA_DUAS { (w * FRACAO_MESA).min(MESA_MAX).min(w - VAO_COLUNAS - OBRA_MIN).max(largura_ficha(w)) } else { w }
}

/// Quanto o palco cede à direita, em janela larga: cada painel com a sua mistura, então a troca da ficha pela mesa anda de uma largura à outra. Dentro de uma tela, nada.
fn quanto_cede(ctx: &egui::Context, a: &Acao, w: f32, com_ficha: bool) -> f32 {
    let fora_da_tela = na_tela::aberta(ctx, a).is_none();
    let k_ficha = mistura(ctx, ("plano-obra-palco-cede-ficha", &a.referencia), com_ficha && fora_da_tela, SEG_CEDE);
    let k_mesa = mistura(ctx, ("plano-obra-palco-cede-mesa", &a.referencia), mesa_acao::aberta(ctx, a) && fora_da_tela, SEG_CEDE);
    let (w_ficha, w_mesa) = (largura_ficha(w), largura_mesa_acao(w));
    ((w_ficha + VAO_COLUNAS) * k_ficha + (w_mesa + VAO_COLUNAS) * k_mesa).min(w_mesa + VAO_COLUNAS)
}

/// O lugar de um painel de largura `lw`, encostado à direita do palco; `fora` é o quanto ele ainda está para lá da borda.
fn lugar(c: &Pincel, x: f32, y: f32, w: f32, lw: f32, alto: f32, fora: f32) -> Rect {
    Rect::from_min_size(c.pt(x + w - lw + fora, y), vec2(lw, alto))
}

fn alto_base(ctx: &egui::Context, w: f32) -> f32 {
    TOPO + alto_palco(ctx, w) + VAO + obra_tempo::ALTO + VAO
}

/// O plano em que se está mergulhado, ou o último, enquanto a câmera sobe.
fn da_mesa<'x, 'm, 'a>(ctx: &egui::Context, a: &Acao, lista: &'x [(&'m Plano, &'m Bloco<'a>)]) -> Option<(&'m Plano, &'m Bloco<'a>)> {
    let r = mergulho::plano(ctx, a).or_else(|| mergulho::ultimo(ctx, a))?;
    lista.iter().find(|(q, _)| q.referencia == r).copied()
}

/// Em janela estreita a ficha fica embaixo do palco e cresce com o mergulho; a mesa vem logo abaixo dela, então as duas nunca se sobrepõem na troca.
fn alto_ficha_embaixo(p: &egui::Painter, f: &Folha, a: &Acao, lista: &[(&Plano, &Bloco)], w: f32) -> f32 {
    let t = mergulho::quanto(p.ctx(), a);
    da_mesa(p.ctx(), a, lista).filter(|_| t > 0.0).map_or(0.0, |(q, b)| mergulho::altura(p, f, a, q, b, w) * t)
}

pub fn altura(p: &egui::Painter, f: &Folha, a: &Acao, m: &Montagem, w: f32) -> f32 {
    let lista = so_planos(m);
    if lista.is_empty() {
        return 40.0 + mesa_acao::altura(p, f, a, m, w, w >= LARGURA_DUAS);
    }
    let ctx = p.ctx();
    // em janela larga a mesa flutua sobre o palco: nada se mexe na página
    if w >= LARGURA_DUAS {
        return alto_base(ctx, w);
    }
    let ficha = alto_ficha_embaixo(p, f, a, &lista, w);
    let mesa = mesa_acao::altura(p, f, a, m, w, false) * mesa_acao::quanto(ctx, a);
    alto_base(ctx, w) + ficha + mesa + VAO
}

#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, m: &Montagem, x: f32, y: f32, w: f32) -> Option<Saida> {
    let ctx = ui.ctx().clone();
    let lista = so_planos(m);
    mergulho::assentar(ui, c, a, x, y, w);
    if lista.is_empty() {
        c.texto(tr("Esta ação ainda não tem planos"), x, y + 14.0, Letra::nova(12.0, MEDIO));
        return mesa_acao::desenhar(ui, c, alvo, f, a, m, x, y + 40.0, w, w >= LARGURA_DUAS);
    }
    let planos: Vec<&Plano> = lista.iter().map(|(q, _)| *q).collect();
    let y_faixa = y;
    mergulho::teclado(&ctx, a, &planos);
    // antes de ler o mergulho: a vista expandida e ele falam sempre do mesmo plano
    explosao::acertar(&ctx, a, &planos);
    let t_tela = na_tela::quanto(&ctx, a);
    let y = y + TOPO;
    let o = Obra::de(&planos);
    let dentro = mergulho::plano(&ctx, a).and_then(|r| planos.iter().position(|q| q.referencia == r));
    let (t, t_mesa) = (mergulho::quanto(&ctx, a), mesa_acao::quanto(&ctx, a));
    let t_lado = t.max(t_mesa);
    let duas = w >= LARGURA_DUAS;
    let (w_ficha, w_mesa) = (largura_ficha(w), largura_mesa_acao(w));
    let alto = alto_palco(&ctx, w);
    // o palco nunca muda de tamanho: a ficha e a mesa flutuam por cima, e o meio da vista anda para o que ficou livre
    let area = Rect::from_min_size(c.pt(x, y), vec2(w, alto));
    let cede = if duas { quanto_cede(&ctx, a, w, dentro.is_some()) } else { 0.0 };
    let livre = Rect::from_min_size(area.min, vec2(w - cede, alto));
    let tau = obra_tempo::instante(&ctx, a);
    let bloco_dentro = dentro.map(|i| lista[i].1);
    let peca = bloco_dentro.and_then(|b| mergulho::peca(&ctx, a, b)).map(|i| i.chave());
    let gesto = obra_figura::desenhar(ui, c, alvo, f, a, &lista, &o, area, livre, dentro, t, tau, peca.as_deref());
    // cada clique desce um nível: a construção chega perto, depois a vista expandida no mesmo palco, e só nela a tela
    match gesto.clique {
        Some(Clique::Modulo(i)) if dentro == Some(i) && t > 0.9 => explodida::abrir(&ctx, a, &planos[i].referencia),
        Some(Clique::Modulo(i)) | Some(Clique::Duto(i)) => mergulho::entrar(&ctx, a, &planos[i].referencia),
        Some(Clique::Peca(i, chave)) => {
            if dentro == Some(i) && peca.as_deref() == Some(chave.as_str()) {
                explodida::abrir(&ctx, a, &planos[i].referencia);
            } else {
                if dentro != Some(i) {
                    mergulho::entrar(&ctx, a, &planos[i].referencia);
                }
                mergulho::escolher(&ctx, a, &chave);
            }
        }
        None => {}
    }
    if let Some(i) = gesto.expandir {
        if dentro != Some(i) {
            mergulho::entrar(&ctx, a, &planos[i].referencia);
        }
        explodida::abrir(&ctx, a, &planos[i].referencia);
    }
    let dentro_da_tela = t_tela >= 0.5;
    // entrando na tela, a página anda até o palco caber inteiro, com a barra
    if t_tela < 1.0 && na_tela::aberta(&ctx, a).is_some() {
        ui.scroll_to_rect(area, Some(egui::Align::Center));
    }
    let mut saida = gesto.fora.and_then(|(r, de)| mergulho::ir_ao_plano(&ctx, f, a, &r, de, false));
    if !dentro_da_tela {
        obra_rotulos::botao_nomes(ui, c, x, y_faixa);
        let pede = obra_bordo::proximo(ui, c, f, a, &lista, x + LARGURA_NOMES, y_faixa, w - LARGURA_NOMES - 8.0);
        saida = pede.and_then(|(r, de)| mergulho::ir_ao_plano(&ctx, f, a, &r, de, false)).or(saida);
    }
    if t_lado < 0.5 && !dentro_da_tela {
        let pintor = com_alfa(c.p, 1.0 - t_lado * 2.0);
        obra_bordo::desenhar(ui, &Pincel { p: &pintor, o: c.o }, alvo, f, a, m, &planos, x + w, y + alto, w);
    }
    let falta: Vec<usize> = lista.iter().map(|(_, b)| b.a_decidir().0).collect();
    obra_tempo::desenhar(ui, c, a, &o, &planos, &falta, x, y + alto + VAO, w);
    if t_tela > 0.0 {
        if let Some((_, b)) = da_mesa(&ctx, a, &lista) {
            let pecas = pecas_de(b);
            let irmas: Vec<&PecaObra> = pecas.iter().filter(|p| p.tipo == Peca::Tela && p.lugar.is_some()).collect();
            if let Some(p) = na_tela::olhando(&ctx, a).and_then(|ch| irmas.iter().find(|p| p.chave == ch)) {
                na_tela::barra(ui, c, alvo, f, a, &irmas, p, area, t_tela);
            }
        }
    }
    // a ficha abre e fecha junto com o mergulho, e sai de cena dentro de uma tela
    let t_ficha = if duas { t * (1.0 - t_tela) } else { t };
    if let Some((q, b)) = da_mesa(&ctx, a, &lista).filter(|_| t_ficha > 0.01) {
        let pede = if duas {
            let ficha = lugar(c, x, y, w, w_ficha, alto, (1.0 - t_ficha) * DESLIZE_FICHA);
            Sumico::gravar(&ctx, a, ficha, t_ficha);
            mergulho::flutuante(ui, c, &q.referencia, area, ficha, t_ficha, |p, w| mergulho::altura(p, f, a, q, b, w), |ui, pc, w| mergulho::painel(ui, pc, alvo, f, a, q, b, 0.0, 0.0, w))
        } else {
            let pintor = com_alfa(c.p, t);
            let opaco = ui.opacity();
            ui.set_opacity(opaco * t);
            let pede = mergulho::painel(ui, &Pincel { p: &pintor, o: c.o }, alvo, f, a, q, b, x, y_faixa + alto_base(&ctx, w), w);
            ui.set_opacity(opaco);
            pede
        };
        saida = pede.or(saida);
    }
    if t_mesa > 0.0 {
        let pede = if duas {
            // entra pela borda direita do palco, já na largura final (nada se rearruma no caminho), e o recorte do palco faz ela se abrir
            let mesa = lugar(c, x, y, w, w_mesa, alto, (1.0 - t_mesa) * w_mesa * DESLIZE_MESA);
            Sumico::gravar(&ctx, a, mesa, t_mesa);
            mergulho::flutuante(ui, c, "mesa-da-acao", area, mesa, t_mesa, |p, w| mesa_acao::altura(p, f, a, m, w, true), |ui, pc, w| mesa_acao::desenhar(ui, pc, alvo, f, a, m, 0.0, 0.0, w, true))
        } else {
            let pintor = com_alfa(c.p, t_mesa);
            let opaco = ui.opacity();
            ui.set_opacity(opaco * t_mesa);
            let y_mesa = y_faixa + alto_base(&ctx, w) + alto_ficha_embaixo(c.p, f, a, &lista, w);
            let pede = mesa_acao::desenhar(ui, &Pincel { p: &pintor, o: c.o }, alvo, f, a, m, x, y_mesa, w, false);
            ui.set_opacity(opaco);
            pede
        };
        saida = pede.or(saida);
    }
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    const LARGAS: [f32; 6] = [900.0, 1000.0, 1200.0, 1440.0, 1800.0, 2600.0];

    #[test]
    fn a_mesa_sempre_deixa_a_faixa_da_obra() {
        for w in LARGAS {
            assert!(w - largura_mesa_acao(w) - VAO_COLUNAS >= OBRA_MIN - 0.01, "w = {w}");
        }
    }

    #[test]
    fn a_mesa_e_mais_larga_que_a_ficha_e_tem_teto() {
        for w in LARGAS {
            assert!(largura_mesa_acao(w) >= largura_ficha(w), "w = {w}");
            assert!(largura_mesa_acao(w) <= MESA_MAX, "w = {w}");
        }
        assert_eq!(largura_mesa_acao(2600.0), MESA_MAX);
    }

    #[test]
    fn a_ficha_continua_como_era() {
        assert!((largura_ficha(1000.0) - 380.0).abs() < 0.01);
        assert_eq!(largura_ficha(2000.0), FICHA_MAX);
        assert!((largura_livre(1000.0) - (1000.0 - 380.0 - VAO_COLUNAS)).abs() < 0.01);
    }

    #[test]
    fn janela_estreita_usa_a_largura_toda() {
        assert_eq!(largura_mesa_acao(800.0), 800.0);
        assert_eq!(largura_livre(800.0), 800.0);
    }
}
