//! O estado de um plano na ação aberta (livre, na fila, esperando, trabalhando, pronto, parado): a figura do agente, o ponto, o texto e um mini indicador que anda.
//! Cor e texto sempre juntos: a cor sozinha não diz nada.

use super::super::acao_estilo::{alfa, ponto};
use super::super::acao_movimento::{fase, pulso, pulso_breve};
use super::super::caixinha::{clique_de, Clique};
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Plano, Situacao};
use crate::i18n::tr;
use crate::tema::{AMBAR, AZUL, MEDIO, VERDE, VERMELHO};
use egui::load::{Bytes, TexturePoll};
use egui::{pos2, vec2, Color32, Image, Rect, Shape, Stroke, Ui};
use std::io::Read;
use std::sync::{Arc, OnceLock};

pub const LARGURA_INDICADOR: f32 = 28.0;
const LARGURA_FIGURA: f32 = 21.0;

/// As figuras da situação: as dos agentes de plano, para não se confundirem com o orquestrador (que é a `principal`).
const POSES: [(&str, &[u8]); 4] = [
    ("trabalhando", include_bytes!("../../assets/claude/construtor-trabalhando-0.svg.gz")),
    ("esperando", include_bytes!("../../assets/claude/construtor-esperando-0.svg.gz")),
    ("bloqueado", include_bytes!("../../assets/claude/construtor-bloqueado-0.svg.gz")),
    ("concluido", include_bytes!("../../assets/claude/construtor-concluido-0.svg.gz")),
];

/// O que o estado diz e a cor dele. `None` só para o que um servidor mais novo mandou e este desenho não conhece.
pub fn situacao_de(q: &Plano) -> Option<(&'static str, Color32)> {
    Some(match q.situacao {
        Situacao::Outra => return None,
        Situacao::Nao if clique_de(q) == Some(Clique::Marcar) => (tr("Livre"), MEDIO),
        Situacao::Nao => return None,
        Situacao::Revisao => (tr("Revise a mudança"), AMBAR),
        Situacao::Trabalhando => (tr("Trabalhando"), AZUL),
        Situacao::Fila => (tr("Na fila"), MEDIO),
        Situacao::Dependencia => (tr("Esperando plano"), AMBAR),
        Situacao::Area => (tr("Esperando área"), AMBAR),
        Situacao::Parado => (tr("Parado"), VERMELHO),
        Situacao::Bloqueado => match q.causa.as_str() {
            "sumiu" => (tr("O agente sumiu"), VERMELHO),
            "falhou" => (tr("Falhou ao iniciar"), VERMELHO),
            _ => (tr("Pediu ajuda"), VERMELHO),
        },
        Situacao::Pronto if q.agente.is_empty() => (tr("Marcado como feito"), VERDE),
        Situacao::Pronto => (tr("O agente concluiu e marcou"), VERDE),
    })
}

/// A pose da figura. Plano feito à mão não tem agente, portanto não tem figura.
pub fn pose_de(q: &Plano) -> Option<usize> {
    match q.situacao {
        Situacao::Trabalhando => Some(0),
        Situacao::Revisao | Situacao::Fila | Situacao::Dependencia | Situacao::Area => Some(1),
        Situacao::Parado | Situacao::Bloqueado => Some(2),
        Situacao::Pronto if !q.agente.is_empty() => Some(3),
        _ => None,
    }
}

