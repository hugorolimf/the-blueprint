//! O data.json inteiro (o que `GET /api/dados` entrega) e as regras da Entrada: workspaces, nome do usuário, exemplo.
//! O documento fica como JSON e só se mexe no que muda: campo que a interface nativa ainda não conhece volta intacto no `PUT`.

use serde_json::{json, Map, Value};

/// O workspace raiz é o próprio arquivo (id `sm`); os outros ficam em `workspaces[]`.
pub const RAIZ: &str = "sm";

/// O dia em que o `data.exemplo.json` foi escrito: as datas do exemplo andam o mesmo tanto que o dia de hoje.
const EXEMPLO_HOJE: &str = "2026-10-20";

pub struct Documento {
    pub raw: Value,
    /// `x-dados-versao`: o `PUT` leva de volta em `If-Match`, e o servidor recusa se outra janela gravou no meio.
    pub versao: u64,
}

pub struct Resumo {
    pub id: String,
    pub nome: String,
    pub icone: String,
    pub descricao: String,
    pub blueprints: usize,
    pub exemplo: bool,
}

fn texto<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

fn lista<'a>(v: &'a Value, campo: &str) -> &'a [Value] {
    v.get(campo).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

/// Compara nomes sem maiúsculas nem acento (o `norm` da página).
fn norma(s: &str) -> String {
    s.trim().chars().flat_map(char::to_lowercase).map(|c| match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        c => c,
    }).collect()
}

/// Um id curto de 7 letras e números (como o `novoId` da página). Sem `rand`: o relógio e um contador bastam para não repetir.
pub fn novo_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CONTADOR: AtomicU64 = AtomicU64::new(0);
    let nanos = web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64);
    let mut x = nanos ^ CONTADOR.fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed).wrapping_mul(0xBF58_476D_1CE4_E5B9) | 1;
    (0..7).map(|_| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        char::from_digit((x % 36) as u32, 36).unwrap_or('0')
    }).collect()
}

/// Dias desde 1970 de uma data `aaaa-mm-dd` (algoritmo de Howard Hinnant); `None` se não for uma data nesse formato.
pub(crate) fn dia(iso: &str) -> Option<i64> {
    let b = iso.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' || !iso.bytes().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit()) {
        return None;
    }
    let (a, m, d) = (iso[..4].parse::<i64>().ok()?, iso[5..7].parse::<i64>().ok()?, iso[8..].parse::<i64>().ok()?);
    let a = if m <= 2 { a - 1 } else { a };
    let era = a.div_euclid(400);
    let ya = a - era * 400;
    let doa = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    Some(era * 146_097 + ya * 365 + ya / 4 - ya / 100 + doa - 719_468)
}

pub(crate) fn iso_do_dia(dias: i64) -> String {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let ya = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doa = doe - (365 * ya + ya / 4 - ya / 100);
    let mp = (5 * doa + 2) / 153;
    let d = doa - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{m:02}-{d:02}", ya + era * 400 + i64::from(m <= 2))
}

/// O dia de hoje (UTC), `aaaa-mm-dd`.
pub fn hoje() -> String {
    let segundos = web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    iso_do_dia(segundos.div_euclid(86_400))
}

impl Documento {
    pub fn nome_usuario(&self) -> &str {
        self.raw.get("usuario").map_or("", |u| texto(u, "nome"))
    }

    pub fn definir_usuario(&mut self, nome: &str) {
        if let Some(raiz) = self.raw.as_object_mut() {
            let usuario = raiz.entry("usuario").or_insert_with(|| json!({}));
            if let Some(u) = usuario.as_object_mut() {
                u.insert("nome".into(), json!(nome.trim()));
            }
        }
    }

    /// A raiz primeiro, depois `workspaces[]`, na ordem de `ordemWorkspaces` (o que não está nela vai para o fim, na ordem do arquivo).
    pub fn workspaces(&self) -> Vec<Resumo> {
        let resumo = |id: &str, nome: &str, icone: &str, v: &Value| Resumo {
            id: id.to_string(),
            nome: nome.to_string(),
            descricao: texto(v, "descricao").to_string(),
            icone: if icone.is_empty() { "blueprint".into() } else { icone.to_string() },
            blueprints: lista(v, "blueprints").len(),
            exemplo: v.get("exemplo").and_then(Value::as_bool) == Some(true) && id != RAIZ,
        };
        let nome_raiz = if texto(&self.raw, "empresa").is_empty() { "Meu workspace" } else { texto(&self.raw, "empresa") };
        let mut todos = if self.raiz_excluida() { Vec::new() } else { vec![resumo(RAIZ, nome_raiz, texto(&self.raw, "icone"), &self.raw)] };
        todos.extend(lista(&self.raw, "workspaces").iter().map(|w| resumo(texto(w, "id"), texto(w, "nome"), texto(w, "icone"), w)));
        let ordem: Vec<&str> = lista(&self.raw, "ordemWorkspaces").iter().filter_map(Value::as_str).collect();
        todos.sort_by_key(|w| ordem.iter().position(|id| *id == w.id).unwrap_or(usize::MAX));
        todos
    }

    pub fn tem_blueprint(&self) -> bool {
        self.workspaces().iter().any(|w| w.blueprints > 0)
    }

    pub fn exemplo(&self) -> Option<Resumo> {
        self.workspaces().into_iter().find(|w| w.exemplo)
    }

    /// O workspace `id` como JSON: a raiz é o próprio arquivo.
    pub fn workspace(&self, id: &str) -> Option<&Value> {
        if id == RAIZ {
            return Some(&self.raw);
        }
        lista(&self.raw, "workspaces").iter().find(|w| texto(w, "id") == id)
    }

