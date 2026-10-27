//! A fila: o servidor guarda, a página busca, aplica e confirma (ack). Cabem no máximo 200 itens.
//! As propostas formam uma fila por alvo (workspace + blueprint): cada uma tem um número crescente, entra no fim e só sai quando a
//! página confirma (ack) que o usuário a aplicou ou descartou. As leituras saem da memória; com `carregar` (o servidor de verdade), cada mudança
//! também vai para o banco (`dados::fila_mcp`) e a fila volta inteira ao reabrir o app. Sem banco (testes), é só memória, como antes.
use crate::dados::fila_mcp::FilaBanco;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

const FILA_MAX: usize = 200;

thread_local! {
    /// quem pôs o item (modo hospedado): `{id, nome}` do membro, valendo só durante a chamada da rota nesta thread
    static AUTOR: std::cell::RefCell<Option<Value>> = const { std::cell::RefCell::new(None) };
}

/// Roda `f` com o autor dos itens que ela empurrar (None = app local: o item sai sem `autor`, como sempre).
pub fn com_autor<T>(autor: Option<Value>, f: impl FnOnce() -> T) -> T {
    let antes = AUTOR.with(|a| a.replace(autor));
    let r = f();
    AUTOR.with(|a| *a.borrow_mut() = antes);
    r
}

#[derive(Default)]
pub struct Fila {
    pub itens: Vec<Value>,
    proximo: i64,
    /// o último número dado a uma proposta de cada alvo (workspace, blueprint); nunca volta atrás
    numeros: HashMap<(String, String), i64>,
    /// o que o agente enviou em cada proposta pendente (id do item → entrada, com os ids provisórios), para `ajustar`
    entradas: HashMap<i64, Value>,
    /// o último número dado a um REGISTRO de cada alvo: a contagem é própria, separada da das propostas (registro não é proposta)
    registros: HashMap<(String, String), i64>,
    /// a cópia durável; None = só memória
    banco: Option<FilaBanco>,
}

fn alvo_de(i: &Value) -> (&str, &str) {
    (i["workspaceId"].as_str().unwrap_or(""), i["blueprintId"].as_str().unwrap_or(""))
}

fn alvo_do_mapa(item: &Map<String, Value>) -> (String, String) {
    let s = |k: &str| item.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    (s("workspaceId"), s("blueprintId"))
}

impl Fila {
    /// A fila guardada no banco, como estava ao fechar; daqui em diante cada mudança é gravada nele.
    pub fn carregar(banco: FilaBanco) -> crate::erro::Resultado<Fila> {
        let guardada = banco.ler()?;
        let mut f = Fila::default();
        for (tipo, ws, bp, ultimo) in guardada.contadores {
            match tipo.as_str() {
                "id" => f.proximo = ultimo,
                "proposta" => {
                    f.numeros.insert((ws, bp), ultimo);
                }
                "registro" => {
                    f.registros.insert((ws, bp), ultimo);
                }
                _ => {}
            }
        }
        for (item, entrada) in guardada.itens {
            let id = item["id"].as_i64().unwrap_or(0);
            f.proximo = f.proximo.max(id);
            if let Some(e) = entrada {
                f.entradas.insert(id, e);
            }
            f.itens.push(item);
        }
        f.banco = Some(banco);
        Ok(f)
    }

    /// Leva a mudança ao banco (se houver): `sai` são os ids que saíram, `entra` é o item novo; os contadores vão juntos.
    /// Falhar aqui só avisa: o item já está na memória, e recusar faria o agente reenviar e duplicar.
    fn guardar(&mut self, sai: &[i64], entra: Option<&Value>) {
        let Some(banco) = self.banco.as_mut() else { return };
        let mut contadores = vec![("id", "", "", self.proximo)];
        for (tipo, mapa) in [("proposta", &self.numeros), ("registro", &self.registros)] {
            contadores.extend(mapa.iter().map(|((ws, bp), n)| (tipo, ws.as_str(), bp.as_str(), *n)));
        }
        let entra = entra.map(|i| {
            let id = i["id"].as_i64().unwrap_or(0);
            (id, i, self.entradas.get(&id))
        });
        if let Err(e) = banco.gravar(sai, entra, &contadores) {
            eprintln!("fila do MCP: não foi possível gravar no banco (o item segue na memória): {e}");
        }
    }

    /// Põe o item no fim, com id e hora. Devolve o item e o id que saiu pelo limite de 200 (se saiu).
    fn empurra_mem(&mut self, mut item: Map<String, Value>) -> (Value, Option<i64>) {
        self.proximo += 1;
        item.insert("id".into(), Value::from(self.proximo));
        item.insert("em".into(), Value::String(crate::tempo::iso_utc_agora()));
        if let Some(autor) = AUTOR.with(|a| a.borrow().clone()) {
            item.insert("autor".into(), autor);
        }
        let novo = Value::Object(item);
        self.itens.push(novo.clone());
        let mut saiu = None;
        if self.itens.len() > FILA_MAX {
            let id = self.itens.remove(0)["id"].as_i64().unwrap_or(0);
            self.entradas.remove(&id);
            saiu = Some(id);
        }
        (novo, saiu)
    }

