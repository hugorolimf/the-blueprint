//! O que a tela de agentes pede ao orquestrador e a folhinha `Configurar`, com os comandos. O cartão de destaque dele é `agentes_orq.rs`.
//! A figura é a `principal`, para não se confundir com os agentes de plano: `parado` dormindo, `trabalhando` acordado. Só desenha e devolve o pedido: quem manda o gesto é `agentes::tela`.

use super::caixinha::botao_mini;
use super::linha::{Letra, Pincel};
use super::modelo::Config;
use crate::fonte::{familia, Peso};
use crate::i18n::tr;
use crate::planta::{self, botao, Botao};
use crate::tema::{BAIXO, CAMPO, CARTAO, MEDIO, TINTA};
use egui::load::Bytes;
use egui::{Color32, Context, FontId, Id, Image, Rect, Sense, Ui, UiBuilder};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex, OnceLock};

pub const TEXTOS: &[(&str, &str)] = &[
    ("Orquestrador", "Orchestrator"),
    ("Mestre do workspace {nome}", "Master of workspace {nome}"),
    ("Coordenando 1 agente", "Coordinating 1 agent"),
    ("Coordenando {n} agentes", "Coordinating {n} agents"),
    ("Ligado", "On"),
    ("Desligado", "Off"),
    ("Dormindo. Acorda quando um plano termina, pede ajuda ou quando você manda uma mensagem.", "Asleep. Wakes up when a plan finishes, asks for help, or when you send a message."),
    ("Ligue, ou mande uma mensagem: mandar também liga.", "Turn it on, or send a message: sending also turns it on."),
    ("Agora", "Now"),
    ("Nada acontecendo agora.", "Nothing going on right now."),
    ("Pedir ao orquestrador…", "Ask the orchestrator…"),
    ("Enviar", "Send"),
    ("Ligar", "Turn on"),
    ("Desligar", "Turn off"),
    ("Configurar", "Configure"),
    ("Comando dos agentes", "Agent command"),
    ("Comando do orquestrador", "Orchestrator command"),
    ("Um argumento por linha, sem shell. Trocar o comando do orquestrador pode devolver a ele ferramentas que hoje ficam de fora.", "One argument per line, no shell. Changing the orchestrator command can give it back tools it does not have today."),
    ("Salvar", "Save"),
    ("Fechar", "Close"),
];

/// A rota de `gesto/` e o corpo que uma tela pede. Quem desenha só devolve; `agentes::tela` manda.
pub type Pedido = (&'static str, Value);

pub const ALTURA_CONFIG: f32 = 188.0;
const ALTO_MINI: f32 = 24.0;

/// Pinta uma figura do Claude, embutida no programa e aberta uma só vez. Se não abrir, fica vazia, e a situação continua escrita ao lado.
pub fn figura_svg(ui: &Ui, r: Rect, nome: &'static str, gz: &'static [u8]) {
    figura_svg_alfa(ui, r, nome, gz, 1.0);
}

/// A mesma figura, mais apagada por `alfa` (0 a 1): entrando em cena ou dormindo.
pub fn figura_svg_alfa(ui: &Ui, r: Rect, nome: &'static str, gz: &'static [u8], alfa: f32) {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, Arc<[u8]>>>> = OnceLock::new();
    let Ok(mut cache) = CACHE.get_or_init(Default::default).lock() else { return };
    let svg = cache.entry(nome).or_insert_with(|| {
        let mut bytes = Vec::new();
        let _ = flate2::read::GzDecoder::new(gz).read_to_end(&mut bytes);
        bytes.into()
    });
    let opaco = Color32::from_white_alpha((alfa.clamp(0.0, 1.0) * 255.0).round() as u8);
    Image::from_bytes(format!("bytes://plano-{nome}.svg"), Bytes::Shared(svg.clone())).fit_to_exact_size(r.size()).tint(opaco).paint_at(ui, r);
}

pub fn corpo_ligado(ws: &str, ligado: bool) -> Value {
    json!({"workspaceId": ws, "ligado": ligado})
}

/// O corpo da mensagem, ou `None` com o campo vazio. Mandar mensagem também liga (no servidor).
pub fn corpo_mensagem(ws: &str, bp: &str, texto: &str) -> Option<Value> {
    let texto = texto.trim();
    (!texto.is_empty()).then(|| json!({"workspaceId": ws, "blueprintId": bp, "mensagem": texto}))
}

/// Um argumento por linha; linha em branco não conta.
pub fn argumentos(texto: &str) -> Vec<String> {
    texto.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect()
}

/// O corpo de `gesto/comando`: só o que mudou e não ficou vazio (lista vazia o servidor recusa). `None`: nada a mandar.
pub fn corpo_comando(agente: &str, orquestrador: &str, atual: &Config) -> Option<Value> {
    let mut corpo = Map::new();
    for (chave, texto, antes) in [("agente", agente, &atual.comando_agente), ("orquestrador", orquestrador, &atual.comando_orquestrador)] {
        let novos = argumentos(texto);
        if !novos.is_empty() && novos != *antes {
            corpo.insert(chave.to_string(), json!(novos));
        }
    }
    (!corpo.is_empty()).then_some(Value::Object(corpo))
}

fn id_folhinha() -> Id {
    Id::unique("plano-agentes-configurar")
}

pub fn folhinha_aberta(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(id_folhinha())).unwrap_or(false)
}

