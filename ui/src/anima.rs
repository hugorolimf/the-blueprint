//! A abertura de cada tela, como na página: os textos se escrevem (`escrever` de web/js/navegacao.js) e as barras, a régua e as
//! peças das ações se desenham (`desenharBarras` de web/js/barras.js). O relógio começa quando a tela muda; quem desenha pergunta
//! aqui o quanto da sua peça já apareceu. Fora da janela, ou depois de terminar, tudo aparece pronto e nada fica repintando.

use egui::{Context, Id, Rect};
use std::hash::{Hash, Hasher};

/// A abertura em curso: milissegundos desde que a tela abriu, a semente do sorteio desta abertura e a área visível (a janela).
#[derive(Clone, Copy)]
pub struct Abertura {
    pub ms: f32,
    semente: u64,
    topo: f32,
    altura: f32,
}

/// Nada da abertura passa disso: o último texto (700 + 330 + 650 ms) e a última barra (1550 ms).
const DURACAO: f32 = 2000.0;

/// `cubic-bezier(.2,.6,.3,1)`: o texto chega devagar ao opaco.
pub const CHEGA: [f32; 4] = [0.2, 0.6, 0.3, 1.0];
/// `cubic-bezier(.45,.05,.2,1)`: as peças das barras.
pub const BARRA: [f32; 4] = [0.45, 0.05, 0.2, 1.0];

fn chave_estado() -> Id {
    Id::unique("anima-abertura-estado")
}

fn chave_atual() -> Id {
    Id::unique("anima-abertura")
}

/// Liga a abertura para o desenho deste quadro. `tela` identifica a tela: quando muda, o relógio recomeça. `janela`: a área visível.
pub fn comecar(ctx: &Context, tela: &str, janela: Rect) {
    let agora = ctx.input(|i| i.time);
    let antes: Option<(String, f64, u64)> = ctx.data(|d| d.get_temp(chave_estado()));
    let (inicio, semente) = match antes {
        Some((t, inicio, semente)) if t == tela => (inicio, semente),
        _ => {
            let semente = (agora.to_bits() ^ 0x9E37_79B9_7F4A_7C15).rotate_left(17);
            ctx.data_mut(|d| d.insert_temp(chave_estado(), (tela.to_string(), agora, semente)));
            (agora, semente)
        }
    };
    let ms = ((agora - inicio) * 1000.0) as f32;
    let atual = (ms < DURACAO && !desligada()).then_some(Abertura { ms, semente, topo: janela.min.y, altura: janela.height().max(1.0) });
    ctx.data_mut(|d| d.insert_temp(chave_atual(), atual));
    if atual.is_some() {
        ctx.request_repaint();
    }
}

/// Desliga até a próxima tela: o que é desenhado depois (diálogos, menus) aparece sem abertura.
pub fn terminar(ctx: &Context) {
    ctx.data_mut(|d| d.insert_temp::<Option<Abertura>>(chave_atual(), None));
}

pub fn atual(ctx: &Context) -> Option<Abertura> {
    ctx.data(|d| d.get_temp::<Option<Abertura>>(chave_atual())).flatten()
}

/// `BLUEPRINT_UI_SEM_ANIMACAO=1`: a tela aparece pronta (o «movimento reduzido» da página).
fn desligada() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return std::env::var_os("BLUEPRINT_UI_SEM_ANIMACAO").is_some();
    #[cfg(target_arch = "wasm32")]
    false
}

/// Recomeça a abertura da tela atual (trocar o período reescreve a folha, como `desenrolar` depois do filtro em dados.js).
pub fn reiniciar(ctx: &Context) {
    ctx.data_mut(|d| d.remove::<(String, f64, u64)>(chave_estado()));
}

/// O que o usuário acabou de fazer mexe na folha: pelos próximos instantes, o que mudar de lugar anda até o novo em `dur` ms
/// (`organizarFolha` de barras.js) e o que nasce aparece aos poucos (`revelarEntrada` de expandir.js). Fora disso, tudo muda na hora.
pub fn mexer(ctx: &Context, dur: f32) {
    let agora = ctx.input(|i| i.time);
    ctx.data_mut(|d| d.insert_temp(Id::unique("anima-mexer"), (agora + 0.3, dur)));
    ctx.request_repaint();
}

