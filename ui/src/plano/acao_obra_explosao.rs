//! A vista expandida no próprio palco: a construção escolhida se abre onde está. As camadas (`acao_partes_explodida.rs`) sobem do lote dela, no mundo do palco, com a câmera dele e na mesma `Cena`; as outras construções se apagam, e a escolhida dá lugar às camadas.
//! Aqui fica o que o palco precisa saber deste nível: quem está aberto, o quanto cada construção some, onde a câmera para e o que o clique e a roda pedem. Um clique numa tela, ou a roda sobre ela, entra na tela; noutra peça, escolhe a peça; no topo, ou com a roda para trás, as camadas fecham e se volta à construção.

use super::super::linha::Pincel;
use super::super::modelo::{Acao, Plano};
use super::blocos::Bloco;
use super::explodida::{self, chamada, Nivel, Pilha, FOLGA, PE_NA_BANDEJA};
use super::figura::{Aparencia, Peca, LADO};
use super::obra_cena::{self, subiu_visto, Quadro};
use super::obra_figura::{Clique, Modulo, Obra, ALTO};
use super::obra_pecas::{Extra, PecaObra};
use super::obra_tela::{self as tela, Lugar};
use super::{na_tela, obra_mergulho as mergulho, obra_zoom};
use crate::anatomia::{Camera, Cena, V3, AZIMUTE, ELEVACAO};
use crate::i18n::tr;
use crate::tema::DESTAQUE;
use egui::{pos2, vec2, Context, CursorIcon, Id, Pos2, Rect, Response, Ui};

/// As chaves de clique das camadas, longe das da planta: construções e peças (`chave_modulo`, `chave_peca`) ficam nos milhares, os dutos (`chave_duto`) a partir de um milhão, e os letreiros são números de extra a partir de `1 << 24`, sem chave. O topo da pilha é esta; a peça `j` é `CHAVE_TOPO + j + 1`.
const CHAVE_TOPO: u32 = 1 << 27;
/// O que fica das outras construções com a vista aberta.
const RESTO: f32 = 0.12;
/// Separação abaixo disto: a construção já fechou.
const FECHOU: f32 = 0.002;
const SEG_VOO: f32 = 0.75;
/// A pilha fica nesta fração da esquerda do que a coluna deixa livre; o resto é das chamadas.
const FRACAO_PILHA: f32 = 0.6;
const MARGEM: f32 = 20.0;
/// A roda sobre uma tela, a partir deste zoom (em relação ao da pilha enquadrada), entra nela.
const ENTRA_NA_TELA: f32 = 1.8;
/// A roda para trás, abaixo deste zoom, fecha as camadas.
const VOLTA_A_PLANTA: f32 = 0.8;
/// O piso da construção, acima do patamar.
const PISO: f32 = 0.12;

/// Uma construção aberta, ou ainda fechando, neste quadro.
pub struct Explosao {
    pub i: usize,
    /// a separação, já suavizada
    pub t: f32,
    pub aberta: bool,
    m: Modulo,
    niveis: Vec<Nivel>,
    zs: Vec<f32>,
    /// até onde a pilha sobe acima do lote, separada
    topo: f32,
}

fn id(a: &Acao, qual: &str) -> Id {
    Id::unique(("plano-obra-explosao", qual, &a.referencia))
}

/// A mesma câmera no ângulo de sempre: o enquadramento da pilha é medido nela.
fn sem_giro(c: &Camera) -> Camera {
    Camera::nova(AZIMUTE, ELEVACAO, c.escala, c.foco, c.origem, c.perspectiva)
}

impl Explosao {
    fn lote(&self) -> V3 {
        [self.m.x, self.m.y, self.m.base + PISO]
    }

    /// O meio da pilha separada: neste nível o giro roda em torno dele.
    fn meio(&self) -> V3 {
        let [x, y, z] = self.lote();
        [x + LADO * 0.5, y + LADO * 0.5, (z + self.topo) * 0.5]
    }

    /// A caixa da pilha separada, do chão da chapa ao topo da construção.
    fn cantos(&self) -> Vec<V3> {
        let [x, y, z] = self.lote();
        let (x0, x1, y0, y1) = (x - FOLGA, x + LADO + FOLGA, y - FOLGA, y + LADO + FOLGA);
        [(x0, y0), (x1, y0), (x1, y1), (x0, y1)].iter().flat_map(|(u, v)| [[*u, *v, 0.0], [*u, *v, z + self.topo]]).collect()
    }

    /// As camadas chegam antes de a construção sumir, e somem depois dela.
    fn entrada(&self) -> f32 {
        (self.t * 2.0).min(1.0)
    }

