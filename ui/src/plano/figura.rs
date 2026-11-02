//! As figuras do plano: pede os bytes ao servidor e rasteriza fora do quadro (a thread do `ehttp`; no navegador, a resposta), com `resvg` e só as
//! três Cascadia embutidas, para a mesma figura sair igual em toda máquina. Cache por hash e largura arredondada a 64 px, LRU de 32 texturas.
//! Só pede o que está visível. A altura para empilhar vem das medidas do servidor (`proporcao`), sem rasterizar antes.

use super::acao_movimento::mistura;
use super::cartao_esqueleto as esqueleto;
use super::modelo::Figura;
use crate::i18n::tr;
use crate::planta::Texto;
use crate::tema;
use egui::{pos2, Color32, ColorImage, Context, Painter, Pos2, Rect, Shape, Stroke, TextureHandle, TextureOptions};
use resvg::tiny_skia;
use resvg::usvg::{self, fontdb, ImageHrefResolver};
use std::collections::{HashMap, HashSet};
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::Duration;
use web_time::Instant;

pub const TEXTOS: &[(&str, &str)] = &[("desenhando…", "drawing…")];

/// Acima desta proporção (largura/altura), a figura ocupa duas colunas da prancha e fica baixa (`tela-acao.mjs:92`).
pub const PROPORCAO_LARGA: f32 = 5.0;
const PASSO_LARGURA: u32 = 64;
const TETO_TEXTURAS: usize = 32;
/// Quantas figuras rasterizam ao mesmo tempo; as outras esperam o próximo quadro.
const TETO_EM_VOO: usize = 3;
const LADO_MAXIMO: usize = 4096;
/// A figura que falhou não é pedida a cada quadro: o servidor pode voltar, mas sem pressa.
const ESPERA_FALHA: Duration = Duration::from_secs(30);

const FAMILIA: &str = "Cascadia Mono";
const REGULAR: &[u8] = include_bytes!("../../assets/fontes/CascadiaMono-Regular.ttf");
const MEDIO: &[u8] = include_bytes!("../../assets/fontes/CascadiaMono-Medium.ttf");
const NEGRITO: &[u8] = include_bytes!("../../assets/fontes/CascadiaMono-Bold.ttf");

type Chave = (String, u32);

/// Largura/altura que o servidor mediu; sem medida, 1.
pub fn proporcao(figura: Option<&Figura>) -> f32 {
    figura.and_then(medida).unwrap_or(1.0)
}

pub fn larga(figura: Option<&Figura>) -> bool {
    proporcao(figura) > PROPORCAO_LARGA
}

/// A altura que a figura pede nesta largura, antes de existir textura.
pub fn altura_para(figura: Option<&Figura>, largura: f32) -> f32 {
    largura / proporcao(figura)
}

/// A figura inteira e centrada em `area`. Sem figura, ou com um tipo que este sistema não abre, a caixa tracejada com `nome`.
pub fn desenhar(p: &Painter, servidor: &str, figura: Option<&Figura>, nome: &str, area: Rect) {
    if !p.clip_rect().intersects(area) {
        return;
    }
    let Some(f) = figura.filter(|f| hash_valido(&f.hash) && abre(&f.tipo)) else {
        caixa(p, area, nome);
        return;
    };
    let ctx = p.ctx();
    let aparece = ("figura-aparece", f.hash.clone());
    let quadro = ctx.cumulative_frame_nr();
    let chave = (f.hash.clone(), arredondar(largura_desenhada(f, area) * ctx.pixels_per_point()));
    let figuras = do_contexto(ctx);
    let mut g = figuras.lock().unwrap_or_else(PoisonError::into_inner);
    g.receber(ctx, quadro);
    if let Some(t) = g.texturas.usar(&chave, quadro) {
        pintar(p, area, t, mistura(ctx, &aparece, true, 0.45));
        return;
    }
    let falhou = g.falhas.contains_key(&chave);
    if !falhou {
        g.pedir(ctx, servidor, f, &chave, quadro);
    }
    // enquanto a largura nova não chega, a que já existe esticada: redimensionar a janela não pisca
    if let Some(t) = g.texturas.outra_largura(&f.hash, quadro) {
        pintar(p, area, t, mistura(ctx, &aparece, true, 0.45));
    } else if falhou {
        caixa(p, area, nome);
    } else {
        // o esqueleto brilha até a figura chegar; ela então clareia de 0 a 1
        mistura(ctx, &aparece, false, 0.45);
        esqueleto::desenhar(p, area, tr("desenhando…"));
    }
}

fn medida(f: &Figura) -> Option<f32> {
    match (f.largura?, f.altura?) {
        (l, a) if l > 0 && a > 0 => Some(l as f32 / a as f32),
        _ => None,
    }
}

