//! Os botões laterais do mouse: voltar e avançar pelos lugares da interface. As telas continuam mudando `Vista` direto; o histórico só observa, a cada quadro,
//! onde a pessoa está, então um caminho novo entre telas entra sozinho. Janela por cima e formulário não são lugares: voltar fecha, como o botão deles faz.

use crate::documento::Documento;
use crate::plano::{self, Sub};
use crate::telas::{Tela, Vista};
use egui::{Context, Key, PointerButton, Value as _};
use serde_json::Value;

/// Quantos lugares ficam para trás: o bastante para qualquer sessão, sem crescer para sempre.
const LIMITE: usize = 200;

/// Um lugar para onde dá para voltar: a tela e, no blueprint, o que está aberto nele.
#[derive(Clone, Debug, PartialEq)]
pub struct Lugar {
    tela: Tela,
    sub: Sub,
    aberta: Option<String>,
    detalhe: Option<String>,
    /// a folha antiga em vez do Planejar
    antiga: bool,
}

#[derive(Default)]
pub struct Historico {
    atras: Vec<Lugar>,
    frente: Vec<Lugar>,
    atual: Option<Lugar>,
}

impl Lugar {
    /// Onde a pessoa está. Trocar de blueprint zera o Planejar só no desenho seguinte: até lá, o que ele guarda é do blueprint anterior e não conta.
    fn de(ctx: &Context, vista: &Vista) -> Lugar {
        let tela = vista.tela.clone();
        let simples = |tela, antiga| Lugar { tela, sub: Sub::default(), aberta: None, detalhe: None, antiga };
        let Tela::Blueprint(ws, bp) = &vista.tela else {
            return simples(tela, false);
        };
        if vista.folha_antiga.contains(bp) {
            return simples(tela, true);
        }
        if !plano::rede::e_o_alvo(&vista.plano, ws, bp) {
            return simples(tela, false);
        }
        let aberta = vista.plano.aberta.clone();
        let detalhe = aberta.as_deref().and_then(|a| plano::detalhe::aberto(ctx, a));
        Lugar { tela, sub: vista.plano.tela.clone(), aberta, detalhe, antiga: false }
    }

    /// Workspace ou blueprint apagado depois da visita não é destino: a tela o trocaria por outro, e voltar ficaria preso nesse vaivém.
    fn existe(&self, doc: &Documento) -> bool {
        match &self.tela {
            Tela::Entrada => true,
            Tela::Workspace(ws) => doc.workspace(ws).is_some(),
            Tela::Blueprint(ws, bp) => doc.workspace(ws).and_then(|w| w.get("blueprints")?.as_array()).is_some_and(|l| l.iter().any(|b| b.get("id").and_then(Value::as_str) == Some(bp))),
        }
    }

    /// Leva a tela de volta a este lugar.
    fn ir(self, ctx: &Context, vista: &mut Vista) {
        vista.tela = self.tela;
        let Tela::Blueprint(ws, bp) = &vista.tela else { return };
        if self.antiga {
            vista.folha_antiga.insert(bp.clone());
            return;
        }
        vista.folha_antiga.remove(bp);
        plano::rede::entrar(&mut vista.plano, ws, bp);
        vista.plano.tela = self.sub;
        if let Some(a) = &self.aberta {
            match &self.detalhe {
                Some(d) => plano::detalhe::abrir(ctx, a, d),
                None => plano::detalhe::fechar(ctx, a),
            }
        }
        vista.plano.aberta = self.aberta;
    }
}

impl Historico {
    /// Anota o lugar deste quadro. Voltar pelo botão da própria tela ao lugar anterior conta como voltar: o histórico não cresce em vaivém, e avançar segue valendo.
    fn visitou(&mut self, lugar: Lugar) {
        let Some(antes) = self.atual.take() else {
            self.atual = Some(lugar);
            return;
        };
        if antes != lugar {
            if self.atras.last() == Some(&lugar) {
                self.atras.pop();
                self.frente.push(antes);
            } else {
                self.atras.push(antes);
                if self.atras.len() > LIMITE {
                    self.atras.remove(0);
                }
                self.frente.clear();
            }
        }
        self.atual = Some(lugar);
    }

    fn voltar(&mut self, existe: impl Fn(&Lugar) -> bool) -> Option<Lugar> {
        andar(&mut self.atras, &mut self.frente, &mut self.atual, existe)
    }

    fn avancar(&mut self, existe: impl Fn(&Lugar) -> bool) -> Option<Lugar> {
        andar(&mut self.frente, &mut self.atras, &mut self.atual, existe)
    }

    /// Um quadro: anota onde a pessoa está e atende os botões laterais (ou as teclas de voltar e avançar, que alguns mouses mandam no lugar deles).
    pub fn passo(&mut self, ctx: &Context, vista: &mut Vista, doc: &Documento) {
        let lugar = Lugar::de(ctx, vista);
        // o painel que acompanha o agente abre sozinho: não foi a pessoa que andou, e a frente dela não se perde
        if std::mem::take(&mut vista.plano.aberta_pelo_agente) {
            self.atual = Some(lugar);
        } else {
            self.visitou(lugar);
        }
        let (tras, frente) = ctx.input(|i| (i.pointer.button_pressed(PointerButton::Extra1) || i.key_pressed(Key::BrowserBack), i.pointer.button_pressed(PointerButton::Extra2) || i.key_pressed(Key::BrowserForward)));
        // a folha de observações segue aberta de propósito: fechá-la por um toque sem querer jogaria fora o que foi escrito
        if !(tras || frente) || doc.nome_usuario().is_empty() || vista.depois.is_some() || vista.nota.is_some() {
            return;
        }
        if tras && fechar_por_cima(vista) {
            return;
        }
        if tem_algo_por_cima(vista) {
            return;
        }
        let existe = |l: &Lugar| l.existe(doc);
        let destino = if tras { self.voltar(existe) } else { self.avancar(existe) };
        if let Some(l) = destino {
            l.ir(ctx, vista);
        }
    }
}

/// O próximo lugar de `de` que ainda existe e não é onde a pessoa já está; o que fica no caminho é esquecido.
fn andar(de: &mut Vec<Lugar>, para: &mut Vec<Lugar>, atual: &mut Option<Lugar>, existe: impl Fn(&Lugar) -> bool) -> Option<Lugar> {
    while let Some(l) = de.pop() {
        if existe(&l) && atual.as_ref() != Some(&l) {
            para.extend(atual.replace(l.clone()));
            return Some(l);
        }
    }
    None
}

/// Fecha o que está por cima da tela, sem salvar nada. Devolve se havia o que fechar.
fn fechar_por_cima(vista: &mut Vista) -> bool {
    let havia = tem_algo_por_cima(vista);
    vista.menu = None;
    vista.excluir = None;
    vista.excluir_item = None;
    vista.agente_aberto = None;
    vista.novo = None;
    vista.novo_bp = None;
    vista.trocando_nome = false;
    havia
}

fn tem_algo_por_cima(vista: &Vista) -> bool {
    vista.menu.is_some() || vista.excluir.is_some() || vista.excluir_item.is_some() || vista.agente_aberto.is_some() || vista.novo.is_some() || vista.novo_bp.is_some() || vista.trocando_nome
}

#[cfg(test)]
#[path = "historico_testes.rs"]
mod testes;
