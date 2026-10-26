//! O lote de operações por item (`POST /api/mudancas`, docs/colaboracao.md seção 6) aplicado numa transação: criar, alterar campos, mover e
//! apagar. Regra de conflito campo a campo: se o valor atual é o `antes` que a página tinha, aplica sem aviso; se é outro, a última gravação
//! vence e a resposta lista o conflito com quem tinha mudado (a última linha de `historico_campos`). Item ou pai apagado não derruba o lote:
//! a operação é ignorada (apagar vence editar). Operação inválida ou sem permissão derruba o lote inteiro (nada muda).
use super::formato::{Tabela, WORKSPACES, RAIZ_ID};
use super::historico::{self, Autor, Linha};
use super::importar::inserir_no;
use super::item::{self, tabela, tipo_do_pai};
use super::linhas::Objeto;
use crate::erro::{Erro, Resultado};
use rusqlite::Connection;
use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub enum Op {
    Criar { tipo: String, id: String, pai: Option<String>, posicao: usize, valor: Objeto },
    Alterar { tipo: String, id: String, campos: Objeto, antes: Objeto },
    Mover { tipo: String, id: String, pai: Option<String>, posicao: usize },
    Apagar { tipo: String, id: String },
}

impl Op {
    pub fn tipo(&self) -> &str {
        match self {
            Op::Criar { tipo, .. } | Op::Alterar { tipo, .. } | Op::Mover { tipo, .. } | Op::Apagar { tipo, .. } => tipo,
        }
    }
    pub fn id(&self) -> &str {
        match self {
            Op::Criar { id, .. } | Op::Alterar { id, .. } | Op::Mover { id, .. } | Op::Apagar { id, .. } => id,
        }
    }
}

/// O que derruba o lote inteiro.
#[derive(Debug)]
pub enum Falha {
    Invalida { indice: usize, motivo: String },
    Proibida { indice: usize, motivo: String },
    Erro(Erro),
}

impl From<Erro> for Falha {
    fn from(e: Erro) -> Self {
        Falha::Erro(e)
    }
}
impl From<rusqlite::Error> for Falha {
    fn from(e: rusqlite::Error) -> Self {
        Falha::Erro(e.into())
    }
}

/// Uma operação que mudou o banco e onde ela caiu (para o evento por blueprint).
#[derive(Debug, PartialEq)]
pub struct Aplicada {
    pub indice: usize,
    pub workspace_id: String,
    pub blueprint_id: String,
}

#[derive(Debug, Default)]
pub struct Saida {
    pub conflitos: Vec<Value>,
    pub ignoradas: Vec<Value>,
    pub aplicadas: Vec<Aplicada>,
}

/// Quem pergunta a permissão: `(administrar, workspace, blueprint)`. Administrar = criar ou apagar workspace, mudar o workspace raiz.
pub type Exige<'a> = &'a dyn Fn(bool, &str, Option<&str>) -> Resultado<()>;

/// Ausente, null, "", false, 0, [] e {} valem o mesmo: a página normaliza os itens (põe `obs: ""`, `links: []`...) e isso não é conflito.
fn vazio(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Bool(b) => !b,
        Value::Number(n) => n.as_f64() == Some(0.0),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
    }
}

pub fn igual(a: &Value, b: &Value) -> bool {
    a == b || (vazio(a) && vazio(b))
}

fn bp(b: &str) -> Option<&str> {
    (!b.is_empty()).then_some(b)
}

fn tab(indice: usize, tipo: &str) -> Result<&'static Tabela, Falha> {
    tabela(tipo).ok_or_else(|| Falha::Invalida { indice, motivo: format!("tipo desconhecido: {tipo}") })
}

fn exigir(exige: Exige, indice: usize, administrar: bool, onde: &(String, String)) -> Result<(), Falha> {
    exige(administrar, &onde.0, bp(&onde.1)).map_err(|e| Falha::Proibida { indice, motivo: e.to_string() })
}

struct Lote<'a> {
    tx: &'a Connection,
    autor: &'a Autor,
    versao: u64,
    em: String,
    saida: Saida,
    tocados: Vec<(String, String)>,
}

