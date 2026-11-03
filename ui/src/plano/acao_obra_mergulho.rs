//! O mergulho: clicar numa construção ou numa peça aproxima o olhar até ela, e o lado direito vira a ficha desse plano: de onde ele vem, o que libera, o que falta decidir, as telas e os checkpoints.
//! É só zoom, não um lugar fechado: a roda afasta e, longe o bastante, a ficha se recolhe (`obra_figura`). Também dá para andar pela obra pelas setas, pelos botões de dependência ou clicando nos vizinhos, e Esc sobe de volta.
//! Daqui também sai a vista expandida, que abre no próprio palco (`obra_explosao`), e o pouso de quem chega de outra ação.

use super::super::acao_movimento::{com_alfa, marcar_instante, mistura};
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Folha, Plano};
use super::super::revisao::Alvo;
use super::blocos::Bloco;
use super::decidir::{self, Item};
use super::figura::Marca;
use super::obra_fora as fora;
use super::voo::rolar;
use super::{explodida, ficha, mesa_acao, na_tela, obra_figura, obra_zoom, prancha, Saida};
use crate::i18n::tr;
use crate::planta::{botao, Botao};
use crate::tema::MEDIO;
use egui::{vec2, Align, Context, Id, Key, Painter, Pos2, Rect, ScrollArea, Sense, Ui, UiBuilder};

const ALTO_BOTAO: f32 = 30.0;
const VAO: f32 = 14.0;
const SEG_VOO: f32 = 0.7;
/// Vindo de outra tela ou de outra ação: dá tempo de ver o caminho enquanto a página anda.
const SEG_VOO_LONGE: f32 = 1.15;
const PAD_MESA: f32 = 16.0;
const LUGAR_DA_BARRA: f32 = 12.0;
/// O que fica acima do palco ao pousar: a linha da ação e o topo dela.
const ACIMA_DO_PALCO: f32 = 110.0;
/// Quanto a página leva para assentar, com um painel fechando e outro abrindo.
const SEG_ASSENTAR: f64 = 0.75;
/// Ao abrir um plano, a página para com a faixa de cima do palco a esta distância do alto da janela.
const ACIMA_DO_PLANO: f32 = 8.0;

fn id(a: &Acao, qual: &str) -> Id {
    Id::unique(("plano-obra-mergulho", qual, &a.referencia))
}

pub fn plano(ctx: &Context, a: &Acao) -> Option<String> {
    ctx.data(|d| d.get_temp::<String>(id(a, "plano"))).filter(|p| !p.is_empty())
}

/// De 0 (de fora) a 1 (lá dentro).
pub fn quanto(ctx: &Context, a: &Acao) -> f32 {
    mistura(ctx, ("plano-obra-mergulhou", &a.referencia), plano(ctx, a).is_some(), 0.7)
}

/// O plano de quem a mesa ainda está falando enquanto a câmera sobe.
pub fn ultimo(ctx: &Context, a: &Acao) -> Option<String> {
    ctx.data(|d| d.get_temp::<String>(id(a, "ultimo")))
}

pub fn entrar(ctx: &Context, a: &Acao, referencia: &str) {
    entrar_em(ctx, a, referencia, SEG_VOO);
}

pub fn entrar_de_longe(ctx: &Context, a: &Acao, referencia: &str) {
    entrar_em(ctx, a, referencia, SEG_VOO_LONGE);
}

fn entrar_em(ctx: &Context, a: &Acao, referencia: &str, seg: f32) {
    mesa_acao::fechar(ctx, a);
    let assenta = ctx.input(|i| i.time) + SEG_ASSENTAR;
    ctx.data_mut(|d| {
        d.insert_temp(id(a, "plano"), referencia.to_string());
        d.insert_temp(id(a, "ultimo"), referencia.to_string());
        d.remove_temp::<String>(id(a, "peca"));
        d.insert_temp(id(a, "voar"), seg);
        // a página anda até a vista do plano: o palco inteiro, com a ficha ao lado
        d.insert_temp(id(a, "ao-plano"), assenta);
    });
    marcar_instante(ctx, ("plano-obra-chegou", &a.referencia, referencia));
    crate::anima::mexer(ctx, 400.0);
}

