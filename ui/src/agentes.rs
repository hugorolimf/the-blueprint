//! Os agentes que trabalham no workspace e onde a folha os põe (`agentes.js` e `claude-logica.js` da página): quem é o principal, o papel e a pose de cada
//! figura, o lugar (a linha do plano, a barra, o cabeçalho da seção ou do blueprint, o cartão) e a posição lado a lado. Só lógica: o desenho é de `folha/claude.rs`.

use serde::Deserialize;
use std::collections::HashSet;

/// O que o servidor diz de um agente (`GET /api/mcp/agentes`).
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Agente {
    pub agente: String,
    pub papel: Option<String>,
    pub workspace_id: String,
    pub blueprint_id: String,
    pub acao_id: Option<String>,
    pub plano_id: Option<String>,
    pub estado: Option<String>,
    /// quando avisou pela última vez (ISO)
    pub em: String,
    pub mensagem: Option<String>,
    pub tipo: Option<String>,
}

/// Largura do Claude na folha, a folga entre figuras e o limite de uma fileira (5 figuras); o resto se junta, sobreposto.
pub const LARGURA: f32 = 48.0;
pub const ALTURA: f32 = 37.0;
const FOLGA: f32 = 4.0;
const LINHA_MAX: f32 = 5.0 * LARGURA + 4.0 * FOLGA;
/// Quantos subagentes aparecem (além do principal); o resto fica sob a pilha.
const MAX_SUB: usize = 7;
/// O subagente que concluiu some da folha depois disto.
const SUB_SOME_MS: i64 = 10_000;
const PAPEIS: [&str; 3] = ["explorador", "construtor", "revisor"];

/// Dias desde 1970 de `aaaa-mm-dd` (algoritmo de Howard Hinnant), só para a idade do sinal de vida do agente; `None` se não for esse formato.
fn dia(iso: &str) -> Option<i64> {
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

/// `aaaa-mm-ddThh:mm:ss.mmmZ` em milissegundos desde 1970 (0 se não entender).
pub fn em_ms(iso: &str) -> i64 {
    let (Some(d), true) = (iso.get(..10).and_then(dia), iso.len() >= 19) else { return 0 };
    let n = |a: usize, b: usize| iso.get(a..b).and_then(|t| t.parse::<i64>().ok()).unwrap_or(0);
    let ms = iso.get(20..23).and_then(|t| t.parse::<i64>().ok()).unwrap_or(0);
    d * 86_400_000 + n(11, 13) * 3_600_000 + n(14, 16) * 60_000 + n(17, 19) * 1000 + ms
}

/// Agora, em milissegundos desde 1970 (no navegador e no desktop).
pub fn agora_ms() -> i64 {
    web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as i64)
}

/// O estado que a figura entende (um evento sem `estado` vira o do tipo dele).
pub fn estado(a: &Agente) -> Option<&str> {
    a.estado.as_deref().or(match a.tipo.as_deref() {
        Some("bloqueio") => Some("bloqueado"),
        Some("concluir") => Some("concluido"),
        Some("progresso") => Some("escrevendo"),
        _ => None,
    })
}

/// A pose pelo estado (`poseDoEstado`, sem a pose «andando»).
pub fn pose(estado: Option<&str>) -> &'static str {
    match estado {
        Some("bloqueado") => "bloqueado",
        Some("concluido") => "concluido",
        Some("aguardando") => "esperando",
        Some("lendo" | "escrevendo" | "testando") => "trabalhando",
        _ => "parado",
    }
}

fn repousa(a: &Agente, agora: i64, vida: i64) -> bool {
    a.estado.as_deref() == Some("ocioso") || (a.estado.as_deref() != Some("concluido") && agora >= em_ms(&a.em) + vida)
}

/// A chave de «já vi»: o aviso novo do mesmo agente (outro `em`) volta a precisar de clique.
pub fn chave_visto(a: &Agente) -> String {
    format!("{}|{}|{}", a.workspace_id, a.agente, a.em)
}

/// O principal: o agente sem papel ou, se todos têm papel, o mais antigo; no empate, pelo nome.
pub fn principal(ags: &[Agente]) -> Option<usize> {
    let sem_papel: Vec<usize> = (0..ags.len()).filter(|i| ags[*i].papel.is_none()).collect();
    let universo: Vec<usize> = if sem_papel.is_empty() { (0..ags.len()).collect() } else { sem_papel };
    let mut v = universo;
    v.sort_by(|&x, &y| {
        let antigo = if ags[x].papel.is_none() { std::cmp::Ordering::Equal } else { em_ms(&ags[x].em).cmp(&em_ms(&ags[y].em)) };
        antigo.then_with(|| ags[x].agente.cmp(&ags[y].agente))
    });
    v.first().copied()
}

