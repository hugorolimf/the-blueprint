//! A pesquisa, sem tela (`web/js/pesquisa-logica.js`): sem acento e sem caixa; a consulta vira termos e TODOS têm de aparecer no item (nos campos
//! dele, juntos). Cada campo diz onde achou. Os trechos voltam como `[início, fim)` em letras do texto original, para destacar.

use serde_json::Value;

/// Sem acento e minúsculo, letra por letra (o tamanho não muda: os trechos valem no texto original).
pub fn norma(texto: &str) -> String {
    texto.chars().map(|c| {
        let m = c.to_lowercase().next().unwrap_or(c);
        match m {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            m => m,
        }
    }).collect()
}

/// Os termos da consulta, sem repetir.
pub fn termos(consulta: &str) -> Vec<String> {
    let mut vistos: Vec<String> = Vec::new();
    for t in norma(consulta).split_whitespace() {
        if !vistos.iter().any(|v| v == t) {
            vistos.push(t.to_string());
        }
    }
    vistos
}

/// Os trechos do texto onde algum termo aparece (juntados quando se tocam), em letras.
pub fn trechos(texto: &str, termos: &[String]) -> Vec<(usize, usize)> {
    let letras: Vec<char> = norma(texto).chars().collect();
    let mut achados = Vec::new();
    for t in termos {
        let t: Vec<char> = t.chars().collect();
        if t.is_empty() || t.len() > letras.len() {
            continue;
        }
        let mut i = 0;
        while i + t.len() <= letras.len() {
            if letras[i..i + t.len()] == t[..] {
                achados.push((i, i + t.len()));
                i += t.len();
            } else {
                i += 1;
            }
        }
    }
    achados.sort();
    let mut fim: Vec<(usize, usize)> = Vec::new();
    for (a, b) in achados {
        match fim.last_mut() {
            Some(u) if a <= u.1 => u.1 = u.1.max(b),
            _ => fim.push((a, b)),
        }
    }
    fim
}

/// `campos`: `(nome do campo, texto)`. Casa quando cada termo aparece em algum campo; devolve os campos em que algo apareceu, ou `None`.
pub fn campos(campos: &[(&'static str, &str)], termos: &[String]) -> Option<Vec<&'static str>> {
    if termos.is_empty() {
        return None;
    }
    let ns: Vec<(&'static str, String)> = campos.iter().map(|(k, v)| (*k, norma(v))).collect();
    if !termos.iter().all(|t| ns.iter().any(|(_, s)| s.contains(t.as_str()))) {
        return None;
    }
    Some(ns.iter().filter(|(_, s)| termos.iter().any(|t| s.contains(t.as_str()))).map(|(k, _)| *k).collect())
}

fn texto<'a>(v: &'a Value, campo: &str) -> &'a str {
    v.get(campo).and_then(Value::as_str).unwrap_or("")
}

fn lista<'a>(v: &'a Value, campo: &str) -> &'a [Value] {
    v.get(campo).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

/// O que casou numa ação: ela mesma (`propria`: onde) e os planos e checkpoints dela.
pub struct ResAcao {
    pub id: String,
    pub propria: Option<Vec<&'static str>>,
    pub planos: Vec<(String, Vec<&'static str>)>,
    pub checkpoints: Vec<(String, Vec<&'static str>)>,
}

#[derive(Default)]
pub struct ResBlueprint {
    /// o nome ou a descrição do blueprint casou
    pub proprio: bool,
    pub acoes: Vec<ResAcao>,
    pub n_acoes: usize,
    pub n_planos: usize,
    pub n_checkpoints: usize,
}

impl ResBlueprint {
    pub fn total(&self) -> usize {
        self.n_acoes + self.n_planos + self.n_checkpoints
    }
    /// O blueprint aparece na lista do workspace: nome, descrição ou algo dentro.
    pub fn casa(&self) -> bool {
        self.proprio || !self.acoes.is_empty()
    }
}

/// Um blueprint inteiro. `visiveis`: os ids das ações que a folha mostra (o filtro «Pendentes» pode tirar algumas).
pub fn blueprint(p: &Value, termos: &[String], visiveis: Option<&[&str]>) -> ResBlueprint {
    let mut r = ResBlueprint { proprio: campos(&[("nome", texto(p, "nome")), ("descricao", texto(p, "descricao"))], termos).is_some(), ..Default::default() };
    if termos.is_empty() {
        return r;
    }
    for f in lista(p, "acoes").iter().filter(|f| visiveis.is_none_or(|v| v.contains(&texto(f, "id")))) {
        let propria = campos(&[("nome", texto(f, "nome")), ("instrucoes", texto(f, "obs"))], termos);
        let planos: Vec<(String, Vec<&'static str>)> = lista(f, "planos").iter().filter_map(|t| campos(&[("nome", texto(t, "texto")), ("descricao", texto(t, "descricao"))], termos).map(|o| (texto(t, "id").to_string(), o))).collect();
        let checkpoints: Vec<(String, Vec<&'static str>)> = lista(f, "checkpoints").iter().filter_map(|m| campos(&[("nome", texto(m, "nome"))], termos).map(|o| (texto(m, "id").to_string(), o))).collect();
        if propria.is_none() && planos.is_empty() && checkpoints.is_empty() {
            continue;
        }
        r.n_acoes += usize::from(propria.is_some());
        r.n_planos += planos.len();
        r.n_checkpoints += checkpoints.len();
        r.acoes.push(ResAcao { id: texto(f, "id").to_string(), propria, planos, checkpoints });
    }
    r
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn sem_acento_sem_caixa_e_todos_os_termos() {
        assert_eq!(norma("Ação Padrão"), "acao padrao");
        let t = termos("  API   pedidos api ");
        assert_eq!(t, ["api", "pedidos"]);
        assert_eq!(campos(&[("nome", "API de Pedidos"), ("descricao", "")], &t), Some(vec!["nome"]));
        assert_eq!(campos(&[("nome", "API de entregas")], &t), None, "falta um termo");
        assert_eq!(campos(&[("nome", "API")], &[]), None);
    }

    #[test]
    fn trechos_juntam_os_que_se_tocam() {
        let t = termos("pedi ped");
        assert_eq!(trechos("API de Pedidos", &t), [(7, 11)]);
        assert_eq!(trechos("Ação e ACAO", &termos("acao")), [(0, 4), (7, 11)]);
        assert!(trechos("nada", &termos("xyz")).is_empty());
    }

    #[test]
    fn blueprint_acha_acao_plano_e_checkpoint() {
        let d: Value = serde_json::from_str(include_str!("../../data.exemplo.json")).unwrap();
        let p = &d["blueprints"][0];
        let r = blueprint(p, &termos("endpoint"), None);
        // os dois planos «Criar o endpoint…» e o que mais citar o termo nas instruções ou descrições
        assert!(!r.proprio && r.n_planos >= 2 && r.n_checkpoints == 0 && r.casa());
        let c = blueprint(p, &termos("entregador"), None);
        assert!(c.casa() && c.proprio, "o nome do blueprint não tem entregador, a descrição tem");
        assert!(blueprint(p, &termos("zzzz"), None).acoes.is_empty());
    }
}
