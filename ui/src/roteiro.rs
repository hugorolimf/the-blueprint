//! Testar sem mouse: `BLUEPRINT_UI_ROTEIRO=nome:Ana,criar:Alfa,exemplo,excluir:Alfa,novo` roda as ações em ordem, cada uma quando a anterior terminou
//! de gravar, e `BLUEPRINT_UI_ABRIR=novo` abre o diálogo. Junto com `BLUEPRINT_UI_CAPTURA` dá para ver o resultado sem tocar na tela.

use crate::documento::Documento;
use crate::telas::{Acao, Novo, Vista};
use std::collections::VecDeque;

pub struct Roteiro(VecDeque<String>);

pub fn do_ambiente() -> Option<Roteiro> {
    let texto = std::env::var("BLUEPRINT_UI_ROTEIRO").ok().filter(|v| !v.is_empty())?;
    Some(Roteiro(texto.split(',').map(str::to_string).collect()))
}

/// `BLUEPRINT_UI_PPP=1`: força a escala a cada quadro (a do monitor, 1,25 neste notebook, muda o tamanho em pontos); a comparação com a web é a 1.
pub fn escala(ctx: &egui::Context) {
    if let Some(ppp) = std::env::var("BLUEPRINT_UI_PPP").ok().and_then(|v| v.parse::<f32>().ok()) {
        ctx.set_zoom_factor(ppp / ctx.native_pixels_per_point().unwrap_or(1.0));
    }
}

/// Abre o que `BLUEPRINT_UI_ABRIR` pede (só o diálogo `novo` por enquanto).
pub fn abrir(vista: &mut Vista) {
    if std::env::var("BLUEPRINT_UI_ABRIR").as_deref() == Ok("novo") {
        vista.novo = Some(Novo::vazio());
    }
}

impl Roteiro {
    /// O próximo passo, se houver. `excluir:Nome` procura o workspace pelo nome.
    pub fn proximo(&mut self, doc: &Documento, vista: &mut Vista) -> Option<Acao> {
        let passo = self.0.pop_front()?;
        let (verbo, arg) = passo.split_once(':').unwrap_or((passo.as_str(), ""));
        match verbo {
            "nome" => Some(Acao::DefinirNome(arg.into())),
            "criar" => Some(Acao::CriarWorkspace { nome: arg.into(), descricao: String::new(), icone: "chip".into() }),
            "exemplo" => Some(Acao::AbrirExemplo),
            "excluir" => doc.workspaces().into_iter().find(|w| w.nome == arg).map(|w| Acao::Excluir(w.id)),
            "mover" => doc.workspaces().into_iter().find(|w| w.nome == arg).map(|w| Acao::Mover(w.id, -1)),
            "voltar" => {
                vista.tela = crate::telas::Tela::Entrada;
                None
            }
            "novo" => {
                vista.novo = Some(Novo::vazio());
                None
            }
            _ => None,
        }
    }
}
