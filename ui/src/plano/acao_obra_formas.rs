//! A forma de cada construção: o plano aparece como o que ele constrói. Uma tela é um monitor, os dados são um banco de discos, a integração são dois flanges ligados, e assim por diante.
//! O agente diz a forma no plano (`forma`); sem ela, sai de um palpite pela `area` e pelo texto (`acao_obra_palpite.rs`), para plano antigo também ter cara. Cada forma recebe a altura atual: é assim que sobe com o tempo.

use super::super::modelo::Plano;
use super::figura::LADO;
use crate::anatomia::{Malha, Ponta, V3};
use crate::i18n::tr;

#[path = "acao_obra_palpite.rs"]
mod palpite;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Forma {
    Padrao,
    Tela,
    Dados,
    Servico,
    Integracao,
    Ia,
    Teste,
    Documento,
    Infra,
}

/// O corpo leva o traço forte; o detalhe (janela, trilho, emenda), o traço fraco.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Papel {
    Corpo,
    Detalhe,
    /// a interface esboçada no vidro do monitor: no palco, o letreiro do plano toma o lugar dela
    Esboco,
}

pub type Partes = Vec<(Malha, Papel)>;

impl Forma {
    pub fn do_plano(q: &Plano) -> Forma {
        match q.forma.as_str() {
            "tela" => Forma::Tela,
            "dados" => Forma::Dados,
            "servico" => Forma::Servico,
            "integracao" => Forma::Integracao,
            "ia" => Forma::Ia,
            "teste" => Forma::Teste,
            "documento" => Forma::Documento,
            "infra" => Forma::Infra,
            _ => palpite::palpite(&q.area, &q.texto, &q.entrega),
        }
    }

    /// O nome dela, para a chamada da vista expandida.
    pub fn nome(self) -> &'static str {
        match self {
            Forma::Padrao => tr("Construção"),
            Forma::Tela => tr("Tela"),
            Forma::Dados => tr("Dados"),
            Forma::Servico => tr("Serviço"),
            Forma::Integracao => tr("Integração"),
            Forma::Ia => tr("Inteligência"),
            Forma::Teste => tr("Teste"),
            Forma::Documento => tr("Documento"),
            Forma::Infra => tr("Infraestrutura"),
        }
    }

    /// As peças, com o canto da construção em `(x, y)`, o chão em `z` e a altura `h`.
    pub fn partes(self, x: f32, y: f32, z: f32, h: f32) -> Partes {
        let h = h.max(0.02);
        match self {
            Forma::Padrao => padrao(x, y, z, h),
            Forma::Tela => tela(x, y, z, h),
            Forma::Dados => dados(x, y, z, h),
            Forma::Servico => servico(x, y, z, h),
            Forma::Integracao => integracao(x, y, z, h),
            Forma::Ia => ia(x, y, z, h),
            Forma::Teste => teste(x, y, z, h),
            Forma::Documento => documento(x, y, z, h),
            Forma::Infra => infra(x, y, z, h),
        }
    }
}

/// As janelas numa fachada: caixinhas finas, para sumirem atrás do prédio quando a câmera gira.
fn janelas(x: f32, y: f32, z: f32, h: f32, v: &mut Partes) {
    let andares = ((h - 0.3) / 0.95).floor().max(0.0) as usize;
    for a in 0..andares {
        let zz = z + 0.3 + a as f32 * 0.95;
        for u in [0.45, 2.2] {
            v.push((Malha::caixa([x + u, y + LADO, zz], [1.35, 0.04, 0.6]), Papel::Detalhe));
            v.push((Malha::caixa([x + LADO, y + u, zz], [0.04, 1.35, 0.6]), Papel::Detalhe));
        }
    }
}

fn padrao(x: f32, y: f32, z: f32, h: f32) -> Partes {
    let mut v = vec![(Malha::caixa([x, y, z], [LADO, LADO, h]), Papel::Corpo)];
    janelas(x, y, z, h, &mut v);
    v
}

/// O fundo do corpo do monitor e a altura dele a partir da qual há vidro.
const FUNDO_MONITOR: f32 = 0.45;
const ALTO_COM_VIDRO: f32 = 0.9;
const ESPESSURA_VIDRO: f32 = 0.03;

/// As medidas do monitor: o pé, onde o corpo começa atrás (`y0`) e embaixo (`z0`), e quanto ele sobe.
struct Monitor {
    pe: f32,
    y0: f32,
    z0: f32,
    alto: f32,
}

fn monitor(y: f32, z: f32, h: f32) -> Monitor {
    let pe = (h * 0.16).min(0.35);
    Monitor { pe, y0: y + 1.5, z0: z + pe + 0.25, alto: (h - pe - 0.25).max(0.02) }
}

