//! As telas da interface nativa. Cada uma desenha com o documento já lido e devolve, no máximo, uma `Acao` que muda os dados;
//! o que é só da tela (abrir um diálogo, trocar de tela) ela mesma muda em `Vista`. Quem grava no servidor é o `app`.

mod entrada;
mod excluir;
mod nome;
mod novo;
mod workspace;

use crate::documento::Documento;

pub use novo::Novo;

pub enum Tela {
    Entrada,
    Workspace(String),
}

/// O que a tela pede que mude nos dados.
pub enum Acao {
    DefinirNome(String),
    CriarWorkspace { nome: String, descricao: String, icone: String },
    Excluir(String),
    Mover(String, i32),
    /// arrastado até outro cartão: vai para a posição dele
    MoverPara(String, usize),
    AbrirExemplo,
    /// o modelo de exemplo chegou do servidor: vira um workspace novo (só o `app` manda esta)
    ExemploPronto(Box<serde_json::Value>),
    Atualizar,
}

/// O estado só da tela: nada daqui vai para o servidor.
pub struct Vista {
    pub tela: Tela,
    /// o campo de «Qual é o seu nome?»
    pub nome: String,
    /// «Trocar nome»: mostra a tela de nome mesmo com o nome já salvo
    pub trocando_nome: bool,
    pub novo: Option<Novo>,
    /// o id do workspace que o diálogo de exclusão pergunta
    pub excluir: Option<String>,
    /// o que fazer quando a gravação em curso terminar (ir para o workspace que acabou de nascer)
    pub depois: Option<Tela>,
    pub aviso: String,
    pub erro: String,
}

impl Vista {
    pub fn nova() -> Vista {
        Vista { tela: Tela::Entrada, nome: String::new(), trocando_nome: false, novo: None, excluir: None, depois: None, aviso: String::new(), erro: String::new() }
    }
}

/// Os diálogos por cima da tela (a exclusão). Devolve o que a pessoa confirmou.
pub fn dialogos(ctx: &egui::Context, vista: &mut Vista, doc: &Documento, ocupado: bool) -> Option<Acao> {
    excluir::dialogo(ctx, vista, doc, ocupado)
}

/// Desenha a tela certa para o estado. `ocupado`: há uma gravação em curso, então nada que grave pode ser pedido agora.
pub fn desenhar(ui: &mut egui::Ui, vista: &mut Vista, doc: &Documento, servidor: &str, ocupado: bool) -> Option<Acao> {
    if doc.nome_usuario().is_empty() || vista.trocando_nome {
        return nome::tela(ui, vista, doc, ocupado);
    }
    if vista.novo.is_some() {
        return novo::tela(ui, vista, ocupado);
    }
    match vista.tela {
        Tela::Entrada => entrada::tela(ui, vista, doc, servidor, ocupado),
        Tela::Workspace(ref id) => {
            let id = id.clone();
            workspace::tela(ui, vista, doc, &id, servidor)
        }
    }
}

/// O texto no estilo dos widgets do egui (diálogos e menus): na cor pedida.
pub fn texto(s: impl Into<String>, cor: egui::Color32) -> egui::RichText {
    egui::RichText::new(s).color(cor)
}