    /// Põe o item no fim, com id e hora. Devolve o item como ficou.
    pub fn empurra(&mut self, item: Map<String, Value>) -> Value {
        let (novo, saiu) = self.empurra_mem(item);
        self.guardar(saiu.as_slice(), Some(&novo));
        novo
    }

    /// Proposta nova entra no fim com o próximo número do alvo e NÃO apaga nenhuma outra. Com `numero` (ajuste ou substituição), a proposta
    /// desse número sai de onde estava e esta entra no fim com o mesmo número e a mesma `criadaEm`.
    pub fn empurra_proposta(&mut self, mut item: Map<String, Value>, entrada: Value, numero: Option<i64>) -> Value {
        let alvo = alvo_do_mapa(&item);
        let antiga = numero.and_then(|n| self.posicao(&alvo.0, &alvo.1, n));
        let mut sai = Vec::new();
        let (numero, criada) = match antiga.map(|i| self.itens.remove(i)) {
            Some(velha) => {
                let id = velha["id"].as_i64().unwrap_or(0);
                self.entradas.remove(&id);
                sai.push(id);
                (numero.unwrap_or(0), velha["criadaEm"].clone())
            }
            None => {
                let atual = self.numeros.entry(alvo).or_insert(0);
                let n = numero.unwrap_or(*atual + 1);
                *atual = (*atual).max(n);
                (n, Value::Null)
            }
        };
        item.insert("numero".into(), Value::from(numero));
        let (novo, saiu) = self.empurra_mem(item);
        sai.extend(saiu);
        let criada = if criada.is_null() { novo["em"].clone() } else { criada };
        let i = self.itens.len() - 1;
        self.itens[i]["criadaEm"] = criada;
        self.entradas.insert(novo["id"].as_i64().unwrap_or(0), entrada);
        let novo = self.itens[i].clone();
        self.guardar(&sai, Some(&novo));
        novo
    }

    /// Registro do trabalho já feito: entra no fim com o próximo número de REGISTRO do alvo (separado das propostas) e não apaga nenhum outro.
    pub fn empurra_registro(&mut self, mut item: Map<String, Value>) -> Value {
        let n = self.registros.entry(alvo_do_mapa(&item)).or_insert(0);
        *n += 1;
        item.insert("numero".into(), Value::from(*n));
        let (novo, saiu) = self.empurra_mem(item);
        let i = self.itens.len() - 1;
        self.itens[i]["criadaEm"] = novo["em"].clone();
        let novo = self.itens[i].clone();
        self.guardar(saiu.as_slice(), Some(&novo));
        novo
    }

    fn posicao(&self, ws: &str, bp: &str, numero: i64) -> Option<usize> {
        self.itens.iter().position(|i| i["tipo"] == "proposta" && alvo_de(i) == (ws, bp) && i["numero"] == numero)
    }

    /// A entrada (o que o agente mandou, com ids provisórios) da proposta pendente `numero` do alvo.
    pub fn entrada(&self, ws: &str, bp: &str, numero: i64) -> Option<Value> {
        self.posicao(ws, bp, numero).and_then(|i| self.entradas.get(&self.itens[i]["id"].as_i64()?)).cloned()
    }

    /// Os números das propostas pendentes do alvo, do menor ao maior.
    pub fn pendentes(&self, ws: &str, bp: &str) -> Vec<i64> {
        let mut n: Vec<i64> = self.itens.iter().filter(|i| i["tipo"] == "proposta" && alvo_de(i) == (ws, bp)).filter_map(|i| i["numero"].as_i64()).collect();
        n.sort_unstable();
        n
    }

    /// A página aplicou ou descartou: os itens saem da memória e do banco (item aplicado nunca volta ao reabrir).
    pub fn ack(&mut self, ids: &HashSet<i64>) {
        let sai: Vec<i64> = self.itens.iter().filter_map(|i| i["id"].as_i64()).filter(|id| ids.contains(id)).collect();
        self.itens.retain(|i| !i["id"].as_i64().is_some_and(|id| ids.contains(&id)));
        self.entradas.retain(|id, _| !ids.contains(id));
        if !sai.is_empty() {
            self.guardar(&sai, None);
        }
    }

    /// A página aplicou ou descartou a proposta (workspace, blueprint, número, criadaEm). Pela chave inteira, e não pelo id do item, uma proposta
    /// antiga de antes de o app reiniciar nunca tira uma nova que ficou com o mesmo id.
    pub fn ack_proposta(&mut self, ws: &str, bp: &str, numero: i64, criada_em: &str) {
        self.ack_numerado("proposta", ws, bp, numero, criada_em);
    }

    /// A página adicionou ou descartou o registro (mesma chave inteira de `ack_proposta`).
    pub fn ack_registro(&mut self, ws: &str, bp: &str, numero: i64, criada_em: &str) {
        self.ack_numerado("registro", ws, bp, numero, criada_em);
    }

    fn ack_numerado(&mut self, tipo: &str, ws: &str, bp: &str, numero: i64, criada_em: &str) {
        let ids: HashSet<i64> = self.itens.iter().filter(|i| i["tipo"] == tipo && alvo_de(i) == (ws, bp) && i["numero"] == numero && i["criadaEm"] == criada_em).filter_map(|i| i["id"].as_i64()).collect();
        self.ack(&ids);
    }
}