/// Um monitor de pé: o pé, a haste e a tela virada para a frente, com a interface esboçada nela.
fn tela(x: f32, y: f32, z: f32, h: f32) -> Partes {
    let Monitor { pe, y0, z0, alto } = monitor(y, z, h);
    let mut v = vec![
        (Malha::laje([x + 0.9, y + 0.8, z], [LADO - 1.8, 2.3, pe * 0.45], 0.25), Papel::Corpo),
        (Malha::caixa([x + LADO * 0.5 - 0.25, y + 1.6, z], [0.5, 0.3, pe + 0.3]), Papel::Corpo),
        (Malha::laje([x + 0.05, y0, z0], [LADO - 0.1, FUNDO_MONITOR, alto], 0.12), Papel::Corpo),
    ];
    if alto > ALTO_COM_VIDRO {
        let face = y0 + FUNDO_MONITOR;
        v.push((Malha::caixa([x + 0.3, face, z0 + 0.2], [LADO - 0.6, ESPESSURA_VIDRO, alto - 0.4]), Papel::Detalhe));
        v.push((Malha::caixa([x + 0.5, face + ESPESSURA_VIDRO, z0 + alto - 0.55], [1.2, 0.02, 0.18]), Papel::Esboco));
        v.push((Malha::caixa([x + 0.5, face + ESPESSURA_VIDRO, z0 + 0.35], [LADO - 1.0, 0.02, (alto - 1.1).max(0.1)]), Papel::Esboco));
    }
    v
}

/// A frente do vidro do monitor da forma Tela, com a mesma conta de `tela`: `[topo esq, topo dir, base dir, base esq]`, para escrever nele. `None` enquanto o monitor é baixo demais para ter vidro.
pub fn vidro(x: f32, y: f32, z: f32, h: f32) -> Option<[V3; 4]> {
    let Monitor { y0, z0, alto, .. } = monitor(y, z, h.max(0.02));
    if alto <= ALTO_COM_VIDRO {
        return None;
    }
    let (f, baixo, topo, x0, x1) = (y0 + FUNDO_MONITOR + ESPESSURA_VIDRO, z0 + 0.2, z0 + alto - 0.2, x + 0.3, x + LADO - 0.3);
    Some([[x0, f, topo], [x1, f, topo], [x1, f, baixo], [x0, f, baixo]])
}

/// O banco de dados: discos empilhados, com a emenda entre eles.
fn dados(x: f32, y: f32, z: f32, h: f32) -> Partes {
    let (c, r) = ((x + LADO * 0.5, y + LADO * 0.5), LADO * 0.46);
    let n = 3;
    let alto = h / n as f32;
    let mut v = Vec::new();
    for i in 0..n {
        let zz = z + i as f32 * alto;
        v.push((Malha::cilindro(c, r, zz, alto - alto.min(0.1)), Papel::Corpo));
        v.push((Malha::cilindro(c, r * 0.9, zz + alto - alto.min(0.1), alto.min(0.1)), Papel::Detalhe));
    }
    v
}

/// O serviço: um gabinete de máquinas, com trilhos na frente e dois respiros em cima.
fn servico(x: f32, y: f32, z: f32, h: f32) -> Partes {
    let (m, w) = (0.3, LADO - 0.6);
    let mut v = vec![(Malha::caixa([x + m, y + m, z], [w, w, h]), Papel::Corpo)];
    let n = ((h - 0.3) / 0.42).floor().max(0.0) as usize;
    for i in 0..n {
        v.push((Malha::caixa([x + m + 0.25, y + m + w, z + 0.25 + i as f32 * 0.42], [w - 0.5, 0.04, 0.22]), Papel::Detalhe));
    }
    for (u, d) in [(1.2, 1.2), (2.6, 2.5)] {
        v.push((Malha::cilindro((x + u, y + d), 0.42, z + h, (h * 0.16).min(0.32)), Papel::Corpo));
    }
    v
}

/// A integração: dois flanges ligados por um tubo. Serve a API, a fila, o que liga um sistema ao outro.
fn integracao(x: f32, y: f32, z: f32, h: f32) -> Partes {
    let perfil = |h: f32| [(0.0, 1.0), (h * 0.1, 1.0), (h * 0.12, 0.62), (h * 0.85, 0.62), (h * 0.87, 0.95), (h, 0.95)];
    let (a, b) = ((x + 1.05, y + 1.05), (x + LADO - 1.05, y + LADO - 1.05));
    let meio = z + h * 0.5;
    vec![
        (Malha::torno(a, z, &perfil(h), Ponta::Tampa), Papel::Corpo),
        (Malha::viga([a.0, a.1, meio - 0.25], [b.0, b.1, meio - 0.25], 0.6, 0.5), Papel::Corpo),
        (Malha::torno(b, z, &perfil(h), Ponta::Tampa), Papel::Corpo),
    ]
}

