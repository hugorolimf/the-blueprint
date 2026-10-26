//! As equipes a que este app já entrou pelo link de convite (`%APPDATA%\Blueprint\equipes.json`). A sessão em si é o cookie da janela da equipe
//! (perfil persistente do WebView2, 30 dias); aqui fica só o que permite voltar: o endereço e os nomes que a página da equipe informou. Sem segredo
//! nenhum (o convite e o cookie nunca passam por aqui). A lista aparece na tela de workspaces do app e abre a janela da equipe por clique.
use crate::registrar::conectar::validar_endereco;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const ARQUIVO: &str = "equipes.json";
/// O link interno que a rota de abrir entrega à janela (só dentro do processo): `blueprint://equipe?u=<origem>`.
pub const PREFIXO_LINK: &str = "blueprint://equipe?u=";
/// shortcut: um teto simples contra lixo; trocar por expiração das equipes que não abrem há meses se alguém juntar dezenas.
const MAXIMO: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Espaco {
    pub id: String,
    pub nome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Equipe {
    /// `https://host[:porta]`, sem caminho
    pub origem: String,
    /// o nome da pessoa nesta equipe
    pub membro: String,
    /// os workspaces que a pessoa vê lá (só nomes, para a lista local)
    pub workspaces: Vec<Espaco>,
}

fn texto(v: &Value, chave: &str) -> String {
    v.get(chave).and_then(Value::as_str).unwrap_or_default().to_string()
}

fn espacos(v: &Value) -> Vec<Espaco> {
    v.as_array().map(|a| a.iter().map(|w| Espaco { id: texto(w, "id"), nome: texto(w, "nome") }).collect()).unwrap_or_default()
}

impl Equipe {
    pub fn json(&self) -> Value {
        json!({"origem": self.origem, "membro": self.membro, "workspaces": self.workspaces.iter().map(|w| json!({"id": w.id, "nome": w.nome})).collect::<Vec<_>>()})
    }
}

/// A lista como a página a lê (`GET /api/equipes`).
pub fn lista_json(pasta: &Path) -> Value {
    Value::Array(ler(pasta).iter().map(Equipe::json).collect())
}

fn arquivo(pasta: &Path) -> PathBuf {
    pasta.join(ARQUIVO)
}

pub fn ler(pasta: &Path) -> Vec<Equipe> {
    let lido: Value = std::fs::read_to_string(arquivo(pasta)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null);
    let itens = lido.as_array().map(Vec::as_slice).unwrap_or_default();
    itens.iter().map(|e| Equipe { origem: texto(e, "origem"), membro: texto(e, "membro"), workspaces: espacos(&e["workspaces"]) }).filter(|e| validar_endereco(&e.origem, false).is_ok()).collect()
}

fn gravar(pasta: &Path, lista: &[Equipe]) {
    let _ = std::fs::create_dir_all(pasta);
    let itens: Vec<Value> = lista.iter().map(Equipe::json).collect();
    if let Ok(texto) = serde_json::to_string_pretty(&itens) {
        let _ = std::fs::write(arquivo(pasta), texto);
    }
}

/// Anota a equipe (nova ou já conhecida). Com `detalhes`, troca o nome e os workspaces; sem, só garante que ela está na lista.
pub fn lembrar(pasta: &Path, origem: &str, detalhes: Option<(String, Vec<Espaco>)>) {
    let Ok(origem) = validar_endereco(origem, false) else { return };
    let mut lista = ler(pasta);
    let i = lista.iter().position(|e| e.origem == origem).unwrap_or_else(|| {
        lista.push(Equipe { origem, ..Equipe::default() });
        lista.len() - 1
    });
    if let Some((membro, workspaces)) = detalhes {
        lista[i].membro = membro.chars().take(60).collect();
        lista[i].workspaces = workspaces.into_iter().take(50).map(|w| Espaco { id: w.id.chars().take(80).collect(), nome: w.nome.chars().take(120).collect() }).collect();
    }
    if lista.len() > MAXIMO {
        lista.remove(0);
    }
    gravar(pasta, &lista);
}

pub fn esquecer(pasta: &Path, origem: &str) {
    let mut lista = ler(pasta);
    lista.retain(|e| e.origem != origem);
    gravar(pasta, &lista);
}

pub fn conhecida(pasta: &Path, origem: &str) -> bool {
    ler(pasta).iter().any(|e| e.origem == origem)
}

/// O que a página da equipe manda pelo IPC (`equipe {json}`): `{membro, workspaces:[{id,nome}]}`.
pub fn detalhes_do_ipc(corpo: &str) -> Option<(String, Vec<Espaco>)> {
    let v: Value = serde_json::from_str(corpo).ok()?;
    let membro = v.get("membro")?.as_str()?.to_string();
    v.get("workspaces")?.as_array()?;
    Some((membro, espacos(&v["workspaces"])))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pasta(nome: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("blueprint-equipes-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn lembra_atualiza_e_esquece() {
        let p = pasta("ciclo");
        lembrar(&p, "https://abc.trycloudflare.com/", None);
        assert_eq!(ler(&p), vec![Equipe { origem: "https://abc.trycloudflare.com".into(), ..Equipe::default() }]);
        lembrar(&p, "https://abc.trycloudflare.com", Some(("Ana".into(), vec![Espaco { id: "w2".into(), nome: "App".into() }])));
        lembrar(&p, "https://abc.trycloudflare.com", None); // sem detalhes não apaga os que já tinha
        let l = ler(&p);
        assert_eq!((l.len(), l[0].membro.as_str(), l[0].workspaces.len()), (1, "Ana", 1));
        assert!(conhecida(&p, "https://abc.trycloudflare.com"));
        esquecer(&p, "https://abc.trycloudflare.com");
        assert!(ler(&p).is_empty());
        let _ = std::fs::remove_dir_all(&p);
    }

    #[test]
    fn recusa_endereco_que_nao_e_https() {
        let p = pasta("ruim");
        for ruim in ["http://abc.com", "https://abc.com/caminho", "https://a.com@evil.com", "javascript:alert(1)"] {
            lembrar(&p, ruim, None);
        }
        assert!(ler(&p).is_empty());
        let _ = std::fs::remove_dir_all(&p);
    }

    #[test]
    fn detalhes_do_ipc_so_aceita_o_formato_esperado() {
        assert!(detalhes_do_ipc(r#"{"membro":"Ana","workspaces":[{"id":"w2","nome":"App"}]}"#).is_some());
        assert!(detalhes_do_ipc(r#"{"membro":"Ana"}"#).is_none());
        assert!(detalhes_do_ipc("lixo").is_none());
    }
}