    /// O zoom em que a pilha inteira cabe na parte esquerda do que a coluna deixa livre, e o ponto que vai para o meio da vista para ela ficar lá.
    fn enquadramento(&self, base: &Camera, area: Rect, livre: Rect) -> (V3, f32) {
        let um = Camera::nova(base.azimute(), base.elevacao(), 1.0, base.foco, Pos2::ZERO, 0.0);
        let r = Rect::from_points(&self.cantos().iter().map(|p| um.ponto(*p)).collect::<Vec<_>>());
        let fim = Rect::from_min_size(area.min, vec2(super::obra::largura_livre(area.width()), area.height()));
        let zona = vec2(fim.width() * FRACAO_PILHA, fim.height()) - vec2(MARGEM, MARGEM) * 2.0;
        let escala = (zona.x / r.width().max(1e-3)).min(zona.y / r.height().max(1e-3));
        let k = (escala / base.escala.max(1e-3)).clamp(obra_zoom::MIN, obra_zoom::MAX);
        // a origem da câmera quando a coluna terminar de abrir, e onde o meio da pilha deve cair
        let origem = fim.center() + (base.origem - livre.center());
        let alvo = pos2(fim.left() + fim.width() * FRACAO_PILHA * 0.5, fim.center().y);
        let d = sem_giro(base).da_tela(alvo - origem);
        let m = self.meio();
        ([m[0] - d[0] / k, m[1] - d[1] / k, m[2] - d[2] / k], k)
    }

    /// Voa até a pilha inteira, na câmera do próprio palco.
    pub fn enquadrar(&self, ctx: &Context, zoom: Id, base: &Camera, area: Rect, livre: Rect, seg: f32) {
        let (ponto, k) = self.enquadramento(base, area, livre);
        obra_zoom::focar(ctx, zoom, &sem_giro(base), ponto, k, obra_zoom::SOLTO, seg);
    }

    fn pilha<'p, 'q>(&'p self, ctx: &Context, a: &Acao, q: &'p Quadro<'q, '_>) -> Option<Pilha<'p, 'q>> {
        let (plano, b) = *q.lista.get(self.i)?;
        let pecas = q.pecas.get(self.i)?;
        let [x, y, z0] = self.lote();
        // a camada que passa por baixo da coluna some como o resto da planta
        let alfas = self.zs.iter().map(|dz| {
            let z = z0 + dz;
            self.entrada() * q.sumico.fica(q.cam, &[[x, y, z], [x + LADO, y, z], [x, y + LADO, z], [x + LADO, y + LADO, z]])
        }).collect();
        let cor = if q.em_destaque(self.i) { DESTAQUE } else { Aparencia::de(plano, 0.0).traco };
        let escolhida = q.peca.filter(|_| q.dentro == Some(self.i));
        let chave = if self.aberta { CHAVE_TOPO } else { 0 };
        Some(Pilha { q: plano, pecas: pecas.as_slice(), comp: tela::composicao(plano, !b.telas.is_empty()), niveis: &self.niveis, zs: &self.zs, alfas, lote: self.lote(), subiu: subiu_visto(ctx, a, q, self.i), cor, acesas: [escolhida, q.acesa.as_deref()], chave })
    }

    /// O clique e a roda neste nível. Devolve o clique que ainda é da planta: outra construção, antes de se apagar de todo.
    #[allow(clippy::too_many_arguments)]
    pub fn gestos(&self, r: Response, a: &Acao, planos: &[&Plano], pecas: &[Vec<PecaObra>], cena: &Cena, cam: &Camera, base: &Camera, zoom: Id, area: Rect, livre: Rect, toque: u32) -> Option<Clique> {
        let ctx = r.ctx.clone();
        let (q, minhas) = (planos.get(self.i)?, pecas.get(self.i)?);
        let peca_de = |chave: u32| chave.checked_sub(CHAVE_TOPO + 1).and_then(|j| minhas.get(j as usize));
        let sob = peca_de(toque);
        mergulho::acender(&ctx, a, sob.map(|p| p.chave.clone()));
        // lida antes do gesto, que consome a roda
        let roda = if r.contains_pointer() { ctx.input(|i| i.smooth_scroll_delta().y) } else { 0.0 };
        obra_zoom::arrastar(&r);
        // a roda aproxima em torno do ponteiro, medido da origem que o giro deixou
        let mut b = *base;
        b.origem = cam.origem;
        obra_zoom::gesto(&r, zoom, &b, obra_zoom::SOLTO);
        let k = obra_zoom::perto(&ctx, zoom);
        let (_, k_cheio) = self.enquadramento(base, area, livre);
        if let Some(p) = sob.filter(|p| p.tipo == Peca::Tela && roda > 0.0 && k >= (k_cheio * ENTRA_NA_TELA).min(obra_zoom::MAX * 0.98)) {
            na_tela::ver(&ctx, a, &q.referencia, &p.chave);
        } else if roda < 0.0 && k <= (k_cheio * VOLTA_A_PLANTA).max(obra_zoom::MIN * 1.02) {
            mergulho::recolher(&ctx, a);
        }
        if r.double_clicked() && toque == 0 {
            obra_zoom::desvirar(&ctx);
            self.enquadrar(&ctx, zoom, base, area, livre, SEG_VOO);
        }
        if r.hovered() {
            ctx.set_cursor_icon(if r.dragged() { CursorIcon::Grabbing } else if toque != 0 { CursorIcon::PointingHand } else { CursorIcon::Grab });
        }
        let mut clique = None;
        if r.clicked() {
            let chave = r.interact_pointer_pos().map_or(0, |p| cena.tocar(cam, p));
            match peca_de(chave) {
                // aqui o clique numa tela já é o próximo nível: entra nela
                Some(p) if p.tipo == Peca::Tela => na_tela::ver(&ctx, a, &q.referencia, &p.chave),
                Some(p) => mergulho::escolher(&ctx, a, &p.chave),
                None if chave == CHAVE_TOPO => mergulho::recolher(&ctx, a),
                None if chave < CHAVE_TOPO => clique = obra_cena::clique_da_chave(planos, pecas, chave),
                None => {}
            }
        }
        let dica = if toque == CHAVE_TOPO {
            Some(tr("Clique para fechar as camadas").to_string())
        } else {
            sob.map(|p| format!("{} {}{}", p.numero, p.nome, if p.tipo == Peca::Tela { format!("\n\n{}", tr("Clique para entrar na tela")) } else { String::new() }))
        };
        if let Some(t) = dica {
            let _ = r.on_hover_text_at_pointer(t);
        }
        clique
    }
}