fn largura_desenhada(f: &Figura, area: Rect) -> f32 {
    medida(f).map_or(area.width(), |r| area.width().min(area.height() * r))
}

fn arredondar(px: f32) -> u32 {
    (px.max(1.0) / PASSO_LARGURA as f32).ceil() as u32 * PASSO_LARGURA
}

fn hash_valido(h: &str) -> bool {
    h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())
}

/// No navegador só o SVG abre: PNG, JPG e WebP precisam do `image`, que é só do desktop.
fn abre(tipo: &str) -> bool {
    match tipo {
        "svg" => true,
        "png" | "jpg" | "webp" => cfg!(not(target_arch = "wasm32")),
        _ => false,
    }
}

fn pintar(p: &Painter, area: Rect, t: &TextureHandle, opacidade: f32) {
    let lado = t.size_vec2();
    let r = Rect::from_center_size(area.center(), lado * (area.width() / lado.x).min(area.height() / lado.y));
    p.image(t.id(), r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE.gamma_multiply(opacidade));
}

/// O `figuraEm` do protótipo sem figura: fundo do cartão, traço `4 3`, texto no meio cortado em `w - 16`.
fn caixa(p: &Painter, area: Rect, texto: &str) {
    p.rect_filled(area, 0, tema::CARTAO);
    let a = area.shrink(0.5);
    let contorno: [Pos2; 5] = [a.left_top(), a.right_top(), a.right_bottom(), a.left_bottom(), a.left_top()];
    p.extend(Shape::dashed_line(&contorno, Stroke::new(1.0, tema::BAIXO), 4.0, 3.0));
    if !texto.is_empty() {
        Texto::novo(texto, 10.0).cor(tema::MEDIO).caixa((area.width() - 16.0).max(1.0)).linhas(1).centro(p, area.center());
    }
}

/// O cache fica na memória do contexto: é um só para todas as telas do Planejar e dura enquanto a janela existir.
fn do_contexto(ctx: &Context) -> Arc<Mutex<Figuras>> {
    ctx.data_mut(|d| d.get_temp_mut_or_default::<Arc<Mutex<Figuras>>>(egui::Id::unique("plano-figuras")).clone())
}

struct Figuras {
    texturas: Lru<TextureHandle>,
    em_voo: HashSet<Chave>,
    falhas: HashMap<Chave, Instant>,
    envio: Sender<(Chave, Option<ColorImage>)>,
    recebe: Receiver<(Chave, Option<ColorImage>)>,
}

impl Default for Figuras {
    fn default() -> Figuras {
        let (envio, recebe) = channel();
        Figuras { texturas: Lru::novo(TETO_TEXTURAS), em_voo: HashSet::new(), falhas: HashMap::new(), envio, recebe }
    }
}

impl Figuras {
    fn receber(&mut self, ctx: &Context, quadro: u64) {
        while let Ok((chave, imagem)) = self.recebe.try_recv() {
            self.em_voo.remove(&chave);
            match imagem {
                Some(i) => {
                    let t = ctx.load_texture(format!("figura-{}-{}", &chave.0[..8], chave.1), i, TextureOptions::LINEAR);
                    self.texturas.por(chave, t, quadro);
                }
                None => {
                    self.falhas.insert(chave, Instant::now());
                }
            }
        }
        self.falhas.retain(|_, em| em.elapsed() < ESPERA_FALHA);
    }

    fn pedir(&mut self, ctx: &Context, servidor: &str, f: &Figura, chave: &Chave, quadro: u64) {
        if self.em_voo.contains(chave) || self.em_voo.len() >= TETO_EM_VOO || !self.texturas.cabe(quadro) {
            return;
        }
        self.em_voo.insert(chave.clone());
        let lado = ctx.input(|i| i.max_texture_side).min(LADO_MAXIMO) as u32;
        let (ctx, envio, chave, tipo) = (ctx.clone(), self.envio.clone(), chave.clone(), f.tipo.clone());
        // o `ehttp` chama de volta numa thread própria no desktop: rasterizar aqui não segura o quadro
        ehttp::fetch(ehttp::Request::get(format!("{servidor}/api/mcp/planejar/figura?hash={}", chave.0)), move |resposta| {
            let imagem = resposta.ok().filter(|r| r.status == 200).and_then(|r| panic::catch_unwind(AssertUnwindSafe(|| rasterizar(&tipo, &r.bytes, chave.1, lado))).ok().flatten());
            let _ = envio.send((chave, imagem));
            ctx.request_repaint();
        });
    }
}

/// Mais usadas ficam. O que apareceu neste quadro ou no anterior não sai: mais de 32 figuras à vista esperam em vez de se expulsarem sem parar.
struct Lru<T> {
    itens: HashMap<Chave, (T, u64)>,
    teto: usize,
}

impl<T> Lru<T> {
    fn novo(teto: usize) -> Lru<T> {
        Lru { itens: HashMap::new(), teto }
    }

