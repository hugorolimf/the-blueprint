//! A camada do Claude (`claude-camada.js`): uma figura por agente ativo ao lado do plano em que ele trabalha (ou da barra, ou do cabeçalho da seção, ou do cartão
//! da lista), e o destaque tracejado em volta da linha do plano. As figuras são os SVGs exportados de `agentes-claude.mjs` (primeiro quadro de cada pose).

use super::cena::Cena;
use super::figuras::{FIGURAS, SEQUENCIAS};
use super::movimento::{self, Alvo};
use crate::agentes::{self, Agente, Lugar, ALTURA, LARGURA};
use crate::tema::{ERRO, TINTA};
use egui::{vec2, Image, Rect, Ui};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex, OnceLock};

/// Onde cada lugar da folha está desenhado, em coordenadas da página `(x, y, w, h)`: as seções e o cartão preenchem enquanto desenham.
pub type Ancoras = RefCell<HashMap<Lugar, (f32, f32, f32, f32)>>;

fn svg(nome: &str) -> Option<Arc<[u8]>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<[u8]>>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().ok()?;
    if let Some(s) = cache.get(nome) {
        return Some(s.clone());
    }
    let (_, gz) = FIGURAS.iter().find(|(n, _)| *n == nome)?;
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(*gz).read_to_end(&mut bytes).ok()?;
    let bytes: Arc<[u8]> = bytes.into();
    cache.insert(nome.to_string(), bytes.clone());
    Some(bytes)
}

/// Desenha o destaque dos planos e as figuras dos agentes do blueprint `escopo` na `tela`. `tipo_cab`: `cab` (cabeçalho do blueprint) ou `cartao` (cartão da lista).
/// Devolve o nome do agente cuja figura foi clicada.
pub fn desenhar(c: &Cena, ui: &Ui, escopo: &str, tela: &str, ags: &[Agente], ancoras: &Ancoras, tipo_cab: &str) -> Option<String> {
    let mapa = ancoras.borrow();
    let itens = agentes::itens(ags, |id| mapa.contains_key(&Lugar::Plano(id.into())), |id| mapa.contains_key(&Lugar::Barra(id.into())), |_| None);
    // as figuras de cada lugar, na ordem, para o passo entre elas
    let mut larguras: HashMap<&Lugar, Vec<f32>> = HashMap::new();
    for it in &itens {
        larguras.entry(&it.lugar).or_default().push(LARGURA);
    }
    let mut destacados = std::collections::HashSet::new();
    for it in &itens {
        let Some(&(x, y, w, h)) = mapa.get(&it.lugar) else { continue };
        if let (Lugar::Plano(_), true) = (&it.lugar, destacados.insert(it.lugar.clone())) {
            let estado = agentes::estado(&it.agente);
            let cor = if estado == Some("bloqueado") { ERRO } else { TINTA };
            c.retangulo(x, y, w, h, Some(egui::Color32::from_rgba_unmultiplied(0xea, 0xf4, 0xff, 0x14)), None);
            c.moldura(x, y, w, h, cor, 1.0, !matches!(estado, Some("concluido" | "bloqueado")));
        }
    }
    let alvos: Vec<Alvo> = itens.iter().filter_map(|it| {
        let &ancora = mapa.get(&it.lugar)?;
        let tipo = if matches!(it.lugar, Lugar::Plano(_) | Lugar::Barra(_)) { "plano" } else { tipo_cab };
        let (x, y) = agentes::posicao(tipo, ancora, it.indice, &larguras[&it.lugar]);
        Some(Alvo { agente: it.agente.agente.clone(), papel: it.papel, pose: agentes::pose(agentes::estado(&it.agente)), lugar: it.lugar.clone(), x, y })
    }).collect();
    let mut clicado = None;
    // da última para a primeira: a mais perto do lugar (o principal) fica por cima
    for d in movimento::passo(ui, escopo, tela, &alvos).iter().rev() {
        let nome = format!("{}-{}-{}", d.papel.unwrap_or("principal"), d.pose, quadro(ui, d.pose, &d.agente));
        let Some(bytes) = svg(&nome) else { continue };
        let cheio = Rect::from_min_size(c.pt(d.x, d.y), vec2(LARGURA, ALTURA));
        let r = Rect::from_center_size(cheio.center(), cheio.size() * d.escala);
        let imagem = Image::from_bytes(format!("bytes://claude-{nome}.svg"), egui::load::Bytes::Shared(bytes)).fit_to_exact_size(r.size());
        // a sombra da página (`drop-shadow(0 2px 3px)`): a própria figura, escura, 2 px abaixo; a opacidade acompanha o tamanho ao entrar e sair
        let alfa = |a: f32| egui::Color32::from_white_alpha((a * d.escala * 255.0) as u8);
        imagem.clone().tint(egui::Color32::from_rgba_unmultiplied(0x02, 0x13, 0x2e, (0x99 as f32 * d.escala) as u8)).paint_at(ui, r.translate(vec2(0.0, 2.0)));
        imagem.tint(alfa(1.0)).paint_at(ui, r);
        if d.vivo {
            let resposta = ui.interact(cheio, ui.scope_id().with(("claude", &d.agente)), egui::Sense::click());
            if resposta.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resposta.clicked() {
                clicado = Some(d.agente.clone());
            }
        }
    }
    clicado
}

/// O quadro da pose neste instante: cada figura anda no ritmo da sua pose, defasada pelo nome (como o relógio por figura da página). Marca o próximo quadro.
fn quadro(ui: &Ui, pose: &str, nome: &str) -> u8 {
    let Some((_, seq, ritmo)) = SEQUENCIAS.iter().find(|(p, _, _)| *p == pose) else { return 0 };
    let agora = crate::agentes::agora_ms().max(0) as u64;
    let defasagem = nome.bytes().fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(u64::from(b))) % (ritmo * seq.len() as u64);
    let passo = (agora + defasagem) / ritmo;
    ui.ctx().request_repaint_after(std::time::Duration::from_millis(ritmo - (agora + defasagem) % ritmo));
    seq[(passo % seq.len() as u64) as usize]
}