/// A inteligência: uma esfera sobre o pedestal, como a lâmpada do kit.
fn ia(x: f32, y: f32, z: f32, h: f32) -> Partes {
    let c = (x + LADO * 0.5, y + LADO * 0.5);
    let raio = (h * 0.34).min(1.25);
    let pedestal = [(0.0, 1.6), (0.14 * h, 1.6), (0.2 * h, 0.75), ((h - raio * 1.7).max(0.21 * h), 0.5)];
    vec![
        (Malha::torno(c, z, &pedestal, Ponta::Tampa), Papel::Corpo),
        (Malha::esfera([c.0, c.1, z + h - raio], raio), Papel::Corpo),
    ]
}

/// O teste: o copo de laboratório, largo embaixo e de boca estreita.
fn teste(x: f32, y: f32, z: f32, h: f32) -> Partes {
    let c = (x + LADO * 0.5, y + LADO * 0.5);
    let perfil = [(0.0, 1.55), (h * 0.08, 1.7), (h * 0.52, 0.72), (h * 0.8, 0.55), (h * 0.84, 0.72), (h, 0.72)];
    vec![
        (Malha::torno(c, z, &perfil, Ponta::Tampa), Papel::Corpo),
        (Malha::cilindro(c, 1.05, z + h * 0.26, 0.03), Papel::Detalhe),
    ]
}

/// O documento: folhas empilhadas, cada uma um pouco fora do lugar.
fn documento(x: f32, y: f32, z: f32, h: f32) -> Partes {
    let n = ((h / 0.34).floor() as usize).clamp(1, 7);
    let alto = h / n as f32;
    (0..n).map(|i| {
        let d = if i % 2 == 0 { 0.08 } else { -0.08 };
        (Malha::laje([x + 0.45 + d, y + 0.3 - d, z + i as f32 * alto], [LADO - 0.9, LADO - 0.6, alto * 0.72], 0.05), Papel::Corpo)
    }).collect()
}

/// A infraestrutura: uma torre em degraus, com a antena em cima.
fn infra(x: f32, y: f32, z: f32, h: f32) -> Partes {
    let trecho = h * 0.3;
    let mut v = Vec::new();
    for (i, m) in [0.25_f32, 0.85, 1.4].iter().enumerate() {
        v.push((Malha::caixa([x + m, y + m, z + i as f32 * trecho], [LADO - m * 2.0, LADO - m * 2.0, trecho]), Papel::Corpo));
    }
    v.push((Malha::cilindro((x + LADO * 0.5, y + LADO * 0.5), 0.09, z + trecho * 3.0, h * 0.1), Papel::Corpo));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plano(forma: &str, area: &str) -> Plano {
        Plano { forma: forma.into(), area: area.into(), ..Plano::default() }
    }

    #[test]
    fn a_forma_do_agente_ganha_da_area() {
        assert_eq!(Forma::do_plano(&plano("dados", "ui/src")), Forma::Dados);
        assert_eq!(Forma::do_plano(&plano("", "ui/src")), Forma::Tela);
        assert_eq!(Forma::do_plano(&plano("nao-existe", "src/dados/migracoes")), Forma::Dados);
        assert_eq!(Forma::do_plano(&plano("", "")), Forma::Padrao);
    }

    #[test]
    fn toda_forma_cabe_no_lote_e_tem_corpo() {
        for f in [Forma::Padrao, Forma::Tela, Forma::Dados, Forma::Servico, Forma::Integracao, Forma::Ia, Forma::Teste, Forma::Documento, Forma::Infra] {
            for h in [0.05, 1.0, 3.0] {
                let p = f.partes(0.0, 0.0, 0.0, h);
                assert!(p.iter().any(|(_, papel)| *papel == Papel::Corpo), "{f:?}");
            }
        }
    }

    #[test]
    fn o_vidro_fica_na_frente_do_monitor_e_so_quando_ele_tem_altura() {
        let c = vidro(0.0, 0.0, 0.0, 2.6).expect("o monitor inteiro tem vidro");
        assert!(c[1][0] > c[0][0] && c[0][2] > c[3][2] && c[0][1] > 1.95, "{c:?}");
        assert!(c.iter().all(|v| v[0] > 0.0 && v[0] < LADO), "{c:?}");
        assert_eq!(vidro(0.0, 0.0, 0.0, 0.5), None);
        let esboco = Forma::Tela.partes(0.0, 0.0, 0.0, 2.6).iter().filter(|(_, p)| *p == Papel::Esboco).count();
        assert_eq!(esboco, 2);
    }
}
