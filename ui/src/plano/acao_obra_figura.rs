//! A obra: onde cada construção fica, o palco em 3D e o que os gestos pedem. A cena em si sai de `acao_obra_cena.rs`, a vista expandida, que abre aqui mesmo, de `acao_obra_explosao.rs`, e a câmera, do motor Anatomy.
//! Quem depende de outra fica um degrau acima dela e à direita: a escada do blueprint, em pé. Arrastar gira a obra em torno do meio da vista, a roda aproxima e afasta, clicar aproxima até lá (o giro e o zoom moram em `acao_obra_zoom.rs`).

use super::super::acao_movimento::{desligado, fase};
use super::super::linha::Pincel;
use super::super::modelo::{Acao, Folha, Plano};
use super::super::revisao::Alvo;
use super::blocos::Bloco;
use super::figura::{self, Aparencia, Conteudo, LADO};
use super::forma::{dispor, niveis};
use super::obra_cena::{self, Quadro};
use super::obra_pecas::{self, PecaObra};
use super::obra_sumico::Sumico;
use super::{na_tela, obra_explosao as explosao, obra_fora as fora, obra_mergulho as mergulho, obra_regra, obra_rotulos, obra_zoom, voo};
#[cfg(not(target_arch = "wasm32"))]
pub use super::obra_zoom::orbitar;
use crate::anatomia::{Camera, V3, AZIMUTE, ELEVACAO};
use crate::plano::vazio_iso::Iso;
use crate::tema::TINTA;
use egui::{vec2, Context, CursorIcon, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

/// A distância entre uma coluna e a seguinte, e entre uma linha e a seguinte, em unidades do mundo. As construções andam nos eixos da chapa, e ela fica justa em volta: sem sobra nos cantos, cada construção aparece maior.
const PASSO: f32 = 7.0;
const PASSO_LINHA: f32 = 7.0;
pub const ALTO: f32 = 2.6;
/// O degrau de cada nível: o que depende fica em cima do que veio antes.
const DEGRAU: f32 = 0.55;
const MARGEM: f32 = 2.2;
const COLUNAS: usize = 5;
/// O que cabe acima de cada construção: a bandeira e o rótulo.
const ACIMA: f32 = 1.5;
/// Quanto do palco a obra ocupa ao abrir.
const FRACAO_ENQUADRADA: f32 = 0.9;
/// O painel de bordo cobre o canto de baixo quando o palco é estreito.
const LARGURA_SEM_RESERVA: f32 = 1300.0;
const RESERVA_BORDO: f32 = 72.0;
/// Desce um pouco do meio: a bandeira e o rótulo ficam em cima.
const DESCE_ROTULOS: f32 = 14.0;
const AUMENTO_DA_PECA: f32 = 1.4;
/// Quantos lados de construção cabem na vista ao mergulhar: ela aparece inteira, com o chão em volta.
const LADOS_NA_VISTA: f32 = 2.5;
/// Afastou até esta fração do zoom de quem entrou: já saiu.
const FRACAO_SOLTA: f32 = 0.7;
const CHAVE_DUTO: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Modulo {
    pub x: f32,
    pub y: f32,
    pub h: f32,
    /// a altura do patamar em que ele está
    pub base: f32,
    pub nivel: usize,
}

impl Modulo {
    pub fn meio(&self, z: f32) -> V3 {
        [self.x + LADO * 0.5, self.y + LADO * 0.5, self.base + z]
    }
}

/// Onde tudo fica no mundo. A escala é da câmera.
#[derive(Clone, Debug, PartialEq)]
pub struct Obra {
    pub modulos: Vec<Modulo>,
    pub niveis: usize,
    pub limites: (f32, f32, f32, f32),
}

impl Obra {
    /// Precisa de pelo menos um plano.
    pub fn de(q: &[&Plano]) -> Obra {
        let n = niveis(q);
        // sem dependência nenhuma, nada obriga a fila: a chapa fica o mais quadrada possível
        let cabem = if n.iter().all(|v| *v == 0) { ((q.len() as f32).sqrt().ceil() as usize).clamp(1, COLUNAS) } else { COLUNAS };
        let modulos: Vec<Modulo> = dispor(q, cabem).iter().zip(&n).map(|(l, v)| {
            Modulo { x: l.col as f32 * PASSO, y: l.lin * PASSO_LINHA, h: ALTO, base: *v as f32 * DEGRAU, nivel: *v }
        }).collect();
        let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for (m, p) in modulos.iter().zip(q) {
            // a esteira que vem de outra ação nasce fora da construção, e a chapa vai até ela
            let entrada = if fora::pede_espaco(p, q) { fora::ENTRADA } else { 0.0 };
            (x0, x1, y0, y1) = (x0.min(m.x - entrada), x1.max(m.x + LADO), y0.min(m.y), y1.max(m.y + LADO));
        }
        Obra { modulos, niveis: n.iter().max().map_or(1, |v| v + 1), limites: (x0 - MARGEM, x1 + MARGEM, y0 - MARGEM, y1 + MARGEM) }
    }

    /// Os pontos que a câmera precisa enquadrar.
    pub fn pontos(&self) -> Vec<V3> {
        let (x0, x1, y0, y1) = self.limites;
        let mut v = vec![[x0, y0, -0.4], [x1, y0, -0.4], [x1, y1, -0.4], [x0, y1, -0.4]];
        v.extend(self.modulos.iter().map(|m| m.meio(m.h + ACIMA)));
        v
    }
}

pub fn chave_modulo(i: usize) -> u32 {
    (i as u32 + 1) * 1000
}

pub fn chave_peca(i: usize, j: usize) -> u32 {
    (i as u32 + 1) * 1000 + j as u32 + 1
}

pub fn chave_duto(n: usize) -> u32 {
    CHAVE_DUTO + n as u32
}

/// O que o clique no palco pediu.
#[derive(Clone, Debug, PartialEq)]
pub enum Clique {
    Modulo(usize),
    Peca(usize, String),
    /// a construção de onde o duto vem
    Duto(usize),
}

/// O que aconteceu no palco neste quadro.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gesto {
    pub clique: Option<Clique>,
    /// a placa de um plano de outra ação, e de onde ela sai na tela
    pub fora: Option<(String, Pos2)>,
    /// a roda passou de perto da construção: o próximo nível é a vista expandida dela
    pub expandir: Option<usize>,
}

