//! A obra de uma ação em miniatura, para a linha da folha: as mesmas construções, telas, esteiras e bandeiras do palco, sem texto e sem clique. De relance, a folha mostra o que o agente planejou e o que já está de pé.
//! Ao lado dela, as peças da ação, cada uma com a sua cor; o clique abre a ação e voa até a peça.
//! A miniatura não se mexe: a cena é montada uma vez por folha (`Folha::geracao`) e guardada pronta, e a cada quadro só vai para o lugar da linha. As peças e as larguras delas também.

use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Figura, Folha, Plano};
use super::blocos::Bloco;
use super::figura::feito;
use super::obra_cena;
use super::obra_figura::Obra;
use super::obra_icones::{self as icones, Icone};
use super::obra_pecas::{pecas_de, Extra, PecaObra};
use super::{blocos, obra_bordo, obra_mergulho as mergulho, obra_regra as regra};
use crate::anatomia::{imagem_na_face, Camera, Cena, Gravado, V3, AZIMUTE, ELEVACAO};
use crate::tema::{MEDIO, TINTA, VERDE};
use egui::{Context, Id, Pos2, Rect, Shape, Stroke, Ui, Vec2};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

const PAD: f32 = 2.0;
const VAO_CASAS: f32 = 10.0;

/// Um pedaço do desenho guardado, na ordem da cena: formas prontas, ou a figura de uma tela, pedida a cada quadro porque a textura pode chegar depois.
#[derive(Clone)]
enum Traco {
    Formas(Vec<Shape>),
    /// a figura, os cantos da face, a transparência e a largura da textura
    Figura(Option<Figura>, [V3; 4], f32, f32),
}

/// A miniatura de uma ação, desenhada em `Rect::from_min_size(Pos2::ZERO, tamanho)`.
struct Guardada {
    tamanho: Vec2,
    /// sem plano: só o losango tracejado
    vazia: bool,
    cam: Camera,
    tracos: Vec<Traco>,
    /// os mesmos traços no último lugar em que a linha apareceu: parada na tela, a linha só repinta
    movida: Mutex<Option<(Vec2, Arc<Vec<Traco>>)>>,
}

impl Guardada {
    fn no_lugar(&self, d: Vec2) -> Arc<Vec<Traco>> {
        let mut movida = self.movida.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((_, tracos)) = movida.as_ref().filter(|(onde, _)| *onde == d) {
            return tracos.clone();
        }
        let tracos: Arc<Vec<Traco>> = Arc::new(self.tracos.iter().map(|t| match t {
            Traco::Formas(v) => Traco::Formas(v.iter().map(|s| {
                let mut s = s.clone();
                s.translate(d);
                s
            }).collect()),
            outro => outro.clone(),
        }).collect());
        *movida = Some((d, tracos.clone()));
        tracos
    }
}

/// A fileira de peças de uma ação: as casas, a largura de cada uma e a da contagem.
struct Fileira {
    casas: Vec<regra::Casa>,
    larguras: Vec<f32>,
    w_de_pe: f32,
}

/// O que a folha guarda das linhas enquanto a folha e a escala da tela forem as mesmas; muda uma, tudo é refeito, e a memória fica só com a folha de agora.
#[derive(Clone, Default)]
struct Guardados {
    chave: (u64, u32),
    miniaturas: HashMap<String, Arc<Guardada>>,
    fileiras: HashMap<String, Arc<Fileira>>,
}

fn das_miniaturas(g: &mut Guardados) -> &mut HashMap<String, Arc<Guardada>> {
    &mut g.miniaturas
}

fn das_fileiras(g: &mut Guardados) -> &mut HashMap<String, Arc<Fileira>> {
    &mut g.fileiras
}

/// O que é desta ação no mapa `de`, feito por `fazer` na primeira vez. A conta roda fora da trava do contexto. Folha sem geração não guarda nada.
fn guardar<T>(ctx: &Context, f: &Folha, a: &Acao, ppp: f32, de: fn(&mut Guardados) -> &mut HashMap<String, Arc<T>>, fazer: impl FnOnce() -> T) -> Arc<T> {
    if f.geracao == 0 {
        return Arc::new(fazer());
    }
    let (id, chave) = (Id::unique("plano-linhas-guardadas"), (f.geracao, ppp.to_bits()));
    let com = |faz: &mut dyn FnMut(&mut HashMap<String, Arc<T>>)| {
        ctx.data_mut(|d| {
            let g = d.get_temp_mut_or_default::<Guardados>(id);
            if g.chave != chave {
                *g = Guardados { chave, ..Guardados::default() };
            }
            faz(de(g));
        });
    };
    let mut achado = None;
    com(&mut |m| achado = m.get(&a.id).cloned());
    if let Some(v) = achado {
        return v;
    }
    let v = Arc::new(fazer());
    com(&mut |m| drop(m.insert(a.id.clone(), v.clone())));
    v
}

