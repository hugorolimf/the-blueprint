//! Testes do compartilhamento: as rotas pelas duas portas (sem socket), a API, o túnel falso (um script que imprime a URL) e o hash.
mod api;
mod portas;
mod tunel;

use super::*;
use crate::http::pedido::{Pedido, Resposta};
use std::path::Path;

pub const URL: &str = "https://teste-falso-tunel.trycloudflare.com";
pub const HOST: &str = "teste-falso-tunel.trycloudflare.com";

impl Compartilhar {
    /// Liga sem túnel nem listener: só o estado que a porta dos convidados e as rotas locais leem.
    pub fn ligado_para_teste(&self, url: &str, workspace: &str) {
        let mut i = self.travar();
        let g = i.geracao + 1;
        *i = Interno { fase: Fase::Ligado, workspaces: vec![workspace.into()], url: Some(url.into()), hospedagem: Some(Hospedagem::de(url, "127.0.0.1", None, false).unwrap()), geracao: g, ..Interno::default() };
    }
}

/// O estado dos testes do http (raiz `sm` com o blueprint p1, workspace w2 com o p9), compartilhando `ws`.
pub fn estado_compartilhando(ws: &str) -> Estado {
    let e = crate::http::tests::estado(false);
    e.compartilhar.ligado_para_teste(URL, ws);
    e
}

/// Um pedido como o túnel o entrega à porta dos convidados: par em loopback, `X-Forwarded-Proto: https`, Host e Origin do túnel.
pub fn convidado(metodo: &str, url: &str, cookie: Option<&str>, corpo: Option<Value>) -> Pedido {
    let mut cab: Vec<(String, String)> = vec![("Host".into(), HOST.into()), ("X-Forwarded-Proto".into(), "https".into()), ("X-Forwarded-For".into(), "203.0.113.9".into()), ("Origin".into(), URL.into())];
    if let Some(c) = cookie {
        cab.push(("Cookie".into(), format!("bp_sessao={c}")));
    }
    if corpo.is_some() {
        cab.push(("Content-Type".into(), "application/json".into()));
    }
    let mut p = Pedido::novo(metodo, url, cab, corpo.map(|c| c.to_string().into_bytes()).unwrap_or_default());
    p.ip = "127.0.0.1".into();
    p
}

/// Um pedido da página local (porta 3000): Host e Origin em loopback.
pub fn local(metodo: &str, url: &str, corpo: Option<Value>) -> Pedido {
    let mut cab: Vec<(String, String)> = vec![("Host".into(), "127.0.0.1:3000".into()), ("Origin".into(), "http://127.0.0.1:3000".into())];
    if corpo.is_some() {
        cab.push(("Content-Type".into(), "application/json".into()));
    }
    let mut p = Pedido::novo(metodo, url, cab, corpo.map(|c| c.to_string().into_bytes()).unwrap_or_default());
    p.ip = "127.0.0.1".into();
    p
}

pub fn pela_porta_dos_convidados(e: &Estado, p: &Pedido) -> Resposta {
    crate::http::rotas::despachar_convidado(e, p)
}

pub fn pela_porta_local(e: &Estado, p: &Pedido) -> Resposta {
    crate::http::rotas::despachar(e, p)
}

pub fn json_de(r: &Resposta) -> Value {
    serde_json::from_slice(r.bytes()).unwrap_or(Value::Null)
}

pub fn codigo(r: &Resposta) -> String {
    json_de(r)["codigo"].as_str().unwrap_or("").to_string()
}

/// O dono local cria um convite pela porta local e o convidado entra pela porta dos convidados: devolve o cookie dele.
pub fn convidado_entra(e: &Estado, ws: &str, papel: &str, nome: &str) -> String {
    let r = pela_porta_local(e, &local("POST", "/api/convites", Some(json!({"workspaceId": ws, "papel": papel}))));
    assert_eq!(r.status, 201, "{}", String::from_utf8_lossy(r.bytes()));
    let link = json_de(&r)["link"].as_str().unwrap().to_string();
    let segredo = link.split_once('#').unwrap().1.to_string();
    let r = pela_porta_dos_convidados(e, &convidado("POST", "/api/convite/entrar", None, Some(json!({"segredo": segredo, "nome": nome}))));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    r.cabecalho("set-cookie").unwrap().trim_start_matches("bp_sessao=").split(';').next().unwrap().to_string()
}

/// Uma pasta temporária nova para o teste.
pub fn pasta(nome: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-compartilhar-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// Um cloudflared falso: anota os argumentos em `args.txt`, imprime na saída de erro as URLs que o de verdade imprime (termos de uso e
/// a do túnel) e espera `segundos` antes de sair.
pub fn tunel_falso(dir: &Path, segundos: u32) -> std::path::PathBuf {
    let args = dir.join("args.txt");
    #[cfg(windows)]
    let (caminho, texto) = (
        dir.join("cloudflared-falso.cmd"),
        format!("@echo off\r\necho %* > \"{}\"\r\necho INF terms https://www.cloudflare.com/website-terms/ 1>&2\r\necho INF {URL} 1>&2\r\nping -n {} 127.0.0.1 >nul\r\n", args.display(), segundos + 1),
    );
    #[cfg(not(windows))]
    let (caminho, texto) = (dir.join("cloudflared-falso.sh"), format!("#!/bin/sh\necho \"$@\" > '{}'\necho 'INF terms https://www.cloudflare.com/website-terms/' >&2\necho 'INF {URL}' >&2\nexec sleep {segundos}\n", args.display()));
    std::fs::write(&caminho, texto).unwrap();
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    caminho
}

/// Espera o compartilhamento chegar a `fase` (até 20 s).
pub fn esperar(c: &Compartilhar, fase: &str) -> Value {
    for _ in 0..400 {
        let j = c.json();
        if j["estado"] == fase {
            return j;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    panic!("não chegou a {fase}: {}", c.json());
}

#[test]
fn so_do_workspace_corta_os_outros_e_o_de_todos() {
    let ctx = Contexto::membro("m1", "Ana", vec![Acesso::novo("*", "", Papel::Dono), Acesso::novo("w2", "", Papel::Colaborador), Acesso::novo("sm", "p1", Papel::Supervisor)]);
    let c = so_do_workspace(&ctx, &["w2".into()]);
    assert_eq!((c.acessos.len(), c.papel, c.local), (1, Papel::Colaborador, false));
    assert!(c.pode(crate::contexto::Acao::Gravar, "w2", Some("p9")) && !c.pode(crate::contexto::Acao::Ler, "sm", Some("p1")));
    assert!(!so_do_workspace(&Contexto::local(), &["w2".into()]).local, "a porta dos convidados nunca vira o dono local");
    assert!(!so_do_workspace(&ctx, &["w3".into()]).pode(crate::contexto::Acao::Ler, "w3", None));
}