fn id_aceso(a: &Acao) -> Id {
    Id::unique(("plano-obra-aceso", &a.referencia))
}

/// A câmera de quem olha a obra inteira, sempre medida pelo palco cheio. A ficha flutua em cima dele e só muda onde fica o meio da vista: o meio do que está `livre`.
fn camera_base(o: &Obra, area: Rect, livre: Rect, [az, el]: [f32; 2]) -> Camera {
    let local = Rect::from_min_size(Pos2::ZERO, area.size());
    let reserva = if area.width() < LARGURA_SEM_RESERVA { RESERVA_BORDO } else { 0.0 };
    let enquadrado = Rect::from_center_size(local.center() + vec2(0.0, DESCE_ROTULOS - reserva * 0.5), area.size() * FRACAO_ENQUADRADA - vec2(0.0, reserva));
    let mut c = Camera::enquadrar(&o.pontos(), enquadrado, 0.0, AZIMUTE, ELEVACAO).girada(az, el);
    c.origem = livre.center() + (c.origem - local.center());
    c
}

/// O zoom em que uma construção aparece inteira no que a ficha deixa livre, sem o corte da borda em cima dela.
fn perto_da_construcao(base: &Camera, area: Rect) -> f32 {
    let livre = super::obra::largura_livre(area.width());
    let escala = (livre.min(area.height() * 1.25) / (LADO * LADOS_NA_VISTA)).clamp(36.0, 150.0);
    (escala / base.escala).max(1.25)
}

/// Até onde o olhar anda, a partir do meio da obra.
fn alcance(o: &Obra) -> Vec2 {
    let (x0, x1, y0, y1) = o.limites;
    vec2(x1 - x0, y1 - y0) * 0.5
}

/// O zoom desta obra: o mergulho volta a ele enquadrado.
pub fn id_zoom(a: &Acao) -> Id {
    Id::unique(("plano-obra-zoom", &a.referencia))
}

