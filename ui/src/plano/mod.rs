//! O Planejar: a folha nova do blueprint, desenhada com o que o servidor já calculou (`/api/mcp/planejar/*`). A interface só mostra e pede;
//! dados e regras são do servidor. Cada tela fica no seu arquivo; aqui ficam o estado, a troca de tela e o laço com o servidor (`rede`).

// helpers de medida e de recarga das peças que as telas ainda não usam (`pasta::altura`, `caixinha::ALTURA_INICIAR`, `Estado::atualizar`)
#![allow(dead_code)]

pub mod acao;
pub mod acao_estilo;
pub mod acao_movimento;
pub mod agentes;
pub mod agentes_movimento;
pub mod agentes_pecas;
pub mod agentes_tempo;
pub mod caixinha;
pub mod cartao_botao;
pub mod cartao_caixa;
pub mod cartao_carimbo;
pub mod cartao_controle;
pub mod cartao_esqueleto;
pub mod cartao_ladrilho;
pub mod chegou;
pub mod detalhe;
pub mod diff;
pub mod escala;
pub mod escrita;
pub mod figura;
pub mod folha;
pub mod linha;
pub mod modelo;
pub mod orquestrador;
pub mod pasta;
pub mod perguntas;
pub mod rede;
pub mod revisao;
pub mod textos;
pub mod vazio_figuras;
pub mod vazio_iso;

use crate::documento::Documento;
use crate::i18n::tr;
use crate::telas::{Acao, Vista};
use crate::{planta, tema};

/// Qual tela do Planejar está aberta. `Agentes` e `Chegou` entram pelos botões do topo da folha.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Sub {
    #[default]
    Folha,
    Chegou,
    Agentes,
}

#[derive(Default)]
pub struct Estado {
    pub folha: Option<modelo::Folha>,
    pub agentes: Option<modelo::Agentes>,
    pub chegou: Option<modelo::Chegou>,
    pub config: Option<modelo::Config>,
    pub tela: Sub,
    /// a ação aberta na folha (`ref`)
    pub aberta: Option<String>,
    /// o roteiro de teste pediu a prévia deste detalhe (`ref` da ação, id): sai no próximo quadro, que tem o `Context`
    pub pedir_detalhe: Option<(String, String)>,
    /// o que o último gesto respondeu: recusa, ou os avisos do servidor
    pub aviso: String,
    /// o menu «Mais» do topo da folha está aberto
    /// `versaoDados` mudou: o `app` busca o documento de novo, sem passar por «Carregando…»
    pub recarregar_doc: bool,
    /// `planejar/folha` falhou e não há folha para mostrar (servidor sem a rota, por exemplo): a tela cai na folha antiga
    pub folha_falhou: bool,
    canal: rede::Canal,
}

impl Estado {
    /// Pede um gesto do usuário ao servidor (`gesto/<rota>`, com o token). A resposta volta em `aviso`, e a folha é refeita.
    pub fn gesto(&self, ctx: &egui::Context, servidor: &str, rota: &str, corpo: &serde_json::Value) {
        rede::gesto(ctx, &self.canal, servidor, rota, corpo);
    }

    /// Refaz a folha, os agentes e o que chegou no próximo quadro.
    pub fn atualizar(&mut self) {
        self.canal.pedir();
    }
}

/// O Planejar de um blueprint: conversa com o servidor e desenha a tela aberta.
pub fn tela(ui: &mut egui::Ui, vista: &mut Vista, doc: &Documento, ws: &str, bp: &str, servidor: &str) -> Option<Acao> {
    let ctx = ui.ctx().clone();
    rede::passo(&ctx, &mut vista.plano, ws, bp, servidor);
    if vista.plano.folha.is_none() {
        ui.painter().rect_filled(ui.max_rect(), 0, tema::PAPEL);
        planta::Texto::novo(tr("Carregando…"), 14.0).cor(tema::MEDIO).na_linha(ui.painter(), 64.0, 64.0, 22.4);
        return None;
    }
    if let Some((acao, det)) = vista.plano.pedir_detalhe.take() {
        detalhe::abrir(&ctx, &acao, &det);
        vista.plano.tela = Sub::Folha;
        vista.plano.aberta = Some(acao);
    }
    match vista.plano.tela.clone() {
        Sub::Folha => folha::tela(ui, vista, doc, ws, bp, servidor),
        Sub::Chegou => chegou::tela(ui, vista, doc, ws, bp, servidor),
        Sub::Agentes => agentes::tela(ui, vista, doc, ws, bp, servidor),
    }
}