#[derive(Clone)]
struct Trilho {
    de: Vec<f32>,
    para: Vec<f32>,
    inicio: f64,
    dur: f32,
    /// quando nasceu durante uma mudança (aparece aos poucos)
    nasceu: Option<f64>,
    /// o último quadro em que foi desenhado: quem sumiu e voltou é novo, não anda do lugar antigo
    quadro: u64,
}

/// Os valores de `chave` (posição, largura…) neste quadro: iguais ao `alvo`, ou a caminho dele se o alvo mudou logo depois de um
/// [`mexer`]. Devolve também a opacidade de quem nasceu numa mudança (220 ms).
pub fn deslize(ctx: &Context, chave: impl Hash + std::fmt::Debug, alvo: &[f32]) -> (Vec<f32>, f32) {
    let (agora, id) = (ctx.input(|i| i.time), Id::unique(("anima-deslize", chave)));
    let mexendo: Option<(f64, f32)> = ctx.data(|d| d.get_temp(Id::unique("anima-mexer"))).filter(|(ate, _)| agora <= *ate);
    let quadro = ctx.cumulative_frame_nr();
    let antes: Option<Trilho> = ctx.data(|d| d.get_temp::<Trilho>(id)).filter(|t| t.quadro + 1 >= quadro);
    let posicao = |t: &Trilho| -> Vec<f32> {
        let k = curva(BARRA, ((agora - t.inicio) as f32 * 1000.0) / t.dur.max(1.0));
        t.de.iter().zip(&t.para).map(|(a, b)| a + (b - a) * k).collect()
    };
    let trilho = match antes {
        None => Trilho { de: alvo.to_vec(), para: alvo.to_vec(), inicio: 0.0, dur: 1.0, nasceu: mexendo.map(|_| agora), quadro },
        Some(t) if t.para.len() == alvo.len() && t.para.iter().zip(alvo).all(|(a, b)| (a - b).abs() < 0.5) => t,
        Some(t) => match mexendo {
            Some((_, dur)) if t.para.len() == alvo.len() => Trilho { de: posicao(&t), para: alvo.to_vec(), inicio: agora, dur, nasceu: t.nasceu, quadro },
            _ => Trilho { de: alvo.to_vec(), para: alvo.to_vec(), inicio: 0.0, dur: 1.0, nasceu: None, quadro },
        },
    };
    let trilho = Trilho { quadro, ..trilho };
    let valores = posicao(&trilho);
    let alfa = trilho.nasceu.map_or(1.0, |n| curva(CHEGA, ((agora - n) as f32 * 1000.0) / 220.0));
    if alfa < 1.0 || ((agora - trilho.inicio) as f32 * 1000.0) < trilho.dur {
        ctx.request_repaint();
    }
    ctx.data_mut(|d| d.insert_temp(id, trilho));
    (valores, alfa)
}

/// A curva `cubic-bezier(x1, y1, x2, y2)` do CSS no instante `x` (0 a 1).
pub fn curva(c: [f32; 4], x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let bez = |a: f32, b: f32, t: f32| 3.0 * a * t * (1.0 - t).powi(2) + 3.0 * b * t * t * (1.0 - t) + t.powi(3);
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..24 {
        let meio = (lo + hi) / 2.0;
        if bez(c[0], c[2], meio) < x { lo = meio } else { hi = meio }
    }
    bez(c[1], c[3], (lo + hi) / 2.0)
}

impl Abertura {
    /// O quanto (0 a 1) da peça que começa em `atraso` e dura `dur` milissegundos já andou, na `curva`.
    pub fn passo(&self, atraso: f32, dur: f32, c: [f32; 4]) -> f32 {
        curva(c, (self.ms - atraso) / dur.max(1.0))
    }