/// Abrir sempre começa do que o servidor tem: o rascunho anterior é descartado.
pub fn alternar_folhinha(ctx: &Context) {
    let aberta = !folhinha_aberta(ctx);
    ctx.data_mut(|d| {
        d.insert_temp(id_folhinha(), aberta);
        d.remove::<(String, String)>(id_folhinha().with("textos"));
    });
}

/// Um campo de várias linhas, com rolagem própria: o comando padrão do orquestrador tem mais linhas do que cabem.
fn campo_comando(ui: &mut Ui, c: &Pincel, rotulo: &'static str, texto: &mut String, x: f32, y: f32, w: f32) {
    c.rotulo(tr(rotulo), x, y, Letra::nova(9.5, MEDIO).trilha(0.1));
    let r = c.caixa(x, y + 8.0, w, 92.0);
    c.p.rect_filled(r, 0, CAMPO);
    planta::borda(c.p, r, BAIXO);
    let dentro = r.shrink(8.0);
    ui.scope_builder(UiBuilder::new().max_rect(dentro), |ui| {
        egui::ScrollArea::vertical().id_salt(("plano-comando", rotulo)).max_height(dentro.height()).auto_shrink([false, false]).show(ui, |ui| {
            let editor = egui::TextEdit::multiline(texto).frame(egui::Frame::NONE).margin(egui::Margin::ZERO).font(FontId::new(11.0, familia(Peso::Normal))).text_color(TINTA).desired_width(dentro.width() - 12.0).desired_rows(5);
            ui.add(editor);
        });
    });
}

/// A folhinha `Configurar`: os dois comandos, um argumento por linha. Enquanto `planejar/config` não chega, os campos ficam vazios e `Salvar` não manda nada.
pub fn folhinha(ui: &mut Ui, c: &Pincel, config: Option<&Config>, x0: f32, y: f32, x1: f32) -> Option<Pedido> {
    let ctx = ui.ctx().clone();
    let id = id_folhinha().with("textos");
    let vindos = || config.map(|k| (k.comando_agente.join("\n"), k.comando_orquestrador.join("\n")));
    let guardados = ctx.data(|d| d.get_temp::<(String, String)>(id));
    let mut textos = guardados.or_else(vindos).unwrap_or_default();
    c.ret(x0, y, x1 - x0, ALTURA_CONFIG, Some(CARTAO), Some((TINTA, 1.0)));
    planta::canto(c.p, c.caixa(x0, y, x1 - x0, ALTURA_CONFIG));
    c.rotulo(tr("Configurar"), x0 + 14.0, y + 20.0, Letra::nova(10.0, TINTA).trilha(0.12));
    let w = (x1 - x0 - 42.0) / 2.0;
    campo_comando(ui, c, "Comando dos agentes", &mut textos.0, x0 + 14.0, y + 40.0, w);
    campo_comando(ui, c, "Comando do orquestrador", &mut textos.1, x0 + 28.0 + w, y + 40.0, w);
    let (salvar, fechar) = (Botao { cheio: true, ativo: config.is_some(), ..botao_mini(tr("Salvar")) }, botao_mini(tr("Fechar")));
    let (ls, lf) = (salvar.largura(c.p), fechar.largura(c.p));
    let y_botoes = y + ALTURA_CONFIG - ALTO_MINI - 8.0;
    let letra = Letra::nova(10.0, MEDIO);
    let aviso = tr("Um argumento por linha, sem shell. Trocar o comando do orquestrador pode devolver a ele ferramentas que hoje ficam de fora.");
    let (t, cortou) = c.cortado(aviso, x1 - x0 - 42.0 - ls - lf, &letra);
    let largura = c.texto(&t, x0 + 14.0, y_botoes + 16.0, letra);
    if cortou {
        let _ = ui.interact(c.caixa(x0 + 14.0, y_botoes + 4.0, largura, 16.0), id.with("aviso"), Sense::hover()).on_hover_text(aviso);
    }
    let fechou = botao(ui, c.pt(x1 - 14.0 - ls - 6.0 - lf, y_botoes), &fechar).clicked();
    let salvou = botao(ui, c.pt(x1 - 14.0 - ls, y_botoes), &salvar).clicked();
    let pedido = config.filter(|_| salvou).and_then(|k| corpo_comando(&textos.0, &textos.1, k)).map(|corpo| ("comando", corpo));
    if config.is_some() {
        ctx.data_mut(|d| d.insert_temp(id, textos));
    }
    if fechou || salvou {
        alternar_folhinha(&ctx);
    }
    pedido
}