/// Em quantos segundos o clique pediu para o olhar ir até lá. A pergunta consome o pedido.
pub fn voo_pedido(ctx: &Context, a: &Acao) -> Option<f32> {
    ctx.data_mut(|d| d.remove_temp::<f32>(id(a, "voar")))
}

/// Leva ao plano `r`: pela obra, se é desta ação; senão abre a ação dele, a página rola até lá e a laje voa do clique até a construção.
pub fn ir_ao_plano(ctx: &Context, f: &Folha, a: &Acao, r: &str, de: Pos2, completa: bool) -> Option<Saida> {
    let (dona, _) = fora::achar(f, r)?;
    if dona.referencia != a.referencia {
        obra_figura::ver_na_planta(ctx, dona, r, de, 8.0);
        pousar(ctx, dona);
        return Some(Saida::IrPara(dona.id.clone()));
    }
    if completa {
        expandir(ctx, a, r);
    } else {
        entrar(ctx, a, r);
    }
    None
}

/// Quem chega de outra ação pousa com a prancha no topo da janela. A página ainda está se mexendo, então o pedido vale até ela assentar.
pub fn pousar(ctx: &Context, a: &Acao) {
    let ate = ctx.input(|i| i.time) + SEG_ASSENTAR;
    ctx.data_mut(|d| d.insert_temp(id(a, "pousar"), ate));
}

/// Enquanto o pouso vale, mantém o palco de topo `y` no alto da janela: quem chega de outra ação vê também a linha dela; quem abre um plano vê o palco inteiro.
pub fn assentar(ui: &Ui, c: &Pincel, a: &Acao, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx();
    for (qual, acima) in [("pousar", ACIMA_DO_PALCO), ("ao-plano", ACIMA_DO_PLANO)] {
        let Some(ate) = ctx.data(|d| d.get_temp::<f64>(id(a, qual))) else { continue };
        if ctx.input(|i| i.time) < ate {
            rolar(ui, c.caixa(x, y - acima, w, 1.0), Some(Align::Min));
            ctx.request_repaint();
            return;
        }
        ctx.data_mut(|d| d.remove_temp::<f64>(id(a, qual)));
    }
}

/// Abre a vista expandida deste plano, no próprio palco.
pub fn expandir(ctx: &Context, a: &Acao, referencia: &str) {
    ctx.data_mut(|d| d.insert_temp(id(a, "expandida"), referencia.to_string()));
    crate::anima::mexer(ctx, 400.0);
}

pub fn expandida(ctx: &Context, a: &Acao) -> Option<String> {
    ctx.data(|d| d.get_temp::<String>(id(a, "expandida"))).filter(|p| !p.is_empty())
}

/// De volta da vista expandida para a planta.
pub fn recolher(ctx: &Context, a: &Acao) {
    ctx.data_mut(|d| d.remove_temp::<String>(id(a, "expandida")));
    crate::anima::mexer(ctx, 400.0);
}

/// A peça que a pessoa escolheu, não a que abriu sozinha.
pub fn escolhida(ctx: &Context, a: &Acao) -> Option<String> {
    ctx.data(|d| d.get_temp::<String>(id(a, "peca")))
}

/// A roda afastou o bastante: a ficha se recolhe e o olhar fica onde a pessoa deixou.
pub fn soltar(ctx: &Context, a: &Acao) {
    ctx.data_mut(|d| d.remove_temp::<String>(id(a, "plano")));
    crate::anima::mexer(ctx, 400.0);
}

pub fn sair(ctx: &Context, a: &Acao) {
    ctx.data_mut(|d| d.remove_temp::<String>(id(a, "plano")));
    obra_zoom::zerar(ctx, obra_figura::id_zoom(a));
    crate::anima::mexer(ctx, 400.0);
}