/// Os agentes do workspace que aparecem: quem repousa vira `ocioso`, sem ação nem plano (parado no cabeçalho); o subagente que concluiu some depois de 10 s;
/// quem o usuário já viu (concluído, aguardando ou parado) sai.
pub fn ativos(todos: &[Agente], workspace: &str, agora: i64, vida: i64, vistos: &HashSet<String>) -> Vec<Agente> {
    let doa: Vec<Agente> = todos.iter().filter(|a| a.workspace_id == workspace).cloned().collect();
    let principal_do_ws = principal(&doa);
    doa.iter().enumerate().filter(|(i, a)| !(a.estado.as_deref() == Some("concluido") && a.papel.is_some() && Some(*i) != principal_do_ws && agora >= em_ms(&a.em) + SUB_SOME_MS)).filter_map(|(_, a)| {
        let repouso = repousa(a, agora, vida);
        let parado = repouso || matches!(a.estado.as_deref(), Some("concluido" | "aguardando" | "ocioso"));
        if parado && vistos.contains(&chave_visto(a)) {
            return None;
        }
        Some(if repouso { Agente { estado: Some("ocioso".into()), acao_id: None, plano_id: None, ..a.clone() } } else { a.clone() })
    }).collect()
}

/// O papel da figura: o principal nunca carrega ferramenta; o subagente sem papel válido vira construtor.
pub fn papel_da_figura(a: &Agente, e_principal: bool) -> Option<&'static str> {
    if e_principal {
        return None;
    }
    PAPEIS.iter().find(|p| Some(**p) == a.papel.as_deref()).copied().or(Some("construtor"))
}

/// O lugar da folha onde o agente fica, do mais preciso ao mais geral.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Lugar {
    Plano(String),
    Barra(String),
    Secao(String),
    Cab(String),
}

pub fn lugar(a: &Agente, plano_visivel: impl Fn(&str) -> bool, barra: impl Fn(&str) -> bool, secao: impl Fn(&str) -> Option<String>) -> Lugar {
    if let Some(p) = a.plano_id.as_deref().filter(|p| plano_visivel(p)) {
        return Lugar::Plano(p.into());
    }
    if let Some(b) = a.acao_id.as_deref().filter(|b| barra(b)) {
        return Lugar::Barra(b.into());
    }
    if let Some(s) = a.acao_id.as_deref().and_then(secao) {
        return Lugar::Secao(s);
    }
    Lugar::Cab(a.blueprint_id.clone())
}

pub struct Item {
    pub agente: Agente,
    pub papel: Option<&'static str>,
    pub lugar: Lugar,
    /// 0 = o mais perto do lugar
    pub indice: usize,
    pub extras: usize,
}

/// As figuras de um blueprint: o principal primeiro, depois os subagentes (quem avisou há menos tempo antes), no máximo 1 + 7.
pub fn itens(ags: &[Agente], plano_visivel: impl Fn(&str) -> bool, barra: impl Fn(&str) -> bool, secao: impl Fn(&str) -> Option<String>) -> Vec<Item> {
    let Some(p) = principal(ags) else { return Vec::new() };
    let mut resto: Vec<usize> = (0..ags.len()).filter(|i| *i != p).collect();
    resto.sort_by_key(|i| std::cmp::Reverse(em_ms(&ags[*i].em)));
    let ordem: Vec<usize> = std::iter::once(p).chain(resto).collect();
    let extras = ordem.len().saturating_sub(1 + MAX_SUB);
    let mut itens: Vec<Item> = ordem.iter().take(1 + MAX_SUB).map(|&i| {
        let a = &ags[i];
        Item { papel: papel_da_figura(a, i == p), agente: a.clone(), lugar: lugar(a, &plano_visivel, &barra, &secao), indice: 0, extras: 0 }
    }).collect();
    // o índice é a ordem de chegada ao lugar (0 = o mais perto)
    let mut chegada: std::collections::HashMap<Lugar, usize> = std::collections::HashMap::new();
    for it in &mut itens {
        let n = chegada.entry(it.lugar.clone()).or_insert(0);
        it.indice = *n;
        *n += 1;
    }
    if let Some(ultimo) = itens.last_mut() {
        ultimo.extras = extras;
    }
    itens
}