    /// Um número de 0 a 1 sorteado para `coisa` nesta abertura (o mesmo em todos os quadros dela).
    fn sorteio(&self, coisa: impl Hash) -> f32 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (self.semente, coisa).hash(&mut h);
        (h.finish() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Visível agora: entre o topo e o pé da janela. O que está fora não espera a abertura.
    fn na_janela(&self, topo: f32, fundo: f32) -> bool {
        fundo >= self.topo && topo <= self.topo + self.altura
    }

    /// A vez da barra que tem o topo em `topo` (de cima para baixo, até 450 ms). `None` fora da janela.
    pub fn vez_da_barra(&self, topo: f32, fundo: f32) -> Option<f32> {
        self.na_janela(topo, fundo).then(|| (topo - self.topo).max(0.0) / self.altura * 450.0)
    }

    /// Quanto de cada terço do texto `s` (em `topo`..`fundo` na tela, começando em `x`) já se escreveu: cada terço chega numa ordem
    /// sorteada; com menos de 6 letras o texto chega inteiro (os três valores iguais). `None`: fora da janela, aparece pronto.
    pub fn escrita(&self, s: &str, x: f32, topo: f32, fundo: f32) -> Option<[f32; 3]> {
        if !self.na_janela(topo, fundo) {
            return None;
        }
        let u = (topo - self.topo).max(0.0) / self.altura * 0.7 + self.sorteio((s, x.to_bits(), topo.to_bits())) * 0.3;
        let (ini, dur) = (700.0 * u * u, 200.0 + 450.0 * u);
        if s.chars().count() < 6 {
            let a = self.passo(ini, dur, CHEGA);
            return Some([a; 3]);
        }
        let mut ordem = [0.0f32, 1.0, 2.0];
        for i in (1..3).rev() {
            let j = (self.sorteio((s, topo.to_bits(), i)) * (i + 1) as f32) as usize;
            ordem.swap(i, j.min(i));
        }
        Some(std::array::from_fn(|k| {
            let atraso = ini + (ordem[k] + self.sorteio((s, x.to_bits(), k))) * 55.0 * (1.0 + u);
            self.passo(atraso, dur, CHEGA)
        }))
    }
}

/// Os cortes do texto em três (`Math.round(n / 3)` e `Math.round(2n / 3)`), em letras.
pub fn tercos(n: usize) -> [usize; 4] {
    [0, (n as f32 / 3.0).round() as usize, (2.0 * n as f32 / 3.0).round() as usize, n]
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn a_curva_vai_de_zero_a_um_e_chega_devagar() {
        assert!(curva(CHEGA, 0.0).abs() < 1e-3);
        assert!((curva(CHEGA, 1.0) - 1.0).abs() < 1e-3);
        assert!(curva(CHEGA, 0.5) > 0.7, "a chegada é rápida no começo e lenta no fim");
        assert!(curva(BARRA, 0.2) < 0.2, "a barra começa devagar");
    }

    #[test]
    fn o_texto_de_cima_chega_antes_e_tudo_termina() {
        let a = |ms| Abertura { ms, semente: 7, topo: 0.0, altura: 1000.0 };
        let cima = a(150.0).escrita("BLUEPRINTS", 70.0, 40.0, 52.0).unwrap();
        let baixo = a(150.0).escrita("BLUEPRINTS", 70.0, 900.0, 912.0).unwrap();
        assert!(cima.iter().sum::<f32>() > baixo.iter().sum::<f32>());
        assert_eq!(a(1990.0).escrita("BLUEPRINTS", 70.0, 990.0, 1002.0), Some([1.0; 3]));
        assert!(a(0.0).escrita("FORA", 0.0, 1200.0, 1210.0).is_none(), "fora da janela aparece pronto");
        assert_eq!(tercos(10), [0, 3, 7, 10]);
    }
    #[test]
    fn o_que_muda_depois_de_mexer_anda_e_o_resto_pula() {
        let ctx = Context::default();
        let quadro = |t: f64, alvo: f32, mexe: bool| {
            let mut v = 0.0;
            let saida = ctx.run_pass(egui::RawInput { time: Some(t), ..Default::default() }, |c| {
                if mexe {
                    mexer(c, 300.0);
                }
                v = deslize(c, "a", &[alvo]).0[0];
            });
            saida.drop_without_applying_deltas();
            v
        };
        assert_eq!(quadro(0.0, 0.0, false), 0.0);
        assert_eq!(quadro(0.1, 50.0, false), 50.0, "sem mexer, pula");
        quadro(0.2, 50.0, true);
        let meio = quadro(0.35, 150.0, false);
        assert!(meio == 50.0, "o trilho começa no quadro em que o alvo muda: {meio}");
        let meio = quadro(0.45, 150.0, false);
        assert!(meio > 50.0 && meio < 150.0, "no meio do caminho: {meio}");
        assert_eq!(quadro(0.8, 150.0, false), 150.0);
    }
}
