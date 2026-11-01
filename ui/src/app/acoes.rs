//! O que as telas pedem (`Acao`) vira mudança no documento e gravação no servidor.

use super::{App, Estado};
use crate::documento::{self, Documento};
use crate::i18n::tr;
use crate::rede;
use crate::telas::{Acao, Tela};

impl App {
    /// Muda o documento e grava. Recusado (`Err`), nada muda e o motivo aparece na tela.
    pub(super) fn mutar(&mut self, ctx: &egui::Context, f: impl FnOnce(&mut Documento) -> Result<Option<Tela>, String>) {
        let Estado::Pronto(doc) = &mut self.estado else { return };
        let antes = doc.raw.clone();
        match f(doc) {
            Err(motivo) => {
                doc.raw = antes;
                self.vista.erro = motivo;
            }
            Ok(depois) => {
                self.vista.erro.clear();
                self.vista.aviso = tr("Salvando…").to_string();
                self.vista.depois = depois;
                self.gravando = Some(antes);
                rede::gravar(&self.servidor, ctx, self.envio.clone(), doc);
            }
        }
    }

    pub(super) fn aplicar(&mut self, ctx: &egui::Context, acao: Acao) {
        if self.gravando.is_some() {
            return;
        }
        match acao {
            Acao::AbrirNota(ws, bp, acao) => {
                if let Estado::Pronto(doc) = &self.estado {
                    self.vista.nota_texto = doc.obs(&ws, &bp, acao.as_deref());
                }
                self.vista.nota = Some((ws, bp, acao));
            }
            Acao::SalvarObs(ws, bp, acao, novo) => self.mutar(ctx, |d| if d.definir_obs(&ws, &bp, acao.as_deref(), &novo) { Ok(None) } else { Err(tr("Não foi possível salvar.").to_string()) }),
            Acao::Atualizar => self.recarregar(ctx),
            Acao::DefinirNome(nome) => self.mutar(ctx, |d| {
                d.definir_usuario(&nome);
                Ok(None)
            }),
            Acao::CriarWorkspace { nome, descricao, icone } => {
                self.mutar(ctx, |d| d.criar_workspace(&nome, &descricao, &icone).map(|id| Some(Tela::Workspace(id))));
                if self.vista.erro.is_empty() {
                    self.vista.novo = None;
                } else if let Some(n) = self.vista.novo.as_mut() {
                    n.erro = std::mem::take(&mut self.vista.erro);
                }
            }
            Acao::CriarBlueprint { workspace, nome, descricao } => {
                self.mutar(ctx, |d| d.criar_blueprint(&workspace, &nome, &descricao).map(|id| Some(Tela::Blueprint(workspace.clone(), id))));
                match self.vista.novo_bp.as_mut() {
                    Some(n) if !self.vista.erro.is_empty() => n.erro = std::mem::take(&mut self.vista.erro),
                    _ => self.vista.novo_bp = None,
                }
            }
            Acao::Excluir(id) => self.mutar(ctx, |d| if d.excluir_workspace(&id) { Ok(None) } else { Err(tr("Não foi possível excluir o workspace.").to_string()) }),
            Acao::Mover(id, delta) => self.mutar(ctx, |d| {
                d.mover_workspace(&id, delta);
                Ok(None)
            }),
            Acao::Marcar(ws, bp, acao, tipo, item) => self.mutar(ctx, |d| if d.marcar_item(&ws, &bp, &acao, tipo, &item) { Ok(None) } else { Err(tr("Não foi possível marcar o item.").to_string()) }),
            Acao::MoverPara(id, para) => self.mutar(ctx, |d| {
                d.mover_para(&id, para);
                Ok(None)
            }),
            Acao::AbrirExemplo => {
                self.vista.aviso = tr("Abrindo o exemplo…").to_string();
                rede::exemplo(&self.servidor, ctx, self.envio.clone());
            }
            Acao::ExemploPronto(modelo) => self.mutar(ctx, |d| Ok(Some(Tela::Workspace(d.adicionar_exemplo(&modelo, &documento::hoje()))))),
        }
    }
}
