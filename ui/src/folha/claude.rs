//! A camada do Claude (`claude-camada.js`): uma figura por agente ativo ao lado do plano em que ele trabalha (ou da barra, ou do cabeçalho da seção, ou do cartão
//! da lista), e o destaque tracejado em volta da linha do plano. As figuras são os SVGs exportados de `agentes-claude.mjs` (primeiro quadro de cada pose).

use super::cena::Cena;
use crate::agentes::{self, Agente, Lugar, ALTURA, LARGURA};
use crate::tema::{ERRO, TINTA};
use egui::{vec2, Image, Rect, Ui};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex, OnceLock};

/// Onde cada lugar da folha está desenhado, em coordenadas da página `(x, y, w, h)`: as seções e o cartão preenchem enquanto desenham.
pub type Ancoras = RefCell<HashMap<Lugar, (f32, f32, f32, f32)>>;

const FIGURAS: &[(&str, &[u8])] = &[
    ("principal-parado", include_bytes!("../../assets/claude/principal-parado.svg.gz")),
    ("principal-andando", include_bytes!("../../assets/claude/principal-andando.svg.gz")),
    ("principal-trabalhando", include_bytes!("../../assets/claude/principal-trabalhando.svg.gz")),
    ("principal-esperando", include_bytes!("../../assets/claude/principal-esperando.svg.gz")),
    ("principal-bloqueado", include_bytes!("../../assets/claude/principal-bloqueado.svg.gz")),
    ("principal-concluido", include_bytes!("../../assets/claude/principal-concluido.svg.gz")),
    ("explorador-parado", include_bytes!("../../assets/claude/explorador-parado.svg.gz")),
    ("explorador-andando", include_bytes!("../../assets/claude/explorador-andando.svg.gz")),
    ("explorador-trabalhando", include_bytes!("../../assets/claude/explorador-trabalhando.svg.gz")),
    ("explorador-esperando", include_bytes!("../../assets/claude/explorador-esperando.svg.gz")),
    ("explorador-bloqueado", include_bytes!("../../assets/claude/explorador-bloqueado.svg.gz")),
    ("explorador-concluido", include_bytes!("../../assets/claude/explorador-concluido.svg.gz")),
    ("construtor-parado", include_bytes!("../../assets/claude/construtor-parado.svg.gz")),
    ("construtor-andando", include_bytes!("../../assets/claude/construtor-andando.svg.gz")),
    ("construtor-trabalhando", include_bytes!("../../assets/claude/construtor-trabalhando.svg.gz")),
    ("construtor-esperando", include_bytes!("../../assets/claude/construtor-esperando.svg.gz")),
    ("construtor-bloqueado", include_bytes!("../../assets/claude/construtor-bloqueado.svg.gz")),
    ("construtor-concluido", include_bytes!("../../assets/claude/construtor-concluido.svg.gz")),
    ("revisor-parado", include_bytes!("../../assets/claude/revisor-parado.svg.gz")),
    ("revisor-andando", include_bytes!("../../assets/claude/revisor-andando.svg.gz")),
    ("revisor-trabalhando", include_bytes!("../../assets/claude/revisor-trabalhando.svg.gz")),
    ("revisor-esperando", include_bytes!("../../assets/claude/revisor-esperando.svg.gz")),
    ("revisor-bloqueado", include_bytes!("../../assets/claude/revisor-bloqueado.svg.gz")),
    ("revisor-concluido", include_bytes!("../../assets/claude/revisor-concluido.svg.gz")),
];

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

/// Desenha o destaque dos planos e as figuras dos agentes do blueprint. `tipo_cab`: `cab` (cabeçalho do blueprint) ou `cartao` (cartão da lista).
pub fn desenhar(c: &Cena, ui: &Ui, ags: &[Agente], ancoras: &Ancoras, tipo_cab: &str) {
    if ags.is_empty() {
        return;
    }
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
            if estado == Some("concluido") || estado == Some("bloqueado") {
                c.moldura(x, y, w, h, cor, 1.0, false);
            } else {
                c.moldura(x, y, w, h, cor, 1.0, true);
            }
        }
    }
    // da última para a primeira: a mais perto do lugar (o principal) fica por cima
    for it in itens.iter().rev() {
        let Some(&ancora) = mapa.get(&it.lugar) else { continue };
        let tipo = match it.lugar {
            Lugar::Plano(_) | Lugar::Barra(_) => "plano",
            _ => tipo_cab,
        };
        let (x, y) = agentes::posicao(tipo, ancora, it.indice, &larguras[&it.lugar]);
        let nome = format!("{}-{}", it.papel.unwrap_or("principal"), agentes::pose(agentes::estado(&it.agente)));
        if let Some(bytes) = svg(&nome) {
            let r = Rect::from_min_size(c.pt(x, y), vec2(LARGURA, ALTURA));
            Image::from_bytes(format!("bytes://claude-{nome}.svg"), egui::load::Bytes::Shared(bytes)).fit_to_exact_size(r.size()).paint_at(ui, r);
        }
    }
}