impl Lote<'_> {
    fn historico(&mut self, onde: &(String, String), tipo: &str, id: &str, campo: &str, operacao: &str, antes: Option<&Value>, depois: Option<&Value>) -> Resultado<()> {
        historico::gravar(self.tx, &Linha { versao: self.versao, em: &self.em, autor: self.autor, workspace_id: &onde.0, blueprint_id: &onde.1, tipo, item_id: id, campo, operacao, antes, depois })?;
        self.tocados.push((tipo.to_string(), id.to_string()));
        Ok(())
    }

    fn ignorar(&mut self, indice: usize, codigo: &str, tipo: &str, id: &str) -> Resultado<()> {
        let quem = historico::ultima(self.tx, tipo, id, None)?.map(|(q, _)| q);
        self.saida.ignoradas.push(json!({"indice": indice, "codigo": codigo, "tipo": tipo, "id": id, "quem": quem}));
        Ok(())
    }

    fn aplicada(&mut self, indice: usize, onde: (String, String)) {
        self.saida.aplicadas.push(Aplicada { indice, workspace_id: onde.0, blueprint_id: onde.1 });
    }

    /// Onde o pai está; None (e a operação ignorada) se ele não existe mais.
    fn pai_existe(&mut self, indice: usize, tipo: &str, pai: &Option<String>) -> Result<Option<(String, String)>, Falha> {
        let Some(tipo_pai) = tipo_do_pai(tipo) else {
            if pai.is_some() {
                return Err(Falha::Invalida { indice, motivo: "workspace não tem pai".into() });
            }
            return Ok(Some((String::new(), String::new())));
        };
        let Some(pai) = pai else { return Err(Falha::Invalida { indice, motivo: "falta o pai".into() }) };
        match item::onde(self.tx, tipo_pai, pai)? {
            Some(o) => Ok(Some(o)),
            None => {
                self.ignorar(indice, "pai-apagado", tipo_pai, pai)?;
                Ok(None)
            }
        }
    }

    fn criar(&mut self, indice: usize, exige: Exige, tipo: &str, id: &str, pai: &Option<String>, posicao: usize, valor: &Objeto) -> Result<(), Falha> {
        let t = tab(indice, tipo)?;
        let Some(onde_pai) = self.pai_existe(indice, tipo, pai)? else { return Ok(()) };
        let onde = match tipo { "workspace" => (id.to_string(), String::new()), "blueprint" => (onde_pai.0.clone(), id.to_string()), _ => onde_pai };
        exigir(exige, indice, tipo == "workspace", &onde)?;
        if item::existe(self.tx, t, id)? {
            self.saida.ignoradas.push(json!({"indice": indice, "codigo": "id-existe", "tipo": tipo, "id": id, "quem": null}));
            return Ok(());
        }
        let mut valor = valor.clone();
        valor.insert("id".into(), Value::String(id.into()));
        let corpo = Value::Object(valor);
        inserir_no(self.tx, t, None, pai.as_deref(), i64::MAX / 2, corpo.clone(), &[]).map_err(|e| match e {
            Erro::Formato(m) => Falha::Invalida { indice, motivo: m },
            Erro::Banco(rusqlite::Error::SqliteFailure(f, _)) if f.code == rusqlite::ErrorCode::ConstraintViolation => Falha::Invalida { indice, motivo: "id repetido na subárvore".into() },
            outro => Falha::Erro(outro),
        })?;
        item::posicionar(self.tx, t, pai.as_deref(), id, posicao)?;
        match (tipo_do_pai(tipo), pai) {
            (Some(tp), Some(p)) => item::marcar_lista(self.tx, tab(indice, tp)?, p, t.lista)?,
            _ => item::marcar_lista(self.tx, &WORKSPACES, RAIZ_ID, WORKSPACES.lista)?,
        }
        self.historico(&onde, tipo, id, "*", "criar", None, Some(&corpo))?;
        self.aplicada(indice, onde);
        Ok(())
    }

    fn alterar(&mut self, indice: usize, exige: Exige, tipo: &str, id: &str, campos: &Objeto, antes: &Objeto) -> Result<(), Falha> {
        let t = tab(indice, tipo)?;
        if let Some(k) = campos.keys().find(|k| item::chave_proibida(t, k)) {
            return Err(Falha::Invalida { indice, motivo: format!("o campo {k} não pode ser alterado") });
        }
        let (Some(lido), Some(onde)) = (item::ler(self.tx, t, id)?, item::onde(self.tx, tipo, id)?) else {
            return Ok(self.ignorar(indice, "item-apagado", tipo, id)?);
        };
        exigir(exige, indice, tipo == "workspace" && id == RAIZ_ID, &onde)?;
        let mut obj = lido.obj;
        let mut mudou = false;
        for (campo, novo) in campos {
            let atual = obj.get(campo).cloned().unwrap_or(Value::Null);
            if igual(&atual, novo) {
                continue;
            }
            if !igual(&atual, antes.get(campo).unwrap_or(&Value::Null)) {
                let (quem, em) = historico::ultima(self.tx, tipo, id, Some(campo))?.map_or((Value::Null, Value::Null), |(q, e)| (q, Value::String(e)));
                self.saida.conflitos.push(json!({"indice": indice, "tipo": tipo, "id": id, "campo": campo, "deles": atual, "seu": novo, "quem": quem, "em": em}));
            }
            if novo.is_null() { obj.remove(campo); } else { obj.insert(campo.clone(), novo.clone()); }
            self.historico(&onde, tipo, id, campo, "alterar", Some(&atual), Some(novo))?;
            mudou = true;
        }
        if mudou {
            item::escrever(self.tx, t, id, lido.listas, obj)?;
            self.aplicada(indice, onde);
        }
        Ok(())
    }

    fn mover(&mut self, indice: usize, exige: Exige, tipo: &str, id: &str, pai: &Option<String>, posicao: usize) -> Result<(), Falha> {
        let t = tab(indice, tipo)?;
        let (Some(lido), Some(antes)) = (item::ler(self.tx, t, id)?, item::onde(self.tx, tipo, id)?) else {
            return Ok(self.ignorar(indice, "item-apagado", tipo, id)?);
        };
        let Some(onde_pai) = self.pai_existe(indice, tipo, pai)? else { return Ok(()) };
        let depois = match tipo { "workspace" => antes.clone(), "blueprint" => (onde_pai.0, id.to_string()), _ => onde_pai };
        exigir(exige, indice, false, &antes)?;
        exigir(exige, indice, false, &depois)?;
        if let (Some(col), Some(p)) = (t.pai, pai) {
            self.tx.execute(&format!("UPDATE {} SET {col} = ?1, revisao = revisao + 1 WHERE id = ?2", t.nome), (p, id))?;
            item::marcar_lista(self.tx, tab(indice, tipo_do_pai(tipo).unwrap_or("workspace"))?, p, t.lista)?;
        }
        item::posicionar(self.tx, t, pai.as_deref(), id, posicao)?;
        self.historico(&depois, tipo, id, "*", "mover", Some(&json!({"pai": lido.pai})), Some(&json!({"pai": pai, "posicao": posicao})))?;
        self.aplicada(indice, depois);
        Ok(())
    }

    fn apagar(&mut self, indice: usize, exige: Exige, tipo: &str, id: &str) -> Result<(), Falha> {
        let t = tab(indice, tipo)?;
        if tipo == "workspace" && id == RAIZ_ID {
            return Err(Falha::Invalida { indice, motivo: "o workspace raiz não pode ser apagado".into() });
        }
        let (Some(lido), Some(onde)) = (item::ler(self.tx, t, id)?, item::onde(self.tx, tipo, id)?) else {
            return Ok(self.ignorar(indice, "item-apagado", tipo, id)?);
        };
        exigir(exige, indice, tipo == "workspace", &onde)?;
        let descendentes = item::descendentes(self.tx, tipo, id)?;
        for (tipo_d, id_d) in &descendentes {
            let onde_d = item::onde(self.tx, tipo_d, id_d)?.unwrap_or_else(|| onde.clone());
            self.historico(&onde_d, tipo_d, id_d, "*", "apagar", None, None)?;
        }
        self.historico(&onde, tipo, id, "*", "apagar", Some(&Value::Object(lido.obj)), None)?;
        self.tx.execute(&format!("DELETE FROM {} WHERE id = ?1", t.nome), [id])?;
        self.aplicada(indice, onde);
        Ok(())
    }
}

