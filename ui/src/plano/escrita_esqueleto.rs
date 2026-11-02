//! A marca de cada ação na régua enquanto o agente escreve: a que aguarda vira esqueleto com o brilho passando, a que está sendo escrita ganha a faixa azul, o cursor e o ponto que pisca,
//! e a que acabou de ser preenchida entra com um clarão verde e o «Plano pronto» deslizando. Quem chama é a folha, linha a linha, com o pincel já recortado na régua.

use super::super::agentes_movimento as mov;
use super::super::agentes_pecas as pecas;
use super::super::escala::{Escala, LINHA};
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, EstadoEscrita, Folha};
use super::marca;
use crate::i18n::tr;
use crate::tema::{AZUL, MEDIO, PAPEL, TINTA, VERDE};
use egui::{Context, Id};
use std::hash::{Hash, Hasher};

/// Quanto tempo dura o clarão de quem acabou de ser preenchido.
const CLARAO_S: f64 = 1.1;
const ENTRADA_S: f64 = 0.5;

/// Há quantos segundos esta ação mudou de estado (muito, se a janela só a viu assim). Lembra o estado de um quadro para o outro.
fn idade(ctx: &Context, a: &Acao) -> f64 {
    let id = Id::unique(("escrita-estado", &a.id));
    let agora = mov::agora(ctx);
    let atual = a.escrita as u8;
    let antes: Option<(u8, f64)> = ctx.data(|d| d.get_temp(id));
    let desde = match antes {
        Some((estado, desde)) if estado == atual => desde,
        Some(_) => agora,
        None => f64::NEG_INFINITY,
    };
    ctx.data_mut(|d| d.insert_temp(id, (atual, desde)));
    if mov::ligado() {
        agora - desde
    } else {
        f64::INFINITY
    }
}

/// Um número de 0 a 1 que só depende da ação: cada esqueleto brilha numa hora diferente.
fn desvio(a: &Acao) -> f32 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    a.id.hash(&mut h);
    (h.finish() % 1000) as f32 / 1000.0
}

/// Aguardando plano: a caixa tracejada da folha ganha o brilho que passa e o texto respira.
fn aguardando(regua: &Pincel, e: &Escala, a: &Acao, y: f32) {
    let x = e.x(e.hoje) + 12.0;
    let caixa = regua.caixa(x, y - 8.0, 96.0, 16.0);
    regua.p.rect_filled(caixa, 0, MEDIO.gamma_multiply(0.07));
    mov::varredura(regua.p, caixa, (mov::fase(regua.p.ctx(), 1.9) + desvio(a)) % 1.0, TINTA.gamma_multiply(0.2));
    let letra = Letra::nova(10.0, MEDIO);
    let largura = regua.largura(tr("Aguardando plano"), &letra);
    let apaga = 0.45 * mov::onda((mov::fase(regua.p.ctx(), 2.6) + desvio(a)) % 1.0);
    regua.ret(x + 104.0, y - 6.0, largura + 4.0, 14.0, Some(PAPEL.gamma_multiply(apaga)), None);
}

/// Escrevendo: a faixa azul na linha, o fio que corre embaixo e o rótulo com o ponto e o cursor.
fn escrevendo(regua: &Pincel, e: &Escala, texto: &str, y: f32) {
    let ctx = regua.p.ctx();
    let respira = mov::onda(mov::fase(ctx, 2.2));
    regua.ret(e.x0, y - LINHA / 2.0, e.x1 - e.x0, LINHA, Some(AZUL.gamma_multiply(0.06 + 0.05 * respira)), None);
    let fio = regua.caixa(e.x0, y + LINHA / 2.0 - 3.0, e.x1 - e.x0, 2.0);
    mov::varredura(regua.p, fio, mov::fase(ctx, 1.4), AZUL);
    let letra = Letra::nova(10.0, AZUL).fim();
    let fim = e.x1 - 14.0;
    let largura = regua.largura(texto, &letra);
    regua.ret(fim - largura - 22.0, y - 8.0, largura + 36.0, 16.0, Some(PAPEL), None);
    pecas::ponto(regua, fim - largura - 12.0, y, AZUL, true);
    regua.texto(texto, fim, y + 4.0, letra);
    let aceso = (mov::agora(ctx) % 1.0 < 0.5) || !mov::ligado();
    mov::vivo(ctx);
    if aceso {
        regua.ret(fim + 3.0, y - 5.0, 2.0, 11.0, Some(AZUL), None);
    }
}

/// Plano pronto: o clarão verde varre a linha, o risco de confere se desenha e o texto chega deslizando da direita.
fn pronto(regua: &Pincel, e: &Escala, texto: &str, y: f32, idade: f64) {
    let ctx = regua.p.ctx();
    let t = (idade / ENTRADA_S).clamp(0.0, 1.0) as f32;
    let chegada = egui::emath::easing::cubic_out(t);
    if idade < CLARAO_S {
        let faixa = regua.caixa(e.x0, y - LINHA / 2.0, e.x1 - e.x0, LINHA);
        mov::varredura(regua.p, faixa, (idade / 0.9).min(1.0) as f32, VERDE.gamma_multiply(0.3 * (1.0 - (idade / CLARAO_S) as f32)));
        ctx.request_repaint();
    }
    let letra = Letra::nova(10.0, VERDE.gamma_multiply(chegada)).fim();
    let fim = e.x1 - 8.0 + (1.0 - chegada) * 16.0;
    let largura = regua.largura(texto, &letra);
    regua.ret(fim - largura - 22.0, y - 8.0, largura + 28.0, 16.0, Some(PAPEL), None);
    let (x, riscado) = (fim - largura - 15.0, chegada);
    regua.fio(x, y, x + 3.0 * riscado, y + 3.0 * riscado, VERDE, 1.4);
    if riscado > 0.5 {
        let r = (riscado - 0.5) * 2.0;
        regua.fio(x + 3.0, y + 3.0, x + 3.0 + 6.0 * r, y + 3.0 - 7.0 * r, VERDE, 1.4);
    }
    regua.texto(texto, fim, y + 4.0, letra);
}

/// A marca da linha da ação, na ponta direita da régua. A folha passa o pincel da régua. Sem escrita, nada.
pub fn etiqueta(regua: &Pincel, e: &Escala, f: &Folha, a: &Acao, y: f32) {
    if f.escrita.is_none() {
        return;
    }
    let tempo = idade(regua.p.ctx(), a);
    match a.escrita {
        EstadoEscrita::Aguardando => aguardando(regua, e, a, y),
        EstadoEscrita::Escrevendo => escrevendo(regua, e, tr("Escrevendo…"), y),
        EstadoEscrita::Pronta => {
            if let Some((texto, _)) = marca(a) {
                pronto(regua, e, texto, y, tempo);
            }
        }
    }
}