    fn usar(&mut self, chave: &Chave, quadro: u64) -> Option<&T> {
        let item = self.itens.get_mut(chave)?;
        item.1 = quadro;
        Some(&item.0)
    }

    /// A maior largura guardada deste hash.
    fn outra_largura(&mut self, hash: &str, quadro: u64) -> Option<&T> {
        let (_, item) = self.itens.iter_mut().filter(|(c, _)| c.0 == hash).max_by_key(|(c, _)| c.1)?;
        item.1 = quadro;
        Some(&item.0)
    }

    fn cabe(&self, quadro: u64) -> bool {
        self.itens.len() < self.teto || self.itens.values().any(|(_, uso)| uso.saturating_add(1) < quadro)
    }

    fn por(&mut self, chave: Chave, valor: T, quadro: u64) {
        if !self.itens.contains_key(&chave) && self.itens.len() >= self.teto {
            if let Some(velha) = self.itens.iter().min_by_key(|(_, (_, uso))| *uso).map(|(c, _)| c.clone()) {
                self.itens.remove(&velha);
            }
        }
        self.itens.insert(chave, (valor, quadro));
    }
}

fn rasterizar(tipo: &str, bytes: &[u8], largura: u32, lado: u32) -> Option<ColorImage> {
    if tipo == "svg" {
        return rasterizar_svg(bytes, largura, lado);
    }
    rasterizar_imagem(bytes, largura, lado)
}

/// O tamanho final: a largura pedida, sem passar de `lado` em nenhuma direção e mantendo a proporção.
fn encaixar(l: f32, a: f32, largura: u32, lado: u32) -> (u32, u32) {
    let w = largura.min(lado).max(1);
    let h = (w as f32 * a / l).round().max(1.0);
    if h <= lado as f32 {
        return (w, h as u32);
    }
    (((lado as f32 * l / a).round() as u32).max(1), lado)
}

fn fontes() -> Arc<fontdb::Database> {
    static BASE: OnceLock<Arc<fontdb::Database>> = OnceLock::new();
    BASE.get_or_init(|| {
        let mut db = fontdb::Database::new();
        for fonte in [REGULAR, MEDIO, NEGRITO] {
            db.load_font_source(fontdb::Source::Binary(Arc::new(fonte)));
        }
        // toda família genérica cai na nossa; a família com nome que não existe cai em `serif`, portanto também
        db.set_serif_family(FAMILIA);
        db.set_sans_serif_family(FAMILIA);
        db.set_monospace_family(FAMILIA);
        db.set_cursive_family(FAMILIA);
        db.set_fantasy_family(FAMILIA);
        Arc::new(db)
    })
    .clone()
}

fn rasterizar_svg(bytes: &[u8], largura: u32, lado: u32) -> Option<ColorImage> {
    let opcoes = usvg::Options {
        font_family: FAMILIA.to_string(),
        fontdb: fontes(),
        // o servidor já recusa `<image>`; aqui, mesmo que passe, nada é lido do disco
        image_href_resolver: ImageHrefResolver { resolve_data: Box::new(|_: &str, _: Arc<Vec<u8>>, _: &usvg::Options| None), resolve_string: Box::new(|_: &str, _: &usvg::Options| None) },
        ..usvg::Options::default()
    };
    let arvore = usvg::Tree::from_data(bytes, &opcoes).ok()?;
    let medida = arvore.size();
    let (w, h) = encaixar(medida.width(), medida.height(), largura, lado);
    let mut tela = tiny_skia::Pixmap::new(w, h)?;
    resvg::render(&arvore, tiny_skia::Transform::from_scale(w as f32 / medida.width(), h as f32 / medida.height()), &mut tela.as_mut());
    Some(ColorImage::from_rgba_premultiplied([w as usize, h as usize], tela.data()))
}

/// PNG, JPG e WebP. O leitor do `image` traz limite de memória: um arquivo pequeno que abre em gigabytes dá erro, não trava.
#[cfg(not(target_arch = "wasm32"))]
fn rasterizar_imagem(bytes: &[u8], largura: u32, lado: u32) -> Option<ColorImage> {
    let img = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().ok()?.decode().ok()?;
    let (ow, oh) = (img.width(), img.height());
    if ow == 0 || oh == 0 {
        return None;
    }
    // só reduz: ampliar quem faz é a textura
    let (w, h) = encaixar(ow as f32, oh as f32, largura.min(ow), lado);
    let img = if (w, h) == (ow, oh) { img } else { img.resize_exact(w, h, image::imageops::FilterType::Triangle) };
    let rgba = img.to_rgba8();
    Some(ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw()))
}

#[cfg(target_arch = "wasm32")]
fn rasterizar_imagem(_: &[u8], _: u32, _: u32) -> Option<ColorImage> {
    None
}
