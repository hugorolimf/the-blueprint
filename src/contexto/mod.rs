//! Quem está pedindo. Toda operação de `dados/` recebe um `Contexto`: no modo local é o dono implícito;
//! no modo servidor (ação 2 da direção do blueprint) vem do cookie do convite. Sem login nem papéis de verdade ainda.
use crate::erro::{Erro, Resultado};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Papel {
    Dono,
    Colaborador,
    Supervisor,
    Visitante,
}

#[derive(Clone, Debug)]
pub struct Contexto {
    pub usuario: String,
    pub papel: Papel,
}

impl Contexto {
    /// o usuário do app local: dono de tudo
    pub fn local() -> Self {
        Contexto { usuario: "local".into(), papel: Papel::Dono }
    }

    pub fn pode_ler(&self) -> bool {
        true
    }

    pub fn pode_editar(&self) -> bool {
        matches!(self.papel, Papel::Dono | Papel::Colaborador)
    }

    pub fn pode_administrar(&self) -> bool {
        self.papel == Papel::Dono
    }

    pub fn exige_editar(&self) -> Resultado<()> {
        if self.pode_editar() { Ok(()) } else { Err(Erro::Proibido(format!("{} não pode editar", self.usuario))) }
    }
}

#[cfg(test)]
mod tests;