    fn raiz_mut(&mut self) -> &mut Map<String, Value> {
        if !self.raw.is_object() {
            self.raw = json!({ "blueprints": [] });
        }
        self.raw.as_object_mut().expect("é um objeto")
    }

    /// Cria o workspace e devolve o id. Os erros já vão em português, como na página.
    pub fn criar_workspace(&mut self, nome: &str, descricao: &str, icone: &str) -> Result<String, String> {
        let nome: String = nome.trim().chars().take(80).collect();
        if nome.is_empty() {
            return Err("Digite o nome do workspace.".into());
        }
        if self.workspaces().iter().any(|w| norma(&w.nome) == norma(&nome)) {
            return Err("Já existe um workspace com esse nome.".into());
        }
        let id = novo_id();
        let novo = json!({ "id": id, "nome": nome, "empresa": nome, "descricao": descricao.trim(), "emblema": null, "icone": icone, "blueprints": [] });
        self.raiz_mut().entry("workspaces").or_insert_with(|| json!([])).as_array_mut().map(|l| l.push(novo));
        Ok(id)
    }

    /// A raiz é o próprio arquivo, então não some dos dados: excluída, fica vazia e fora da lista. Se um agente puser um blueprint nela, ela volta a aparecer.
    fn raiz_excluida(&self) -> bool {
        self.raw.get("raizExcluida").and_then(Value::as_bool) == Some(true) && lista(&self.raw, "blueprints").is_empty()
    }

    /// Apaga o workspace com o que está dentro.
    pub fn excluir_workspace(&mut self, id: &str) -> bool {
        if self.workspace(id).is_none() || !self.workspaces().iter().any(|w| w.id == id) {
            return false;
        }
        let raiz = self.raiz_mut();
        if id == RAIZ {
            for campo in ["empresa", "titulo", "descricao", "icone", "emblema"] {
                raiz.remove(campo);
            }
            raiz.insert("blueprints".into(), json!([]));
            raiz.insert("raizExcluida".into(), json!(true));
        }
        if let Some(l) = raiz.get_mut("workspaces").and_then(Value::as_array_mut) {
            l.retain(|w| texto(w, "id") != id);
        }
        if let Some(l) = raiz.get_mut("ordemWorkspaces").and_then(Value::as_array_mut) {
            l.retain(|x| x.as_str() != Some(id));
        }
        true
    }

    /// Anda `delta` casas (-1 sobe, +1 desce) e grava a ordem inteira em `ordemWorkspaces`.
    pub fn mover_workspace(&mut self, id: &str, delta: i32) -> bool {
        let Some(de) = self.workspaces().iter().position(|w| w.id == id) else { return false };
        match de.checked_add_signed(delta as isize) {
            Some(para) => self.mover_para(id, para),
            None => false,
        }
    }

    /// Põe o workspace na posição `para` (o que o arrastar faz): os outros se afastam e a ordem inteira vai para `ordemWorkspaces`.
    pub fn mover_para(&mut self, id: &str, para: usize) -> bool {
        let mut ids: Vec<String> = self.workspaces().into_iter().map(|w| w.id).collect();
        let Some(de) = ids.iter().position(|x| x == id) else { return false };
        if para >= ids.len() || para == de {
            return false;
        }
        let movido = ids.remove(de);
        ids.insert(para, movido);
        self.raiz_mut().insert("ordemWorkspaces".into(), json!(ids));
        true
    }

    /// O `data.exemplo.json` como um workspace novo: ids novos em tudo, datas trazidas para perto de hoje. Nunca toca no que já existe.
    pub fn adicionar_exemplo(&mut self, modelo: &Value, hoje: &str) -> String {
        let desloca = dia(hoje).zip(dia(EXEMPLO_HOJE)).map_or(0, |(h, e)| h - e);
        let data = |v: &Value| match v.as_str().and_then(dia) {
            Some(d) => json!(iso_do_dia(d + desloca)),
            None => v.clone(),
        };
        let com_id = |v: &Value| {
            let mut o = v.as_object().cloned().unwrap_or_default();
            o.insert("id".into(), json!(novo_id()));
            o
        };
        let blueprints: Vec<Value> = lista(modelo, "blueprints").iter().map(|p| {
            let mut p = com_id(p);
            let acoes: Vec<Value> = lista(&Value::Object(p.clone()), "acoes").iter().map(|a| {
                let mut a = com_id(a);
                for campo in ["inicio", "fim"] {
                    if let Some(v) = a.get(campo) {
                        a.insert(campo.into(), data(v));
                    }
                }
                let planos: Vec<Value> = lista(&Value::Object(a.clone()), "planos").iter().map(|x| Value::Object(com_id(x))).collect();
                let checkpoints: Vec<Value> = lista(&Value::Object(a.clone()), "checkpoints").iter().map(|m| {
                    let mut m = com_id(m);
                    if let Some(v) = m.get("data") {
                        m.insert("data".into(), data(v));
                    }
                    Value::Object(m)
                }).collect();
                a.insert("planos".into(), json!(planos));
                a.insert("checkpoints".into(), json!(checkpoints));
                a.insert("prototipos".into(), json!([]));
                Value::Object(a)
            }).collect();
            p.insert("acoes".into(), json!(acoes));
            Value::Object(p)
        }).collect();
        let nome = if texto(modelo, "empresa").is_empty() { "Exemplo" } else { texto(modelo, "empresa") }.to_string();
        let id = novo_id();
        let ws = json!({ "id": id, "nome": nome, "empresa": nome, "descricao": "Dados fictícios para conhecer o app. Remova quando quiser.", "icone": "blueprint", "exemplo": true, "blueprints": blueprints });
        self.raiz_mut().entry("workspaces").or_insert_with(|| json!([])).as_array_mut().map(|l| l.push(ws));
        id
    }
}

#[cfg(test)]
mod testes;
mod blueprints;
