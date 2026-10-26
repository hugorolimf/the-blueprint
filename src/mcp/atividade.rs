//! Quem está trabalhando agora: a presença dos agentes do usuário (Claude Code, Codex e seus subagentes), só em memória.
//! É presença, não histórico: some ao reiniciar (o histórico fica nos comentários do plano). O agente NÃO sai por falta de sinal: a página o põe em
//! repouso (parado no cabeçalho do blueprint) depois de `VIDA_MS` e ele só deixa a lista quando o limite de `MAX_AGENTES` o substitui.
//! O app não abre, não conversa e não executa agentes: isto só guarda o que eles mesmos reportam pelo MCP.
use serde_json::{Map, Value};

/// Sem sinal por tanto tempo, a página considera o agente em repouso (ele continua na lista).
pub const VIDA_MS: i64 = 10 * 60 * 1000;
/// Regra do limite: com a lista cheia, o agente novo substitui o que está há mais tempo sem sinal.
pub const MAX_AGENTES: usize = 32;

struct Registro {
    nome: String,
    dados: Map<String, Value>,
    em_ms: i64,
    /// desempata avisos no mesmo milissegundo: quem avisou por último vem primeiro
    seq: u64,
}

#[derive(Default)]
pub struct Atividade {
    registros: Vec<Registro>,
    seq: u64,
}

/// O que descreve o plano em que o agente está: vale o do último aviso (um aviso sem plano a limpa).
const DA_PLANO: [&str; 3] = ["acaoId", "planoId", "plano"];

impl Atividade {
    /// Anota o aviso do agente `nome`. O que o aviso trouxe (papel, estado, onde, percentual) substitui o anterior; o que não trouxe fica
    /// como estava, e os campos do plano sempre refletem o último aviso.
    pub fn registrar(&mut self, agora_ms: i64, nome: &str, mut novos: Map<String, Value>) {
        self.seq += 1;
        for c in DA_PLANO {
            novos.entry(c).or_insert(Value::Null);
        }
        if let Some(r) = self.registros.iter_mut().find(|r| r.nome == nome) {
            r.dados.extend(novos);
            (r.em_ms, r.seq) = (agora_ms, self.seq);
            return;
        }
        if self.registros.len() >= MAX_AGENTES {
            if let Some(i) = self.registros.iter().enumerate().min_by_key(|(_, r)| (r.em_ms, r.seq)).map(|(i, _)| i) {
                self.registros.remove(i);
            }
        }
        novos.entry("estado").or_insert_with(|| Value::String("escrevendo".into())); // quem avisa que trabalha, trabalha
        self.registros.push(Registro { nome: nome.to_string(), dados: novos, em_ms: agora_ms, seq: self.seq });
    }

    /// Um plano foi marcado como concluído (a ferramenta não diz quem é o agente): quem avisou por último que trabalhava nele passa a `concluido`,
    /// com a evidência como mensagem. Devolve se achou alguém.
    pub fn concluir_plano(&mut self, agora_ms: i64, blueprint_id: &str, plano_id: &str, mensagem: &Value) -> bool {
        if plano_id.is_empty() {
            return false;
        }
        let Some(r) = self.registros.iter_mut().filter(|r| r.dados.get("blueprintId").and_then(Value::as_str) == Some(blueprint_id) && r.dados.get("planoId").and_then(Value::as_str) == Some(plano_id)).max_by_key(|r| (r.em_ms, r.seq)) else { return false };
        self.seq += 1;
        r.dados.insert("estado".into(), Value::String("concluido".into()));
        r.dados.insert("mensagem".into(), mensagem.clone());
        (r.em_ms, r.seq) = (agora_ms, self.seq);
        true
    }

    /// A ação e o plano em que `nome` trabalha agora neste blueprint: só vale se o último aviso é recente (`VIDA_MS`) e ele ainda não concluiu.
    pub fn plano_de(&self, agora_ms: i64, nome: &str, blueprint_id: &str) -> Option<(String, String)> {
        let r = self.registros.iter().find(|r| r.nome == nome)?;
        let texto = |c: &str| r.dados.get(c).and_then(Value::as_str).map(String::from);
        let vivo = agora_ms - r.em_ms <= VIDA_MS && texto("estado").as_deref() != Some("concluido") && texto("blueprintId").as_deref() == Some(blueprint_id);
        if !vivo {
            return None;
        }
        Some((texto("acaoId")?, texto("planoId")?))
    }

