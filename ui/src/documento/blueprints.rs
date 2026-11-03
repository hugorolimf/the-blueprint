//! O que muda dentro de um workspace: criar blueprint, marcar plano ou checkpoint e as observações.

use super::{lista, novo_id, texto, Documento, RAIZ};
use serde_json::{json, Value};

impl Documento {
    /// O workspace `id` para mudar: a raiz é o próprio arquivo.
    fn workspace_mut(&mut self, id: &str) -> Option<&mut Value> {
        if id == RAIZ {
            return Some(&mut self.raw);
        }
        self.raw.get_mut("workspaces").and_then(Value::as_array_mut).and_then(|l| l.iter_mut().find(|w| texto(w, "id") == id))
    }

    /// O blueprint vazio da tela «Novo blueprint» (`criarBlueprint` de proposta.js), no fim da lista do workspace. Devolve o id.
    pub fn criar_blueprint(&mut self, workspace: &str, nome: &str, descricao: &str) -> Result<String, String> {
        let nome = nome.trim();
        if nome.is_empty() {
            return Err("Defina o nome do blueprint antes de criar.".into());
        }
        let ws = self.workspace_mut(workspace).ok_or("Workspace não encontrado.")?;
        let id = novo_id();
        let novo = json!({ "id": id, "nome": nome, "descricao": descricao.trim(), "obs": "", "links": [], "acoes": [], "emblema": null });
        let Some(o) = ws.as_object_mut() else { return Err("Workspace não encontrado.".into()) };
        match o.entry("blueprints").or_insert_with(|| json!([])).as_array_mut() {
            Some(l) => l.push(novo),
            None => return Err("Workspace não encontrado.".into()),
        }
        Ok(id)
    }

    /// Liga ou desliga o `feito` de um plano (`tipo` «planos») ou checkpoint (`tipo` «checkpoints») de uma ação. `false` se não achou.
    pub fn marcar_item(&mut self, workspace: &str, blueprint: &str, acao: &str, tipo: &str, item: &str) -> bool {
        let alvo = self
            .workspace_mut(workspace)
            .and_then(|w| w.get_mut("blueprints")?.as_array_mut()?.iter_mut().find(|b| texto(b, "id") == blueprint))
            .and_then(|b| b.get_mut("acoes")?.as_array_mut()?.iter_mut().find(|a| texto(a, "id") == acao))
            .and_then(|a| a.get_mut(tipo)?.as_array_mut()?.iter_mut().find(|i| texto(i, "id") == item));
        let Some(Value::Object(i)) = alvo else { return false };
        let feito = i.get("feito").and_then(Value::as_bool) == Some(true);
        i.insert("feito".into(), json!(!feito));
        true
    }

    fn acao_mut(&mut self, workspace: &str, blueprint: &str, acao: &str) -> Option<&mut serde_json::Map<String, Value>> {
        self.workspace_mut(workspace)?.get_mut("blueprints")?.as_array_mut()?.iter_mut().find(|b| texto(b, "id") == blueprint)?.get_mut("acoes")?.as_array_mut()?.iter_mut().find(|a| texto(a, "id") == acao)?.as_object_mut()
    }

    /// A situação escolhida no menu (`mudarSituacaoAcao`): concluir enche o progresso, voltar a planejada zera, em andamento mantém.
    pub fn definir_situacao(&mut self, workspace: &str, blueprint: &str, acao: &str, status: &str) -> bool {
        let Some(a) = self.acao_mut(workspace, blueprint, acao) else { return false };
        a.insert("status".into(), json!(status));
        match status {
            "concluido" => a.insert("progresso".into(), json!(1)),
            "planejado" => a.insert("progresso".into(), json!(0)),
            _ => None,
        };
        true
    }

    pub fn excluir_acao(&mut self, workspace: &str, blueprint: &str, acao: &str) -> bool {
        let lista = self.workspace_mut(workspace).and_then(|w| w.get_mut("blueprints")?.as_array_mut()?.iter_mut().find(|b| texto(b, "id") == blueprint)).and_then(|b| b.get_mut("acoes")?.as_array_mut());
        let Some(lista) = lista else { return false };
        let antes = lista.len();
        lista.retain(|a| texto(a, "id") != acao);
        lista.len() != antes
    }

    pub fn excluir_blueprint(&mut self, workspace: &str, blueprint: &str) -> bool {
        let Some(lista) = self.workspace_mut(workspace).and_then(|w| w.get_mut("blueprints")?.as_array_mut()) else { return false };
        let antes = lista.len();
        lista.retain(|b| texto(b, "id") != blueprint);
        lista.len() != antes
    }

    /// O texto das observações do blueprint (`acao` vazia) ou da ação.
    pub fn obs(&self, workspace: &str, blueprint: &str, acao: Option<&str>) -> String {
        let bp = lista(self.workspace(workspace).unwrap_or(&Value::Null), "blueprints").iter().find(|b| texto(b, "id") == blueprint);
        let alvo = match acao {
            None => bp,
            Some(a) => bp.and_then(|b| lista(b, "acoes").iter().find(|x| texto(x, "id") == a)),
        };
        alvo.map(|v| texto(v, "obs").to_string()).unwrap_or_default()
    }

    /// Grava as observações (vazias apagam o campo). `false` se o blueprint ou a ação não existem.
    pub fn definir_obs(&mut self, workspace: &str, blueprint: &str, acao: Option<&str>, novo: &str) -> bool {
        let bp = self.workspace_mut(workspace).and_then(|w| w.get_mut("blueprints")?.as_array_mut()?.iter_mut().find(|b| texto(b, "id") == blueprint));
        let alvo = match (bp, acao) {
            (Some(b), None) => Some(b),
            (Some(b), Some(a)) => b.get_mut("acoes").and_then(Value::as_array_mut).and_then(|l| l.iter_mut().find(|x| texto(x, "id") == a)),
            _ => None,
        };
        let Some(Value::Object(o)) = alvo else { return false };
        if novo.trim().is_empty() {
            o.remove("obs");
        } else {
            o.insert("obs".into(), json!(novo));
        }
        true
    }
}
