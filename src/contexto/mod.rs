//! Quem está pedindo. Toda rota e toda operação de `dados/` recebem um `Contexto`: no modo local é o dono implícito de tudo; no modo
//! hospedado (`blueprint serve --hospedar`) vem da sessão (cookie) ou da chave do MCP, resolvido uma vez no despacho (`crate::acesso`).
//! A decisão de ler, gravar ou administrar é uma função só: `Contexto::pode` (tabela em docs/colaboracao.md, seção 3).
use crate::erro::{Erro, Resultado};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Papel {
    // a ordem importa: vale o maior papel que cobre o alvo
    Visitante,
    Supervisor,
    Colaborador,
    Dono,
}

impl Papel {
    pub fn texto(self) -> &'static str {
        match self {
            Papel::Dono => "dono",
            Papel::Colaborador => "colaborador",
            Papel::Supervisor => "supervisor",
            Papel::Visitante => "visitante",
        }
    }

    pub fn de_texto(t: &str) -> Option<Papel> {
        [Papel::Dono, Papel::Colaborador, Papel::Supervisor, Papel::Visitante].into_iter().find(|p| p.texto() == t)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acao {
    Ler,
    Gravar,
    Administrar,
}

/// O papel de alguém num escopo: `workspace_id` `'*'` = todos os workspaces; `blueprint_id` vazio = o workspace inteiro.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Acesso {
    pub workspace_id: String,
    pub blueprint_id: String,
    pub papel: Papel,
    /// o que o link abre ("folha" ou "apresentacao"); só o visitante usa
    pub abre: String,
}

impl Acesso {
    pub fn novo(workspace_id: &str, blueprint_id: &str, papel: Papel) -> Acesso {
        Acesso { workspace_id: workspace_id.into(), blueprint_id: blueprint_id.into(), papel, abre: "folha".into() }
    }

    fn cobre(&self, workspace: &str, blueprint: Option<&str>) -> bool {
        (self.workspace_id == "*" || self.workspace_id == workspace) && (self.blueprint_id.is_empty() || Some(self.blueprint_id.as_str()) == blueprint)
    }
}

#[derive(Clone, Debug)]
pub struct Contexto {
    /// id do membro; "local" no app local; "visitante:<convite>" para quem entrou por um link público; "" = ninguém (sem sessão)
    pub usuario: String,
    pub nome: String,
    /// o maior papel entre os acessos (compatível com o código de antes dos acessos)
    pub papel: Papel,
    pub acessos: Vec<Acesso>,
    pub local: bool,
    pub visitante: bool,
}

impl Contexto {
    /// o usuário do app local: dono de tudo
    pub fn local() -> Self {
        Contexto { usuario: "local".into(), nome: "local".into(), papel: Papel::Dono, acessos: vec![Acesso::novo("*", "", Papel::Dono)], local: true, visitante: false }
    }

    /// Um membro do modo hospedado com os acessos dele.
    pub fn membro(id: &str, nome: &str, acessos: Vec<Acesso>) -> Self {
        let papel = acessos.iter().map(|a| a.papel).max().unwrap_or(Papel::Visitante);
        Contexto { usuario: id.into(), nome: nome.into(), papel, acessos, local: false, visitante: false }
    }

    /// Quem entrou por um link público: só lê o blueprint do convite.
    pub fn visitante(convite_id: &str, workspace_id: &str, blueprint_id: &str, abre: &str) -> Self {
        let acesso = Acesso { abre: abre.into(), ..Acesso::novo(workspace_id, blueprint_id, Papel::Visitante) };
        Contexto { usuario: format!("visitante:{convite_id}"), nome: String::new(), papel: Papel::Visitante, acessos: vec![acesso], local: false, visitante: true }
    }

    /// Ninguém: pedido sem sessão no modo hospedado (só a página e as rotas de convite atendem).
    pub fn anonimo() -> Self {
        Contexto { usuario: String::new(), nome: String::new(), papel: Papel::Visitante, acessos: Vec::new(), local: false, visitante: false }
    }

    /// Tem sessão (ou chave)? No modo local, sempre.
    pub fn autenticado(&self) -> bool {
        self.local || !self.usuario.is_empty()
    }

    /// O maior papel que cobre o alvo (`'*'` > workspace > blueprint); None = não alcança.
    pub fn papel_em(&self, workspace: &str, blueprint: Option<&str>) -> Option<Papel> {
        self.acessos.iter().filter(|a| a.cobre(workspace, blueprint)).map(|a| a.papel).max()
    }

    /// A função única de decisão. `blueprint` None = o workspace inteiro (quem só tem um blueprint dele não alcança).
    pub fn pode(&self, acao: Acao, workspace: &str, blueprint: Option<&str>) -> bool {
        if self.local {
            return true;
        }
        let Some(papel) = self.papel_em(workspace, blueprint) else { return false };
        match acao {
            Acao::Ler => true,
            Acao::Gravar => matches!(papel, Papel::Dono | Papel::Colaborador),
            Acao::Administrar => papel == Papel::Dono,
        }
    }

    pub fn exige(&self, acao: Acao, workspace: &str, blueprint: Option<&str>) -> Resultado<()> {
        if self.pode(acao, workspace, blueprint) { Ok(()) } else { Err(Erro::Proibido(crate::tr!("you do not have permission for this", "você não tem permissão para isso"))) }
    }

    pub fn pode_ler(&self) -> bool {
        self.local || !self.acessos.is_empty()
    }

    /// Edita algum lugar (as gravações da camada de dados conferem o alvo com `pode`; esta é a guarda geral, de antes dos acessos).
    pub fn pode_editar(&self) -> bool {
        matches!(self.papel, Papel::Dono | Papel::Colaborador)
    }

    pub fn pode_administrar(&self) -> bool {
        self.papel == Papel::Dono
    }

    pub fn exige_editar(&self) -> Resultado<()> {
        if self.pode_editar() { Ok(()) } else { Err(Erro::Proibido(crate::tr!("{} cannot edit", "{} não pode editar", self.usuario))) }
    }
}

#[cfg(test)]
mod tests;