    /// Os agentes conhecidos, do mais recente ao mais antigo, cada um com `agente` e `em` (ISO).
    pub fn lista(&self) -> Vec<Value> {
        let mut todos: Vec<&Registro> = self.registros.iter().collect();
        todos.sort_by_key(|r| std::cmp::Reverse((r.em_ms, r.seq)));
        todos
            .into_iter()
            .map(|r| {
                let mut o = r.dados.clone();
                o.insert("agente".into(), Value::String(r.nome.clone()));
                o.insert("em".into(), Value::String(crate::tempo::iso_utc(r.em_ms)));
                Value::Object(o)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn dados(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn tres_agentes_aparecem_do_mais_recente_ao_mais_antigo() {
        let mut a = Atividade::default();
        a.registrar(1_000, "principal", dados(json!({"estado": "lendo", "papel": "explorador"})));
        a.registrar(2_000, "revisor", dados(json!({"estado": "testando", "papel": "revisor", "onde": ["src/"]})));
        a.registrar(3_000, "construtor", dados(json!({"estado": "escrevendo", "percentual": 40})));
        let nomes: Vec<String> = a.lista().iter().map(|x| x["agente"].as_str().unwrap().to_string()).collect();
        assert_eq!(nomes, ["construtor", "revisor", "principal"]);
        assert_eq!(a.lista()[1]["onde"], json!(["src/"]));
    }

    #[test]
    fn um_aviso_novo_atualiza_so_o_que_trouxe() {
        let mut a = Atividade::default();
        a.registrar(1_000, "ana", dados(json!({"estado": "lendo", "papel": "explorador", "onde": ["a.rs"], "percentual": 10, "planoId": "t1", "plano": "Criar"})));
        a.registrar(2_000, "ana", dados(json!({"mensagem": "só texto"})));
        let l = a.lista();
        assert_eq!(l.len(), 1);
        assert_eq!((l[0]["estado"].as_str(), l[0]["papel"].as_str(), l[0]["onde"].clone(), l[0]["percentual"].clone()), (Some("lendo"), Some("explorador"), json!(["a.rs"]), json!(10)), "o que não veio fica");
        assert_eq!((l[0]["planoId"].clone(), l[0]["plano"].clone()), (Value::Null, Value::Null), "o plano é a do último aviso");
        assert_eq!(l[0]["mensagem"], "só texto");
    }

    #[test]
    fn quem_so_manda_texto_aparece_trabalhando() {
        let mut a = Atividade::default();
        a.registrar(1_000, "agente", dados(json!({"mensagem": "lendo o código"})));
        assert_eq!(a.lista()[0]["estado"], "escrevendo");
    }

    #[test]
    fn sem_sinal_o_agente_continua_na_lista() {
        let mut a = Atividade::default();
        a.registrar(0, "velho", dados(json!({})));
        a.registrar(VIDA_MS + 5, "novo", dados(json!({})));
        let nomes: Vec<String> = a.lista().iter().map(|x| x["agente"].as_str().unwrap().to_string()).collect();
        assert_eq!(nomes, ["novo", "velho"], "quem passou de VIDA_MS sem sinal não sai: a página o põe em repouso");
    }

    #[test]
    fn o_agente_33_substitui_o_mais_antigo() {
        let mut a = Atividade::default();
        for i in 0..MAX_AGENTES as i64 {
            a.registrar(1_000 + i, &format!("a{i}"), dados(json!({})));
        }
        a.registrar(5_000, "a0", dados(json!({}))); // o a0 ganha sinal novo: o mais antigo passa a ser o a1
        a.registrar(5_001, "novo", dados(json!({})));
        let nomes: Vec<String> = a.lista().iter().map(|x| x["agente"].as_str().unwrap().to_string()).collect();
        assert_eq!(nomes.len(), MAX_AGENTES);
        assert!(nomes.contains(&"novo".to_string()) && nomes.contains(&"a0".to_string()) && !nomes.contains(&"a1".to_string()));
    }

    #[test]
    fn plano_concluido_deixa_o_agente_dele_concluido_com_a_evidencia() {
        let mut a = Atividade::default();
        a.registrar(1_000, "um", dados(json!({"blueprintId": "b", "planoId": "t1", "estado": "escrevendo"})));
        a.registrar(2_000, "outro", dados(json!({"blueprintId": "b", "planoId": "t2", "estado": "lendo"})));
        assert!(a.concluir_plano(3_000, "b", "t1", &json!("cargo test passou")));
        let l = a.lista();
        let um = l.iter().find(|x| x["agente"] == "um").unwrap();
        assert_eq!((um["estado"].as_str(), um["mensagem"].as_str()), (Some("concluido"), Some("cargo test passou")));
        assert_eq!(l.iter().find(|x| x["agente"] == "outro").unwrap()["estado"], "lendo", "quem está em outro plano não muda");
        assert!(!a.concluir_plano(4_000, "b", "t9", &json!("x")) && !a.concluir_plano(4_000, "b", "", &json!("x")), "sem agente no plano, nada muda");
    }

    #[test]
    fn a_hora_sai_em_iso() {
        let mut a = Atividade::default();
        a.registrar(1_000_000_000_123, "x", dados(json!({})));
        assert_eq!(a.lista()[0]["em"], "2001-09-09T01:46:40.123Z");
    }
}