/// Desenha o palco em `area` (tela); o mouse só mexe nele em `livre`, o que a ficha não cobre. `t_dentro` é o mergulho, de 0 a 1.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, lista: &[(&Plano, &Bloco)], o: &Obra, area: Rect, livre: Rect, dentro: Option<usize>, t_dentro: f32, tau: Option<f32>, peca: Option<&str>) -> Gesto {
    let ctx = ui.ctx().clone();
    let planos: Vec<&Plano> = lista.iter().map(|(q, _)| *q).collect();
    let aceso = ctx.data(|d| d.get_temp::<Option<usize>>(id_aceso(a))).flatten();
    let pecas: Vec<Vec<PecaObra>> = lista.iter().map(|(_, b)| obra_pecas::pecas_de(b)).collect();
    let base = camera_base(o, area, livre, obra_zoom::orbita(&ctx));
    let zoom = id_zoom(a);
    let perto = perto_da_construcao(&base, area);
    // a vista expandida abre no próprio palco: a construção escolhida se separa em camadas onde está
    let abertas = explosao::ativas(&ctx, a, lista, o);
    let expandida = abertas.iter().find(|e| e.aberta);
    // `or`, e não `or_else`: os dois pedidos são consumidos a cada quadro
    if let Some(seg) = explosao::voo_pedido(&ctx, a).or(mergulho::voo_pedido(&ctx, a)) {
        if let Some(e) = expandida {
            e.enquadrar(&ctx, zoom, &base, area, livre, seg);
        } else if let Some(i) = dentro {
            let escolhida = mergulho::escolhida(&ctx, a).and_then(|ch| pecas[i].iter().find(|p| p.chave == ch).map(|p| p.centro(&o.modulos[i])));
            let (ponto, k) = escolhida.map_or((o.modulos[i].meio(0.9), perto), |p| (p, perto * AUMENTO_DA_PECA));
            obra_zoom::focar(&ctx, zoom, &base, ponto, k, alcance(o), seg);
        }
    }
    let cam = explosao::sem_balanco(&abertas, obra_zoom::aplicar(&ctx, zoom, base));
    // dentro da tela: a câmera sai de onde está e vai até ela, de frente, na bandeja se a construção está aberta
    let t_tela = na_tela::quanto(&ctx, a);
    let vista = na_tela::olhando(&ctx, a).filter(|_| t_tela > 0.0).and_then(|ch| pecas.iter().enumerate().find_map(|(i, v)| v.iter().find(|p| p.chave == ch && p.lugar.is_some()).map(|p| explosao::onde_esta(&abertas, i, p, &o.modulos[i]))));
    let cam = vista.as_ref().and_then(|(p, m)| na_tela::camera(&ctx, a, p, m, area)).map_or(cam, |la| cam.rumo(&la, t_tela));
    let de_fora = fora::todos(f, &planos);
    let proximo = obra_regra::proximo(f, lista).indice().filter(|_| tau.is_none());
    let apagar = explosao::apagar(planos.len(), &abertas);
    let quadro = Quadro { f, de_fora: &de_fora, o, planos: &planos, lista, pecas: &pecas, aceso, dentro, t_dentro, tau, peca, proximo, acesa: mergulho::acesa(&ctx, a), fase: if desligado() { 0.4 } else { fase(&ctx, 0.9) }, cam: &cam, sumico: Sumico::de(&ctx, a), apagar: &apagar };
    let (mut cena, mut extras) = obra_cena::montar(&ctx, a, &quadro);
    explosao::por_na_cena(&ctx, a, &abertas, &quadro, &mut cena, &mut extras.imagens);
    let r = ui.interact(livre, ui.scope_id().with(("plano-obra-palco", &a.referencia)), Sense::click_and_drag());
    let toque = r.hover_pos().map_or(0, |p| cena.tocar(&cam, p));
    let recorte = c.p.with_clip_rect(area.intersect(c.p.clip_rect()));
    let antes = ui.clip_rect();
    ui.set_clip_rect(area.intersect(antes));
    // as figuras das telas e os letreiros dos monitores sem tela, cada um na sua vez
    cena.desenhar(&cam, &recorte, |n| extras.pintar(n, &ctx, alvo.servidor, &cam, &recorte));
    // de perto a chapa passa da borda: a moldura diz que é uma janela para a obra, não um desenho cortado
    let k = obra_zoom::perto(&ctx, zoom);
    if k > 1.02 {
        recorte.rect_stroke(area, 0, Stroke::new(1.0, TINTA.gamma_multiply(((k - 1.0) * 2.0).clamp(0.0, 1.0) * 0.55)), StrokeKind::Inside);
    }
    let pc = Pincel { p: &recorte, o: c.o };
    explosao::chamadas(ui, &pc, a, &abertas, &quadro, livre, t_tela);
    let ocupadas: Vec<Rect> = fora::areas(&pc, &cam, o, &de_fora).iter().map(|r| r.translate(-c.o.to_vec2())).collect();
    let do_rotulo = obra_rotulos::desenhar(ui, &pc, &cam, alvo, a, &quadro, &ocupadas);
    let placa = fora::placas(ui, &pc, &quadro, 1.0 - t_dentro * 0.5).or(do_rotulo);
    if let Some((p, m)) = &vista {
        na_tela::pintar(&ctx, alvo.servidor, &cam, &recorte, p, m, area, t_tela);
    }
    ui.set_clip_rect(antes);
    let mut gesto = Gesto { fora: placa, ..Gesto::default() };
    if na_tela::aberta(&ctx, a).is_some() {
        if let Some(l) = vista.as_ref().and_then(|(p, _)| p.lugar) {
            na_tela::gesto(&r, a, &l, area);
        }
        return gesto;
    }
    if let Some(e) = expandida {
        ctx.data_mut(|d| d.insert_temp(id_aceso(a), None::<usize>));
        gesto.clique = e.gestos(r, a, &planos, &pecas, &cena, &cam, &base, zoom, area, livre, toque);
        return gesto;
    }
    // a laje que vem de outra tela pousa no seu módulo
    for (i, (q, b)) in lista.iter().enumerate() {
        let m = &o.modulos[i];
        if let Some(v) = voo::em_voo(&ctx, &a.referencia, &format!("planta:{}", q.referencia), cam.ponto(m.meio(m.h)), cam.escala * 0.8) {
            let camada = voo::camada(&ctx);
            figura::laje(&Iso { p: &camada, o: v.o, k: v.k }, &Conteudo::de(b), &Aparencia::de(q, 1.0));
        }
    }
    let duto = |n: u32| obra_cena::duto_da_chave(&planos, n);
    let (sobre, dica) = obra_cena::o_que_e(&planos, &pecas, toque);
    ctx.data_mut(|d| d.insert_temp(id_aceso(a), sobre.or(duto(toque).map(|u| u.para))));
    let sob_o_mouse = obra_cena::clique_da_chave(&planos, &pecas, toque);
    mergulho::acender(&ctx, a, if let Some(Clique::Peca(_, ch)) = &sob_o_mouse { Some(ch.clone()) } else { None });
    let aproximou = r.contains_pointer() && ctx.input(|i| i.smooth_scroll_delta().y) > 0.0;
    obra_zoom::arrastar(&r);
    obra_zoom::gesto(&r, zoom, &base, alcance(o));
    // a roda continua depois de chegar perto da construção: o próximo nível é a vista expandida, e só depois a tela
    let no_fundo = aproximou && obra_zoom::perto(&ctx, zoom) >= (perto * AUMENTO_DA_PECA).min(obra_zoom::MAX) * 0.98;
    gesto.expandir = match sob_o_mouse.as_ref().filter(|_| no_fundo) {
        Some(Clique::Peca(i, _)) | Some(Clique::Modulo(i)) => Some(*i),
        _ => None,
    };
    if dentro.is_some() && k < (perto * FRACAO_SOLTA).max(1.05) {
        mergulho::soltar(&ctx, a);
    }
    if r.hovered() {
        ctx.set_cursor_icon(if r.dragged() { CursorIcon::Grabbing } else if toque != 0 { CursorIcon::PointingHand } else { CursorIcon::Grab });
    }
    if r.double_clicked() && toque == 0 {
        obra_zoom::desvirar(&ctx);
        obra_zoom::zerar(&ctx, zoom);
    }
    if r.clicked() {
        let chave = r.interact_pointer_pos().map_or(0, |p| cena.tocar(&cam, p));
        gesto.clique = obra_cena::clique_da_chave(&planos, &pecas, chave);
    }
    if let Some(t) = dica.filter(|t| !t.is_empty()) {
        let _ = r.on_hover_text_at_pointer(t);
    }
    gesto
}

/// Leva o olhar à planta, já dentro do plano: a laje sai de `de` e pousa no módulo dele.
pub fn ver_na_planta(ctx: &Context, a: &Acao, plano: &str, de: Pos2, k: f32) {
    voo::decolar(ctx, &a.referencia, de, k, &format!("planta:{plano}"));
    mergulho::recolher(ctx, a);
    mergulho::entrar_de_longe(ctx, a, plano);
}

#[cfg(test)]
#[path = "acao_obra_testes.rs"]
mod tests;