/// Onde a figura fica: o canto de cima à esquerda `(x, y)` dado o apoio. `tipo`: `plano` (à esquerda da linha), `cartao` (no canto de cima do cartão da lista) ou
/// cabeçalho/barra (à esquerda dos medidores do canto). `ancora` = `(x, y, w, h)` do lugar; `larguras` = as dos itens do lugar, na ordem do índice.
pub fn posicao(tipo: &str, ancora: (f32, f32, f32, f32), indice: usize, larguras: &[f32]) -> (f32, f32) {
    let (x, y, w, h) = ancora;
    let (dir, base) = match tipo {
        "plano" => (x - 6.0, y + h + 5.0),
        "cartao" => (x + w - 10.0, y + 4.0),
        _ => (x + w - 158.0, y + h - 8.0),
    };
    let passos = |ate: usize| larguras.iter().skip(1).take(ate).map(|l| l + FOLGA).sum::<f32>();
    let resto = passos(larguras.len());
    let k = if resto > 0.0 { ((LINHA_MAX - larguras[0]) / resto).min(1.0) } else { 1.0 };
    ((dir - larguras[0] - k * passos(indice)).round(), (base - ALTURA).round())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn ag(nome: &str, papel: Option<&str>, em: &str) -> Agente {
        Agente { agente: nome.into(), papel: papel.map(Into::into), workspace_id: "sm".into(), blueprint_id: "b".into(), em: em.into(), estado: Some("escrevendo".into()), ..Default::default() }
    }

    #[test]
    fn principal_e_o_sem_papel_ou_o_mais_antigo() {
        let a = [ag("Zeca", Some("construtor"), "2026-11-01T10:00:00.000Z"), ag("Claude", None, "2026-11-01T11:00:00.000Z"), ag("Ana", Some("revisor"), "2026-11-01T09:00:00.000Z")];
        assert_eq!(principal(&a), Some(1));
        assert_eq!(principal(&[a[0].clone(), a[2].clone()]), Some(1), "todos com papel: o mais antigo");
        assert_eq!(papel_da_figura(&a[1], true), None);
        assert_eq!(papel_da_figura(&ag("X", Some("inventado"), ""), false), Some("construtor"));
    }

    #[test]
    fn datas_iso_viram_milissegundos() {
        assert_eq!(em_ms("1970-01-02T00:00:01.500Z"), 86_400_000 + 1500);
        assert_eq!(em_ms("lixo"), 0);
    }

    #[test]
    fn lugar_do_mais_preciso_ao_mais_geral() {
        let mut a = ag("C", None, "");
        a.plano_id = Some("p".into());
        a.acao_id = Some("f".into());
        let tudo = |_: &str| true;
        let nada = |_: &str| false;
        assert_eq!(lugar(&a, tudo, nada, |_| None), Lugar::Plano("p".into()));
        assert_eq!(lugar(&a, nada, tudo, |_| None), Lugar::Barra("f".into()));
        assert_eq!(lugar(&a, nada, nada, |_| Some("b|".into())), Lugar::Secao("b|".into()));
        assert_eq!(lugar(&a, nada, nada, |_| None), Lugar::Cab("b".into()));
    }

    #[test]
    fn repouso_some_e_concluido_do_sub_sai() {
        let velho = "2026-11-01T00:00:00.000Z";
        let agora = em_ms(velho) + 11 * 60_000;
        let todos = [ag("Claude", None, velho), Agente { estado: Some("concluido".into()), ..ag("Sub", Some("revisor"), velho) }];
        let a = ativos(&todos, "sm", agora, 600_000, &HashSet::new());
        assert_eq!(a.len(), 1, "o subagente que concluiu há mais de 10 s sai");
        assert_eq!(a[0].estado.as_deref(), Some("ocioso"), "10 min sem aviso: repousa");
        let vistos: HashSet<String> = [chave_visto(&todos[0])].into();
        assert!(ativos(&todos, "sm", agora, 600_000, &vistos).is_empty(), "repouso visto some");
    }

    #[test]
    fn lado_a_lado_e_o_principal_primeiro() {
        let a = [ag("Explorador", Some("explorador"), "2026-11-01T10:00:00.000Z"), ag("Claude", None, "2026-11-01T09:00:00.000Z")];
        let it = itens(&a, |_| false, |_| false, |_| None);
        assert_eq!((it[0].agente.agente.as_str(), it[0].indice, it[1].indice), ("Claude", 0, 1));
        assert_eq!(posicao("plano", (100.0, 50.0, 400.0, 22.0), 0, &[48.0, 48.0]).0, 100.0 - 6.0 - 48.0);
        assert_eq!(posicao("plano", (100.0, 50.0, 400.0, 22.0), 1, &[48.0, 48.0]).0, 100.0 - 6.0 - 48.0 - 52.0);
    }
}