pub fn escolher(ctx: &Context, a: &Acao, chave: &str) {
    ctx.data_mut(|d| {
        d.insert_temp(id(a, "peca"), chave.to_string());
        d.insert_temp(id(a, "voar"), SEG_VOO);
        d.insert_temp(id(a, "rolar"), true);
    });
    ctx.request_repaint();
}

/// Recolhe a peça aberta e o olhar volta à construção.
pub fn fechar_peca(ctx: &Context, a: &Acao) {
    escolher(ctx, a, "");
    ctx.data_mut(|d| d.remove_temp::<bool>(id(a, "rolar")));
}

/// A lista precisa rolar até a peça que acabou de abrir? A pergunta consome o pedido.
pub fn rolar_pedido(ctx: &Context, a: &Acao) -> bool {
    ctx.data_mut(|d| d.remove_temp::<bool>(id(a, "rolar"))).unwrap_or(false)
}

/// A peça que o mouse acendeu, no palco ou na lista: a outra metade acende junto.
pub fn acesa(ctx: &Context, a: &Acao) -> Option<String> {
    ctx.data(|d| d.get_temp::<Option<String>>(id(a, "acesa"))).flatten()
}

pub fn acender(ctx: &Context, a: &Acao, chave: Option<String>) {
    ctx.data_mut(|d| d.insert_temp(id(a, "acesa"), chave));
}

/// A peça aberta: a escolhida; senão a primeira que falta decidir; senão a primeira.
pub fn peca<'a>(ctx: &Context, a: &Acao, b: &Bloco<'a>) -> Option<Item<'a>> {
    let lista = decidir::todas(b);
    let guardada = ctx.data(|d| d.get_temp::<String>(id(a, "peca")));
    if guardada.as_deref() == Some("") {
        return None;
    }
    guardada.and_then(|g| lista.iter().find(|i| i.chave() == g).copied()).or_else(|| lista.iter().find(|i| i.marca() == Marca::Espera).copied()).or_else(|| lista.first().copied())
}

/// Esc sobe; as setas passam ao plano anterior e ao seguinte. Nada disso enquanto se escreve. Dentro de uma tela, Esc só sai dela, e as setas são da barra dela.
pub fn teclado(ctx: &Context, a: &Acao, planos: &[&Plano]) {
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    let esc = ctx.input(|i| i.key_pressed(Key::Escape));
    if na_tela::aberta(ctx, a).is_some() {
        if esc {
            na_tela::sair(ctx, a);
        }
        return;
    }
    if expandida(ctx, a).is_some() {
        if esc {
            recolher(ctx, a);
        }
        return;
    }
    if esc && mesa_acao::aberta(ctx, a) {
        mesa_acao::fechar(ctx, a);
    }
    let Some(atual) = plano(ctx, a) else { return };
    if esc {
        sair(ctx, a);
    }
    let aqui = planos.iter().position(|q| q.referencia == atual).unwrap_or(0);
    if ctx.input(|i| i.key_pressed(Key::ArrowRight)) && aqui + 1 < planos.len() {
        entrar(ctx, a, &planos[aqui + 1].referencia);
    }
    if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) && aqui > 0 {
        entrar(ctx, a, &planos[aqui - 1].referencia);
    }
}

/// A coluna é a mesma no mergulho e na vista expandida: na vista expandida só o detalhe cresce, com as tabelas do plano entrando junto com as camadas.
pub fn altura(p: &Painter, f: &Folha, a: &Acao, q: &Plano, b: &Bloco, w: f32) -> f32 {
    let completa = expandida(p.ctx(), a).as_deref() == Some(q.referencia.as_str());
    let tabelas = prancha::altura_tabelas(&b.tabelas) * explodida::separacao(p.ctx(), a, &q.referencia, completa);
    ALTO_BOTAO + VAO + ficha::altura(p, f, a, q, b, w, completa) + if tabelas > 0.0 { VAO + tabelas } else { 0.0 }
}

