//! As chamadas da vista expandida: de cada camada sai um fio até o lado da pilha, entre ela e a coluna, e ali, direto na planta e sem caixa, o que há nela e como está, na cor do estado. Clicar numa chamada escolhe a primeira peça da camada.

use super::super::super::acao_movimento::com_alfa;
use super::super::super::linha::{Letra, Pincel};
use super::super::super::modelo::{Acao, Folha, Plano};
use super::super::blocos::Bloco;
use super::super::figura::{Marca, LADO};
use super::super::obra_figura::ALTO;
use super::super::obra_formas::Forma;
use super::super::obra_pecas::PecaObra;
use super::super::{obra_fora, obra_mergulho as mergulho, obra_rotulos};
use super::{Nivel, Pilha};
use crate::anatomia::{Camera, V3};
use crate::i18n::{tr, tr_com};
use crate::tema::{AMBAR, AZUL, MEDIO, TINTA, VERDE};
use egui::{Color32, CursorIcon, Pos2, Rect, Sense, Ui};

const ALTO_CHAMADA: f32 = 34.0;
/// Da borda da pilha até onde o texto começa.
const VAO_FIO: f32 = 28.0;
/// O menor lugar do texto: com menos que isto à direita, a chamada chega mais perto da pilha.
const TEXTO_MIN: f32 = 96.0;

/// O que cada camada diz: o título e o estado.
fn chamada(f: &Folha, q: &Plano, b: &Bloco, pecas: &[PecaObra], n: Nivel) -> (String, String, Color32) {
    let do_tipo: Vec<&PecaObra> = n.peca().map_or(Vec::new(), |t| pecas.iter().filter(|p| p.tipo == t).collect());
    let total = do_tipo.len();
    let esperando = do_tipo.iter().filter(|p| p.marca == Marca::Espera).count();
    let conta = |pede: &'static str, tudo: &'static str| if esperando > 0 { (tr_com(pede, &[("n", &esperando.to_string())]), AZUL) } else { (tr(tudo).to_string(), VERDE) };
    let titulo = |t: &'static str| tr_com(t, &[("n", &total.to_string())]);
    match n {
        Nivel::Obra => {
            let (s, c) = obra_rotulos::fala(q, |r| obra_fora::pronto(f, r));
            (Forma::do_plano(q).nome().to_string(), s, c)
        }
        Nivel::Base => {
            let lista = q.depende_de.join(", ");
            if lista.is_empty() {
                (tr("Base").to_string(), tr("começa aqui").to_string(), MEDIO)
            } else {
                let falta = q.depende_de.iter().any(|d| !obra_fora::pronto(f, d));
                (tr("Base").to_string(), tr_com("depende de {lista}", &[("lista", &lista)]), if falta { AMBAR } else { MEDIO })
            }
        }
        Nivel::Checkpoints => {
            let feitos = b.checkpoints.iter().filter(|k| k.feito).count();
            (titulo("Checkpoints · {n}"), tr_com("{n} de {total} feitos", &[("n", &feitos.to_string()), ("total", &total.to_string())]), if feitos == total { VERDE } else { MEDIO })
        }
        Nivel::Perguntas => {
            let (s, c) = conta("{n} sem resposta", "todas respondidas");
            (titulo("Perguntas · {n}"), s, c)
        }
        Nivel::Decisoes => {
            let (s, c) = conta("{n} a decidir", "todas decididas");
            (titulo("Decisões · {n}"), s, c)
        }
        Nivel::Telas => {
            let (s, c) = conta("{n} para aprovar", if do_tipo.iter().any(|p| p.marca == Marca::Ok) { "todas aprovadas" } else { "só para ver" });
            (titulo("Telas · {n}"), s, if esperando == 0 && !do_tipo.iter().any(|p| p.marca == Marca::Ok) { MEDIO } else { c })
        }
    }
}

/// Onde o fio sai da camada na altura `z`: o canto do lote mais à direita na tela, seja qual for o giro.
fn canto(cam: &Camera, [x, y, z]: V3) -> Pos2 {
    [(0.0, 0.0), (LADO, 0.0), (LADO, LADO), (0.0, LADO)].into_iter().map(|(u, v)| cam.ponto([x + u, y + v, z])).fold(Pos2::new(f32::MIN, 0.0), |a, b| if b.x > a.x { b } else { a })
}

/// As chamadas da pilha `p`, com `k` de presença. Nunca passam de `livre`, o que a coluna deixa; `tocavel` deixa clicar nelas.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, a: &Acao, f: &Folha, b: &Bloco, p: &Pilha, cam: &Camera, livre: Rect, k: f32, tocavel: bool) {
    if k <= 0.01 {
        return;
    }
    let ctx = ui.ctx().clone();
    let [x, y, z0] = p.lote;
    let pes: Vec<Pos2> = p.niveis.iter().zip(p.zs).map(|(n, z)| canto(cam, [x, y, z0 + z + if *n == Nivel::Obra { ALTO * 0.5 } else { 0.0 }])).collect();
    let direita = pes.iter().map(|q| q.x).fold(f32::MIN, f32::max);
    let x_texto = (direita + VAO_FIO).min(livre.right() - TEXTO_MIN) - c.o.x;
    let cabe = livre.right() - c.o.x - x_texto - 8.0;
    let recorte = c.p.with_clip_rect(livre.intersect(c.p.clip_rect()));
    let pintor = com_alfa(&recorte, k);
    let pc = Pincel { p: &pintor, o: c.o };
    let (forte, fio, topo) = (Letra::nova(11.0, TINTA).forte(), TINTA.gamma_multiply(0.55), livre.top() - c.o.y + 4.0);
    // de baixo para cima: cada chamada fica pelo menos uma altura acima da anterior
    let mut ultimo = f32::MAX;
    for (i, (n, pe)) in p.niveis.iter().zip(&pes).enumerate() {
        let (px, py) = (pe.x - c.o.x, pe.y - c.o.y);
        let ly = (py - 8.0).min(ultimo - ALTO_CHAMADA).max(topo);
        ultimo = ly;
        let (titulo, frase, cor) = chamada(f, p.q, b, p.pecas, *n);
        pc.fio(px, py, x_texto - 10.0, ly + 8.0, fio, 1.0);
        pc.fio(x_texto - 10.0, ly + 8.0, x_texto, ly + 8.0, fio, 1.0);
        let letra = Letra::nova(10.5, cor);
        let w_t = pc.texto(&pc.cortado(&titulo, cabe, &forte).0, x_texto + 4.0, ly + 8.0, forte);
        let w_f = pc.texto(&pc.cortado(&frase, cabe, &letra).0, x_texto + 4.0, ly + 23.0, letra);
        let Some(tipo) = n.peca().filter(|_| tocavel) else { continue };
        let toca = ui.interact(pc.caixa(x_texto, ly - 4.0, w_t.max(w_f) + 8.0, 32.0), ui.scope_id().with(("plano-parte-chamada", &a.referencia, &p.q.referencia, i)), Sense::click());
        if toca.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            if let Some(q) = p.pecas.iter().find(|q| q.tipo == tipo) {
                mergulho::escolher(&ctx, a, &q.chave);
            }
        }
    }
}