/// As construções abertas, ou ainda fechando. Lê a separação de todas a cada quadro, para a abertura e o fechamento andarem.
pub fn ativas(ctx: &Context, a: &Acao, lista: &[(&Plano, &Bloco)], o: &Obra) -> Vec<Explosao> {
    let aberta = mergulho::expandida(ctx, a);
    lista.iter().zip(&o.modulos).enumerate().filter_map(|(i, ((q, b), m))| {
        let e_ela = aberta.as_deref() == Some(q.referencia.as_str());
        let t = explodida::separacao(ctx, a, &q.referencia, e_ela);
        if !e_ela && t < FECHOU {
            return None;
        }
        let niveis = explodida::niveis(b);
        let topo = explodida::alturas(&niveis, 1.0).last().copied().unwrap_or(0.0) + ALTO;
        let zs = explodida::alturas(&niveis, t);
        Some(Explosao { i, t, aberta: e_ela, m: *m, niveis, zs, topo })
    }).collect()
}

/// O quanto cada construção fica, pela vista expandida: a aberta dá lugar às camadas (de 1 a 0) e as outras se apagam até `RESTO`.
pub fn apagar(n: usize, abertas: &[Explosao]) -> Vec<f32> {
    (0..n).map(|k| abertas.iter().fold(1.0_f32, |f, e| f * if e.i == k { 1.0 - e.t } else { 1.0 - e.t * (1.0 - RESTO) })).collect()
}

/// Em quantos segundos a câmera voa porque o nível mudou. A pergunta consome o pedido.
pub fn voo_pedido(ctx: &Context, a: &Acao) -> Option<f32> {
    ctx.data_mut(|d| d.remove_temp::<f32>(id(a, "voar")))
}

/// Uma vez por quadro, antes de desenhar: o mergulho e a vista expandida falam do mesmo plano. Quem foi a outro plano (setas, dependência, outra construção) fecha as camadas; quem abriu as camadas de outro plano entra nele. Mudou o nível, a câmera voa.
pub fn acertar(ctx: &Context, a: &Acao, planos: &[&Plano]) {
    let era = ctx.data(|d| d.get_temp::<Option<String>>(id(a, "era"))).flatten();
    let plano = mergulho::plano(ctx, a);
    let mut agora = mergulho::expandida(ctx, a);
    if let Some(e) = agora.clone() {
        let fora = plano.as_deref() != Some(e.as_str());
        if !planos.iter().any(|q| q.referencia == e) || (fora && era.as_deref() == Some(e.as_str())) {
            mergulho::recolher(ctx, a);
            agora = None;
        } else if fora {
            mergulho::entrar(ctx, a, &e);
        }
    }
    if agora != era {
        ctx.data_mut(|d| {
            d.insert_temp(id(a, "era"), agora);
            d.insert_temp(id(a, "voar"), SEG_VOO);
        });
    }
}

/// O giro, neste nível, roda em torno do meio da pilha e não do chão: ela fica onde o enquadramento a pôs.
pub fn sem_balanco(abertas: &[Explosao], cam: Camera) -> Camera {
    let Some(e) = abertas.iter().max_by(|x, y| x.t.total_cmp(&y.t)) else { return cam };
    let mut c = cam;
    c.origem += (sem_giro(&cam).ponto(e.meio()) - cam.ponto(e.meio())) * e.t;
    c
}

