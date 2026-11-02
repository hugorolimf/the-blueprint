//! A recarga no lugar: o Planejar viu `versaoDados` mudar e o documento é buscado de novo sem passar por «Carregando…», com a tela à vista.

use super::{App, Estado};
use crate::documento::Documento;
use crate::rede;

impl App {
    /// Busca o documento, se o Planejar pediu e nada estiver gravando (a resposta pisaria na gravação).
    pub(super) fn recarregar_no_lugar(&mut self, ctx: &egui::Context) {
        if self.vista.plano.recarregar_doc && self.gravando.is_none() && !self.recarga_em_curso && matches!(self.estado, Estado::Pronto(_)) {
            self.vista.plano.recarregar_doc = false;
            self.recarga_em_curso = true;
            rede::buscar(&self.servidor, ctx, self.envio.clone());
        }
    }

    /// A resposta: entra se ainda vale; se uma gravação começou nesse meio-tempo, fica para depois dela. Falha não vira a tela de erro.
    pub(super) fn recarga_chegou(&mut self, resultado: Result<Documento, String>) {
        self.recarga_em_curso = false;
        let Estado::Pronto(atual) = &self.estado else { return };
        match resultado {
            Ok(_) if self.gravando.is_some() => self.vista.plano.recarregar_doc = true,
            // gravamos outra versão enquanto esta vinha: o pulso já pede a nova
            Ok(doc) if versao_velha(doc.versao, atual.versao) => {}
            Ok(doc) => self.estado = Estado::Pronto(doc),
            Err(motivo) => self.vista.erro = motivo,
        }
    }
}

fn versao_velha(chegou: u64, atual: u64) -> bool {
    chegou < atual
}

#[cfg(test)]
mod testes {
    #[test]
    fn recarga_mais_velha_que_a_tela_nao_entra() {
        assert!(super::versao_velha(4, 5));
        assert!(!super::versao_velha(5, 5));
        assert!(!super::versao_velha(6, 5));
    }
}
