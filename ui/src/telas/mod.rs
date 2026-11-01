//! As telas da interface nativa. Cada uma desenha com o documento já lido e devolve, no máximo, uma `Acao` que muda os dados;
//! o que é só da tela (abrir um diálogo, trocar de tela) ela mesma muda em `Vista`. Quem grava no servidor é o `app`.

mod agente;
mod entrada;
mod excluir;
mod nome;
mod nota;
mod novo;
mod novo_bp;

use serde_json::Value;
use crate::documento::Documento;

pub use novo::Novo;
pub use novo_bp::NovoBp;

#[derive(Debug)]
pub enum Tela {
    Entrada,
    Workspace(String),
    /// um blueprint aberto: (workspace, blueprint)
    Blueprint(String, String),
}

/// O período escolhido na régua de um blueprint (`periodos` de dados.js): `aguardando` é o 1º clique, que só marca o mês inicial.
#[derive(Clone, Debug)]
pub struct Periodo {
    pub inicio: String,
    pub fim: String,
    pub aguardando: bool,
    /// a janela que estava na tela antes do 1º clique (enquanto se espera o 2º)
    pub anterior: Option<(String, String)>,
}

/// O que a tela pede que mude nos dados.
pub enum Acao {
    DefinirNome(String),
    CriarWorkspace { nome: String, descricao: String, icone: String },
    /// «Criar blueprint vazio»: entra no fim do workspace e abre na folha
    CriarBlueprint { workspace: String, nome: String, descricao: String },
    Excluir(String),
    Mover(String, i32),
    /// arrastado até outro cartão: vai para a posição dele
    MoverPara(String, usize),
    AbrirExemplo,
    /// o modelo de exemplo chegou do servidor: vira um workspace novo (só o `app` manda esta)
    ExemploPronto(Box<serde_json::Value>),
    /// marcar ou desmarcar um plano ou checkpoint: (workspace, blueprint, ação, `planos`/`checkpoints`, item)
    Marcar(String, String, String, &'static str, String),
    /// abrir a folha de observações do blueprint (sem ação) ou da ação: (workspace, blueprint, ação)
    AbrirNota(String, String, Option<String>),
    /// gravar o texto dessa folha
    SalvarObs(String, String, Option<String>, String),
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
    /// a tela «Novo blueprint» aberta (cartão tracejado da lista)
    pub novo_bp: Option<NovoBp>,
    /// o id do workspace que o diálogo de exclusão pergunta
    pub excluir: Option<String>,
    /// o que fazer quando a gravação em curso terminar (ir para o workspace que acabou de nascer)
    pub depois: Option<Tela>,
    pub aviso: String,
    pub erro: String,
    /// a busca da lista de blueprints
    pub consulta: String,
    /// só nos testes: o quanto rolar a folha ao abrir (`BLUEPRINT_UI_ROLAR`)
    pub rolagem: Option<f32>,
    /// a busca da barra compacta: aberta por cima dos ícones (clique na lupa) e se ela deve pegar o foco agora
    pub busca_aberta: bool,
    pub focar_busca: bool,
    /// as seções abertas ou recolhidas (`blueprint|projeto`); sem entrada vale o padrão: o blueprint sem projetos nasce aberto, o projeto nasce fechado
    pub abertas: std::collections::HashMap<String, bool>,
    /// o período escolhido por blueprint e os que estão no filtro «Pendentes»
    pub periodos: std::collections::HashMap<String, Periodo>,
    pub pendentes: std::collections::HashSet<String>,
    /// a janela de meses desenhada por último em cada blueprint (`janelasRenderizadas`)
    pub janelas: std::collections::HashMap<String, (String, String)>,
    /// os agentes que o servidor conhece e quanto tempo sem aviso leva ao repouso; os que o usuário já viu (`concluidosVistos`)
    pub agentes: Vec<crate::agentes::Agente>,
    pub agentes_vida: i64,
    pub vistos: std::collections::HashSet<String>,
    /// o agente cuja folhinha está aberta (clicou na figura)
    pub agente_aberto: Option<String>,
    /// só nos testes: um clique de ponteiro de verdade a injetar (roteiro `clicar:x,y`)
    pub clique_pedido: Option<egui::Pos2>,
    /// a folha de observações aberta e o texto que está sendo editado
    pub nota: Option<(String, String, Option<String>)>,
    pub nota_texto: String,
}

impl Vista {
    /// O que vale guardar entre sessões: as seções abertas e os últimos 200 agentes vistos.
    pub fn para_guardar(&self) -> Value {
        let mut vistos: Vec<&String> = self.vistos.iter().collect();
        vistos.sort();
        let vistos: Vec<&&String> = vistos.iter().rev().take(200).collect();
        serde_json::json!({ "abertas": self.abertas, "vistos": vistos })
    }

    pub fn restaurar(&mut self, salvo: &Value) {
        if let Some(m) = salvo.get("abertas").and_then(Value::as_object) {
            self.abertas = m.iter().filter_map(|(k, v)| v.as_bool().map(|b| (k.clone(), b))).collect();
        }
        if let Some(l) = salvo.get("vistos").and_then(Value::as_array) {
            self.vistos = l.iter().filter_map(|v| v.as_str().map(str::to_string)).collect();
        }
    }

    /// O workspace aberto (onde os agentes aparecem); vazio na entrada.
    pub fn agente_ws(&self) -> String {
        match &self.tela {
            Tela::Workspace(id) | Tela::Blueprint(id, _) => id.clone(),
            Tela::Entrada => String::new(),
        }
    }

    pub fn nova() -> Vista {
        Vista { tela: Tela::Entrada, nome: String::new(), trocando_nome: false, novo: None, novo_bp: None, excluir: None, depois: None, aviso: String::new(), erro: String::new(), consulta: String::new(), rolagem: None, busca_aberta: false, focar_busca: false, abertas: std::collections::HashMap::new(), periodos: std::collections::HashMap::new(), pendentes: std::collections::HashSet::new(), janelas: std::collections::HashMap::new(), agentes: Vec::new(), agentes_vida: 600_000, vistos: std::collections::HashSet::new(), agente_aberto: None, clique_pedido: None, nota: None, nota_texto: String::new() }
    }
}

/// Os diálogos por cima da tela (a exclusão). Devolve o que a pessoa confirmou.
pub fn dialogos(ctx: &egui::Context, vista: &mut Vista, doc: &Documento, ocupado: bool) -> Option<Acao> {
    agente::faixa(ctx, vista, doc);
    agente::folhinha(ctx, vista, doc);
    if let Some(a) = nota::folha(ctx, vista, doc, ocupado) {
        return Some(a);
    }
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
    if vista.novo_bp.is_some() {
        return novo_bp::tela(ui, vista, ocupado);
    }
    match vista.tela {
        Tela::Entrada => entrada::tela(ui, vista, doc, servidor, ocupado),
        Tela::Workspace(ref id) => {
            let id = id.clone();
            crate::folha::lista(ui, vista, doc, &id, servidor)
        }
        Tela::Blueprint(ref id, ref bp) => {
            let (id, bp) = (id.clone(), bp.clone());
            crate::folha::blueprint_aberto(ui, vista, doc, &id, &bp, servidor)
        }
    }
}

/// O texto no estilo dos widgets do egui (diálogos e menus): na cor pedida.
pub fn texto(s: impl Into<String>, cor: egui::Color32) -> egui::RichText {
    egui::RichText::new(s).color(cor)
}