/// Põe as camadas na cena do palco, depois de tudo o que já está nela. As figuras das telas entram na mesma lista das outras, então a cena as pinta na vez delas sem conta à parte.
pub fn por_na_cena<'q>(ctx: &Context, a: &Acao, abertas: &[Explosao], q: &Quadro<'q, '_>, cena: &mut Cena, imagens: &mut Vec<Extra<'q>>) {
    for e in abertas {
        if let Some(p) = e.pilha(ctx, a, q) {
            explodida::montar(ctx, a, cena, imagens, &p);
        }
    }
}

/// As chamadas de cada camada, ao lado da pilha, entre ela e a coluna. Somem ao entrar numa tela.
pub fn chamadas(ui: &mut Ui, c: &Pincel, a: &Acao, abertas: &[Explosao], q: &Quadro, livre: Rect, t_tela: f32) {
    let ctx = ui.ctx().clone();
    for e in abertas {
        let (Some(p), Some((_, b))) = (e.pilha(&ctx, a, q), q.lista.get(e.i)) else { continue };
        let k = ((e.t - 0.5) * 2.0).clamp(0.0, 1.0) * (1.0 - t_tela);
        chamada::desenhar(ui, c, a, q.f, b, &p, q.cam, livre, k, e.aberta && t_tela == 0.0);
    }
}

/// Onde a tela está agora: na bandeja da pilha, se a construção dela está aberta; senão, de pé na construção. A câmera entra na tela por aí, e é aí que ela é pintada.
pub fn onde_esta<'a>(abertas: &[Explosao], i: usize, p: &PecaObra<'a>, m: &Modulo) -> (PecaObra<'a>, Modulo) {
    let z = abertas.iter().find(|e| e.i == i).and_then(|e| e.niveis.iter().position(|n| *n == Nivel::Telas).and_then(|k| e.zs.get(k)));
    let (Some(z), Some(l)) = (z, p.lugar) else { return (p.clone(), *m) };
    let fila = explodida::fila_de(Peca::Tela);
    let lugar = Lugar { v: fila, chao: 0.0, pe: PE_NA_BANDEJA, ..l };
    (PecaObra { lugar: Some(lugar), u: l.u, v: fila, ..p.clone() }, Modulo { base: m.base + z, ..*m })
}

#[cfg(test)]
mod tests {
    use super::super::obra_figura::{chave_duto, chave_modulo, chave_peca};
    use super::*;

    fn aberta(i: usize, t: f32) -> Explosao {
        let niveis = vec![Nivel::Base, Nivel::Telas, Nivel::Obra];
        let zs = explodida::alturas(&niveis, t);
        Explosao { i, t, aberta: true, m: Modulo { x: 7.0, y: 0.0, h: ALTO, base: 0.55, nivel: 1 }, niveis, zs, topo: 3.5 + ALTO }
    }

    #[test]
    fn a_aberta_some_e_as_outras_ficam_de_leve() {
        assert_eq!(apagar(3, &[]), vec![1.0; 3]);
        let v = apagar(3, &[aberta(1, 1.0)]);
        assert_eq!(v[1], 0.0);
        assert!((v[0] - RESTO).abs() < 1e-6 && (v[2] - RESTO).abs() < 1e-6);
    }

    #[test]
    fn as_chaves_das_camadas_nao_encostam_nas_da_planta() {
        assert!(chave_peca(4000, 999) < CHAVE_TOPO && chave_duto(100_000) < CHAVE_TOPO && chave_modulo(4000) < CHAVE_TOPO);
        assert!(CHAVE_TOPO > 1 << 24);
    }

    #[test]
    fn a_tela_da_pilha_aberta_fica_na_bandeja() {
        let p = PecaObra { tipo: Peca::Tela, chave: "t:1".into(), nome: String::new(), marca: super::super::figura::Marca::Neutra, tela: None, lugar: Some(Lugar { u: 0.4, v: 0.42, chao: 0.12, pe: 0.5, w: 2.0, h: 1.2 }), numero: "1.1".into(), visivel: true, u: 0.4, v: 0.42 };
        let m = aberta(0, 1.0).m;
        let (fechada, _) = onde_esta(&[], 0, &p, &m);
        assert_eq!(fechada.lugar, p.lugar);
        let (na_bandeja, mb) = onde_esta(&[aberta(0, 1.0)], 0, &p, &m);
        // a base e a bandeja das telas: um de vão, e o pé curto em cima dela
        assert!((na_bandeja.centro(&mb)[2] - (m.base + PISO + 1.0 + PE_NA_BANDEJA + 0.6)).abs() < 1e-5);
    }
}
