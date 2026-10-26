//! Presença (só memória, só modo hospedado): uma entrada por conexão SSE com o membro, o cliente (a aba), o blueprint e a ação abertos e o item
//! em edição. Nasce quando o SSE abre, some quando ele fecha; `editando` vence em `EDITANDO_VALE` sem nova mensagem. O evento `presenca` vai ao
//! workspace no máximo a cada `INTERVALO` (uma thread curta espera o resto do intervalo só quando há mudança; nada roda em repouso).
use super::{sse, travar, Interno};
use crate::dados::Autor;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub const INTERVALO: Duration = Duration::from_millis(500);
pub const EDITANDO_VALE: Duration = Duration::from_secs(120);

/// O que `POST /api/presenca` manda.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Onde {
    pub blueprint_id: String,
    pub acao_id: String,
    /// `{tipo, id}` do item em edição, ou None
    pub editando: Option<Value>,
}

struct Pessoa {
    conexao: u64,
    membro: String,
    nome: String,
    cliente: String,
    workspace_id: String,
    onde: Onde,
    editando_em: Instant,
}

#[derive(Default)]
pub struct Presencas {
    pessoas: Vec<Pessoa>,
    envio: HashMap<String, Instant>,
    agendado: HashSet<String>,
}

impl Presencas {
    pub fn entrar(&mut self, conexao: u64, autor: &Autor, cliente: &str, workspace_id: &str) {
        self.pessoas.push(Pessoa { conexao, membro: autor.id.clone(), nome: autor.nome.clone(), cliente: cliente.into(), workspace_id: workspace_id.into(), onde: Onde::default(), editando_em: Instant::now() });
    }

    /// A conexão caiu: devolve o workspace dela, se estava na presença.
    pub fn sair(&mut self, conexao: u64) -> Option<String> {
        let i = self.pessoas.iter().position(|p| p.conexao == conexao)?;
        Some(self.pessoas.remove(i).workspace_id)
    }

    /// Atualiza as conexões do (membro, cliente); devolve os workspaces a avisar.
    pub fn atualizar(&mut self, autor: &Autor, cliente: &str, onde: Onde) -> Vec<String> {
        let mut ws = Vec::new();
        for p in self.pessoas.iter_mut().filter(|p| p.membro == autor.id && p.cliente == cliente) {
            if p.onde != onde {
                p.onde = onde.clone();
                p.editando_em = Instant::now();
                if !ws.contains(&p.workspace_id) {
                    ws.push(p.workspace_id.clone());
                }
            }
        }
        ws
    }

    pub fn lista(&self, workspace_id: &str) -> Value {
        let agora = Instant::now();
        let pessoas: Vec<Value> = self
            .pessoas
            .iter()
            .filter(|p| p.workspace_id == workspace_id)
            .map(|p| {
                let editando = p.onde.editando.clone().filter(|_| agora.duration_since(p.editando_em) < EDITANDO_VALE);
                json!({"id": p.membro, "nome": p.nome, "cliente": p.cliente, "blueprintId": p.onde.blueprint_id, "acaoId": p.onde.acao_id, "editando": editando})
            })
            .collect();
        Value::Array(pessoas)
    }
}

fn enviar_lista(i: &mut Interno, ws: &str) {
    i.presenca.envio.insert(ws.to_string(), Instant::now());
    let texto = sse(None, "presenca", &json!({"workspaceId": ws, "pessoas": i.presenca.lista(ws)}));
    i.enviar(Some(ws), &texto);
}

/// Manda a lista do workspace agora ou, se a última foi há menos de `INTERVALO`, agenda uma só para o fim do intervalo.
pub(crate) fn avisar(interno: &Arc<Mutex<Interno>>, ws: &str) {
    let mut i = travar(interno);
    if i.presenca.agendado.contains(ws) {
        return;
    }
    let espera = i.presenca.envio.get(ws).map_or(Duration::ZERO, |t| INTERVALO.saturating_sub(t.elapsed()));
    if espera.is_zero() {
        return enviar_lista(&mut i, ws);
    }
    i.presenca.agendado.insert(ws.to_string());
    let (interno, ws) = (interno.clone(), ws.to_string());
    thread::spawn(move || {
        thread::sleep(espera);
        let mut i = travar(&interno);
        i.presenca.agendado.remove(&ws);
        enviar_lista(&mut i, &ws);
    });
}