/// A coluna escrita direto na planta, sem caixa nem fio: quem abre o espaço é o sumiço do que está atrás dela (`obra_sumico`). O palco não muda de tamanho; ela chega deslizando, na altura dele, e rola por conta própria.
#[allow(clippy::too_many_arguments)]
pub fn flutuante<R>(ui: &mut Ui, _c: &Pincel, chave: &str, palco: Rect, mesa: Rect, t: f32, medir: impl FnOnce(&Painter, f32) -> f32, pintar: impl FnOnce(&mut Ui, &Pincel, f32) -> R) -> R {
    let recorte = palco.intersect(ui.clip_rect());
    let miolo = mesa.shrink2(vec2(PAD_MESA, 10.0));
    let mut filho = ui.new_child(UiBuilder::new().max_rect(miolo).id_salt(("plano-obra-mesa", chave)));
    filho.set_clip_rect(miolo.intersect(recorte));
    filho.set_opacity(ui.opacity() * t);
    let w = (miolo.width() - LUGAR_DA_BARRA).max(1.0);
    crate::rolagem::mostrar_compacta(&mut filho, ScrollArea::vertical().id_salt(("plano-obra-mesa-rola", chave)).auto_shrink([false, false]), |ui| {
        let alto = medir(ui.painter(), w);
        let (r, _) = ui.allocate_exact_size(vec2(w, alto), Sense::hover());
        let dentro = ui.painter().clone();
        pintar(ui, &Pincel { p: &dentro, o: r.min }, w)
    })
}

/// A mesa do plano por dentro: sair, a dica do próximo nível (ou, na vista expandida, voltar à construção) e, embaixo, a ficha.
#[allow(clippy::too_many_arguments)]
pub fn painel(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, q: &Plano, b: &Bloco, x: f32, y: f32, w: f32) -> Option<Saida> {
    let ctx = ui.ctx().clone();
    let completa = expandida(&ctx, a).as_deref() == Some(q.referencia.as_str());
    let sobe = Botao { texto: tr("← Sair da construção"), tam: 10.0, cheio: false, ativo: true, altura: ALTO_BOTAO, pad_x: 12.0 };
    // sair de dentro das camadas também as fecha: o palco acerta isso no quadro seguinte (`obra_explosao::acertar`)
    if botao(ui, c.pt(x, y), &sobe).clicked() {
        sair(&ctx, a);
    }
    if completa {
        let volta = Botao { texto: tr("← Voltar"), tam: 10.0, cheio: false, ativo: true, altura: ALTO_BOTAO, pad_x: 12.0 };
        if botao(ui, c.pt(x + sobe.largura(c.p) + 8.0, y), &volta).clicked() {
            recolher(&ctx, a);
        }
    } else {
        // a vista expandida não tem botão: é o próximo nível do zoom, com mais um clique ou mais roda na construção
        let dica = tr("Clique de novo ou role para ver as camadas");
        let largura = c.largura(dica, &Letra::nova(10.5, MEDIO));
        if largura + sobe.largura(c.p) + 16.0 <= w {
            c.texto(dica, x + w - largura, y + ALTO_BOTAO * 0.5 + 4.0, Letra::nova(10.5, MEDIO));
        }
    }
    let y_ficha = y + ALTO_BOTAO + VAO;
    let saida = ficha::desenhar(ui, c, alvo, f, a, q, b, x, y_ficha, w, completa);
    let k = explodida::separacao(&ctx, a, &q.referencia, completa);
    if k > 0.01 && !b.tabelas.is_empty() {
        let pintor = com_alfa(c.p, k);
        prancha::tabelas(ui, &Pincel { p: &pintor, o: c.o }, &b.tabelas, x, y_ficha + ficha::altura(c.p, f, a, q, b, w, completa) + VAO, w);
    }
    saida
}