/// A referência do ponto em que a folha está (`regra::ponto_da_folha`), contada uma vez por folha: a conta percorre a folha inteira, e cada linha pergunta.
fn com_ponto<R>(f: &Folha, faz: impl FnOnce(Option<&str>) -> R) -> R {
    thread_local! {
        static PONTO: RefCell<(u64, Option<String>)> = const { RefCell::new((0, None)) };
    }
    if f.geracao == 0 {
        return faz(regra::ponto_da_folha(f).map(|q| q.referencia.as_str()));
    }
    PONTO.with(|p| {
        if p.borrow().0 != f.geracao {
            let r = regra::ponto_da_folha(f).map(|q| q.referencia.clone());
            *p.borrow_mut() = (f.geracao, r);
        }
        faz(p.borrow().1.as_deref())
    })
}

/// A largura da textura pela largura da tela no desenho, como `obra_pecas::pintar_imagem`: nítida de perto, sem gastar de longe.
fn largura_da_figura(cam: &Camera, cantos: &[V3; 4]) -> f32 {
    let na_tela = cam.ponto(cantos[1]).distance(cam.ponto(cantos[0]));
    if na_tela <= 320.0 {
        320.0
    } else if na_tela <= 640.0 {
        640.0
    } else if na_tela <= 1280.0 {
        1280.0
    } else {
        2560.0
    }
}

fn gravar(cena: &Cena, cam: &Camera, extras: &[Extra]) -> Vec<Traco> {
    let (mut tracos, mut formas) = (Vec::new(), Vec::new());
    for g in cena.gravar(cam) {
        match g {
            Gravado::Forma(s) => formas.push(s),
            Gravado::Extra(n) => {
                if !formas.is_empty() {
                    tracos.push(Traco::Formas(std::mem::take(&mut formas)));
                }
                let Extra::Imagem(d, cantos, alfa) = &extras[n];
                tracos.push(Traco::Figura(d.figura.clone(), *cantos, *alfa, largura_da_figura(cam, cantos)));
            }
        }
    }
    if !formas.is_empty() {
        tracos.push(Traco::Formas(formas));
    }
    tracos
}

/// A cena da miniatura com o canto em zero: a câmera só anda junto com a linha, então o desenho vale em qualquer lugar.
fn montar_cena(f: &Folha, a: &Acao, tamanho: Vec2) -> Guardada {
    let montagem = blocos::montar(f, a);
    let lista: Vec<(&Plano, &Bloco)> = montagem.blocos.iter().filter_map(|b| b.plano.as_ref().map(|q| (q, b))).collect();
    if lista.is_empty() {
        return Guardada { tamanho, vazia: true, cam: Camera::default(), tracos: Vec::new(), movida: Mutex::new(None) };
    }
    let planos: Vec<&Plano> = lista.iter().map(|(q, _)| *q).collect();
    let o = Obra::de(&planos);
    let cam = Camera::enquadrar(&o.pontos(), Rect::from_min_size(Pos2::ZERO, tamanho).shrink(PAD), 0.0, AZIMUTE, ELEVACAO);
    let pecas: Vec<Vec<PecaObra>> = lista.iter().map(|(_, b)| pecas_de(b)).collect();
    let ponto: Option<String> = com_ponto(f, |r| r.map(str::to_string));
    let (cena, extras) = obra_cena::miniatura(&o, &lista, &pecas, ponto.as_deref());
    Guardada { tamanho, vazia: false, cam, tracos: gravar(&cena, &cam, &extras), movida: Mutex::new(None) }
}