pub fn figura(c: &Pincel, ui: &Ui, indice: usize, r: Rect) {
    static CACHE: [OnceLock<Arc<[u8]>>; 4] = [const { OnceLock::new() }; 4];
    let (nome, gz) = POSES[indice];
    let svg = CACHE[indice].get_or_init(|| {
        let mut bytes = Vec::new();
        // embutido no programa: se não abrir, a figura fica vazia e a situação continua escrita ao lado
        let _ = flate2::read::GzDecoder::new(gz).read_to_end(&mut bytes);
        bytes.into()
    });
    // pelo pincel do bloco, e não pelo `ui`: assim a figura clareia e é cortada junto com o painel que está abrindo
    let imagem = Image::from_bytes(format!("bytes://plano-agente-{nome}.svg"), Bytes::Shared(svg.clone())).fit_to_exact_size(r.size());
    if let Ok(TexturePoll::Ready { texture }) = imagem.load_for_size(ui.ctx(), r.size()) {
        c.p.image(texture.id, r, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
}

/// O mini indicador de `LARGURA_INDICADOR` px em `(x, y)` (o meio da linha): uma barra que diz a mesma coisa que o texto, andando quando o plano anda.
fn indicador(c: &Pincel, x: f32, y: f32, q: &Plano, cor: Color32) {
    let s = q.situacao;
    let l = LARGURA_INDICADOR;
    let trilho = Rect::from_min_size(c.pt(x, y - 1.5), vec2(l, 3.0));
    let p = c.p;
    p.rect_filled(trilho, 1.5, alfa(cor, 0.2));
    match s {
        Situacao::Trabalhando => {
            let t = fase(p.ctx(), 1.5);
            let ponta = trilho.min.x + (l + 12.0) * t;
            let trecho = Rect::from_min_max(pos2((ponta - 12.0).max(trilho.min.x), trilho.min.y), pos2(ponta.min(trilho.max.x), trilho.max.y));
            if trecho.width() > 0.0 {
                p.rect_filled(trecho, 1.5, cor);
            }
        }
        Situacao::Pronto => {
            p.rect_filled(trilho, 1.5, alfa(cor, 0.8));
        }
        Situacao::Fila | Situacao::Dependencia | Situacao::Area => {
            let respira = pulso_breve(p.ctx(), ("espera", &q.id), 2.2, 8.0);
            for i in 0..3 {
                let acesa = if (respira * 3.0) as usize == i { 1.0 } else { 0.45 };
                p.circle_filled(pos2(trilho.min.x + 4.0 + 10.0 * i as f32, trilho.center().y), 1.6, alfa(cor, acesa));
            }
        }
        Situacao::Revisao => {
            p.extend(Shape::dashed_line(&[trilho.left_center(), trilho.right_center()], Stroke::new(2.0, alfa(cor, 0.9)), 4.0, 3.0));
        }
        Situacao::Parado | Situacao::Bloqueado => {
            p.rect_filled(Rect::from_min_size(trilho.min, vec2(l * 0.4, 3.0)), 1.5, alfa(cor, 0.85));
            p.rect_filled(Rect::from_min_size(pos2(trilho.min.x + l * 0.55, trilho.min.y), vec2(l * 0.45, 3.0)), 1.5, alfa(cor, 0.35));
        }
        Situacao::Nao | Situacao::Outra => {}
    }
}

/// A largura que o estado de `q` ocupa, com o texto cortado em `max`.
pub fn largura(c: &Pincel, q: &Plano, max: f32) -> f32 {
    let Some((rot, _)) = situacao_de(q) else { return 0.0 };
    let frase = frase_de(q, rot);
    let corte = c.cortado(&frase, (max - LARGURA_INDICADOR - 26.0).max(30.0), &Letra::nova(9.5, MEDIO)).0;
    let figura = if pose_de(q).is_some() { LARGURA_FIGURA + 5.0 } else { 0.0 };
    figura + 12.0 + c.largura(&corte, &Letra::nova(9.5, MEDIO)) + 8.0 + LARGURA_INDICADOR
}

fn frase_de(q: &Plano, rot: &str) -> String {
    if q.por.is_empty() {
        rot.to_string()
    } else {
        format!("{rot} · {}", q.por)
    }
}

/// Desenha o estado preso à borda direita `x_fim`, com o meio da linha em `y_meio`. A dica traz a frase inteira e o motivo.
pub fn desenhar(ui: &Ui, c: &Pincel, q: &Plano, x_fim: f32, y_meio: f32, max: f32) {
    let Some((rot, cor)) = situacao_de(q) else { return };
    let frase = frase_de(q, rot);
    let letra = Letra::nova(9.5, cor);
    let (corte, cortou) = c.cortado(&frase, (max - LARGURA_INDICADOR - 26.0).max(30.0), &letra);
    let w_texto = c.largura(&corte, &letra);
    let x_indicador = x_fim - LARGURA_INDICADOR;
    indicador(c, x_indicador, y_meio, q, cor);
    let x_texto = x_indicador - 8.0 - w_texto;
    c.texto(&corte, x_texto, y_meio + 3.4, letra);
    let halo = if q.situacao == Situacao::Trabalhando { pulso(ui.ctx(), 1.6) } else { 0.0 };
    ponto(c.p, c.pt(x_texto - 7.0, y_meio), cor, 2.6, halo);
    let mut x_esq = x_texto - 12.0;
    if let Some(pose) = pose_de(q) {
        x_esq -= LARGURA_FIGURA + 3.0;
        figura(c, ui, pose, Rect::from_min_size(c.pt(x_esq + 3.0, y_meio - 8.0), vec2(LARGURA_FIGURA, LARGURA_FIGURA * 170.0 / 220.0)));
    }
    if cortou || !q.motivo.is_empty() {
        let dica = if q.motivo.is_empty() { frase } else { format!("{frase}\n{}", q.motivo) };
        let area = Rect::from_min_max(c.pt(x_esq, y_meio - 9.0), c.pt(x_fim, y_meio + 9.0));
        let _ = ui.interact(area, ui.scope_id().with(("plano-situacao", &q.id)), egui::Sense::hover()).on_hover_text(dica);
    }
}