/// Aplica o lote numa transação. `versao` é a versão que o lote cria (vai para o histórico); quem chama só a publica se `aplicadas` não
/// estiver vazio. Em `Falha` nada muda.
pub fn aplicar(conn: &mut Connection, autor: &Autor, versao: u64, agora_ms: i64, ops: &[Op], exige: Exige) -> Result<Saida, Falha> {
    let tx = conn.transaction()?;
    let mut lote = Lote { tx: &tx, autor, versao, em: crate::tempo::iso_utc(agora_ms), saida: Saida::default(), tocados: Vec::new() };
    for (i, op) in ops.iter().enumerate() {
        match op {
            Op::Criar { tipo, id, pai, posicao, valor } => lote.criar(i, exige, tipo, id, pai, *posicao, valor)?,
            Op::Alterar { tipo, id, campos, antes } => lote.alterar(i, exige, tipo, id, campos, antes)?,
            Op::Mover { tipo, id, pai, posicao } => lote.mover(i, exige, tipo, id, pai, *posicao)?,
            Op::Apagar { tipo, id } => lote.apagar(i, exige, tipo, id)?,
        }
    }
    historico::reter(lote.tx, agora_ms, &lote.tocados)?;
    let saida = std::mem::take(&mut lote.saida);
    drop(lote);
    tx.commit()?;
    Ok(saida)
}