/// Desenha a obra dentro de `area`, sem tirar nada do lugar. `realce` é o mouse em cima da linha, de 0 a 1.
pub fn miniatura(c: &Pincel, f: &Folha, a: &Acao, servidor: &str, area: Rect, realce: f32) {
    let ctx = c.p.ctx();
    let g = guardar(ctx, f, a, c.p.pixels_per_point(), das_miniaturas, || montar_cena(f, a, area.size()));
    // `size()` é `max - min`: com a linha em outro lugar, o mesmo tamanho pode sair com um resto de arredondamento, e isso não é outra miniatura
    let g = if (g.tamanho - area.size()).abs().max_elem() < 0.01 { g } else { Arc::new(montar_cena(f, a, area.size())) };
    if g.vazia {
        let meio = area.center();
        let (w, h) = (area.width() * 0.34, area.width() * 0.17);
        let losango = [meio + egui::vec2(0.0, -h), meio + egui::vec2(w, 0.0), meio + egui::vec2(0.0, h), meio + egui::vec2(-w, 0.0), meio + egui::vec2(0.0, -h)];
        c.p.extend(egui::Shape::dashed_line(&losango, Stroke::new(1.0, MEDIO), 5.0, 4.0));
        return;
    }
    let d = area.min.to_vec2();
    let mut cam = g.cam;
    cam.origem += d;
    let recorte = c.p.with_clip_rect(area.intersect(c.p.clip_rect()));
    for t in g.no_lugar(d).iter() {
        match t {
            Traco::Formas(v) => recorte.extend(v.iter().cloned()),
            Traco::Figura(figura, cantos, alfa, largura) => {
                if let Some(textura) = super::super::figura::textura(ctx, servidor, figura.as_ref(), *largura) {
                    imagem_na_face(&cam, &recorte, textura, cantos, *alfa);
                }
            }
        }
    }
    if realce > 0.0 {
        c.p.rect_filled(area, 0, TINTA.gamma_multiply(0.03 * realce));
    }
}

/// O ponto em que a folha está mora nesta ação?
pub fn tem_o_ponto(f: &Folha, a: &Acao) -> bool {
    com_ponto(f, |r| r.is_some_and(|r| a.planos.iter().any(|p| p.referencia == r)))
}

/// O canto de baixo da chapa na miniatura em `area`: é por ali que as esteiras da folha saem e entram.
pub fn pe(a: &Acao, area: Rect) -> Pos2 {
    let planos = blocos::planos_da(a);
    if planos.is_empty() {
        return area.center_bottom();
    }
    let q: Vec<&Plano> = planos.iter().collect();
    let o = Obra::de(&q);
    let cam = Camera::enquadrar(&o.pontos(), area.shrink(PAD), 0.0, AZIMUTE, ELEVACAO);
    let (_, x1, _, y1) = o.limites;
    cam.ponto([x1, y1, -0.35])
}

fn casas(f: &Folha, a: &Acao) -> Vec<regra::Casa> {
    regra::casas(blocos::montar(f, a).blocos.iter())
}

fn de_pe(a: &Acao) -> (String, usize, usize) {
    let (n, t) = (a.planos.iter().filter(|q| feito(q)).count(), a.planos.len());
    (format!("{n}/{t}"), n, t)
}

fn letra_de_pe() -> Letra {
    Letra::nova(11.0, TINTA).forte()
}

/// As casas da ação e as larguras, medidas uma vez por folha e escala.
fn fileira(c: &Pincel, f: &Folha, a: &Acao) -> Arc<Fileira> {
    guardar(c.p.ctx(), f, a, c.p.pixels_per_point(), das_fileiras, || {
        let casas = casas(f, a);
        let larguras = casas.iter().map(|k| icones::largura_casa(c, k)).collect();
        Fileira { casas, larguras, w_de_pe: c.largura(&de_pe(a).0, &letra_de_pe()) }
    })
}

/// A largura da fileira de peças.
pub fn largura_pecas(c: &Pincel, f: &Folha, a: &Acao) -> f32 {
    let k = fileira(c, f, a);
    icones::LADO + 3.0 + k.w_de_pe + VAO_CASAS + k.larguras.iter().map(|w| w + VAO_CASAS).sum::<f32>()
}

/// A fileira de peças com o meio em `y`. Devolve se clicaram numa delas.
pub fn pecas(ui: &mut Ui, c: &Pincel, f: &Folha, a: &Acao, x: f32, y: f32) -> bool {
    let ctx = ui.ctx().clone();
    let (texto, n, t) = de_pe(a);
    icones::desenhar(c.p, c.pt(x + icones::LADO * 0.5, y), icones::LADO, Icone::Obra, if n == t && t > 0 { VERDE } else { MEDIO });
    let mut xx = x + icones::LADO + 3.0;
    xx += c.texto(&texto, xx, y + 4.0, letra_de_pe()) + VAO_CASAS;
    let mut clicou = false;
    let k = fileira(c, f, a);
    for (j, (casa, w)) in k.casas.iter().zip(&k.larguras).enumerate() {
        if icones::casa(ui, c, (&a.referencia, "linha", j), casa, xx, y) {
            obra_bordo::ir(&ctx, a, casa);
            // a ação abre lá embaixo: a página anda até o palco, para o clique dar em algum lugar à vista
            mergulho::pousar(&ctx, a);
            clicou = true;
        }
        xx += w + VAO_CASAS;
    }
    clicou
}
