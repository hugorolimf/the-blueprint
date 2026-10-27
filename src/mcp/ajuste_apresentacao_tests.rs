//! O ajuste de uma apresentação que já existe: o texto do usuário limpo, o prompt nos dois idiomas, o pedido ao terminal ou a uma sessão e as ferramentas
//! `ler_apresentacao` e `editar_apresentacao` de ponta a ponta (a mesma via direta que o `/mcp` HTTP usa).
use super::hooks::travado;
use super::hooks_tests::{dados, mcp};
use super::pedido_apresentacao::{ajustar, ajustar_com, prompt_ajuste, texto_do_usuario};
use super::rotas::rota_com;
use super::stdio::tratar_via;
use crate::apresentacao::armazenar;
use crate::contexto::Contexto;
use crate::http::pedido::Pedido;
use crate::idioma::{com, Idioma};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

fn pasta_real(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-ajuste-{nome}-{}", std::process::id()));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn registro(id: &str, bp: &str) -> Value {
    json!({"id": id, "workspaceId": "sm", "blueprintId": bp, "titulo": "Plano do Portal", "publico": "cliente", "nivel": "rapido", "em": "2026-10-26T10:00:00Z",
           "conteudo": {"versao": 1, "publico": "cliente", "nivel": "rapido", "titulo": "Plano do Portal", "etapas": [{"id": "e1", "titulo": "Cadastro", "figura": "capa"}]},
           "figuras": [{"id": "capa", "titulo": "Capa", "impressao": "x", "codigo": "function (L, c) { return { svg: '<svg></svg>' }; }"}]})
}

#[test]
fn o_texto_do_usuario_entra_no_prompt_sem_quebrar_o_terminal() {
    assert_eq!(texto_do_usuario("  troque\no título;\tpor «Novo»  \r\n  agora "), "troque o título, por \"Novo\" agora", "sem controle, sem ; e sem as aspas do pedido");
    assert_eq!(texto_do_usuario("a".repeat(2000).as_str()).chars().count(), 600);
    assert_eq!(texto_do_usuario(" \n\t"), "");
}

#[test]
fn o_prompt_de_ajuste_manda_ler_mudar_so_o_pedido_e_nunca_regerar() {
    let en = com(Idioma::En, || prompt_ajuste("Portal", "sm", "p1", "0123456789abcdef", "Plano", "mude a data"));
    for trecho in ["«Plano»", "apresentacao_id 0123456789abcdef", "workspace_id sm, blueprint_id p1", "«mude a data»", "ler_apresentacao", "ONLY what was asked", "editar_apresentacao", "do not call anexar_apresentacao over it", "apresentacao_ajustar", "in one sentence"] {
        assert!(en.contains(trecho), "falta em inglês: {trecho}\n{en}");
    }
    let pt = com(Idioma::Pt, || prompt_ajuste("Portal", "sm", "p1", "0123456789abcdef", "Plano", "mude a data"));
    for trecho in ["SÓ o que foi pedido", "não a reescreva", "numa frase"] {
        assert!(pt.contains(trecho), "falta em português: {trecho}\n{pt}");
    }
    assert!(!en.contains(';') && !pt.contains(';'), "o Windows Terminal leria o ; como outro comando");
}

#[test]
fn ajustar_abre_o_terminal_na_pasta_e_recusa_o_que_nao_e_deste_blueprint() {
    let (m, d) = (mcp(), dados());
    let pasta = pasta_real("terminal");
    travado(&d).vincular_pasta(&Contexto::local(), pasta.to_str().unwrap(), "sm", "p1").unwrap();
    let ap = m.pasta_apresentacoes();
    armazenar::gravar(&ap, "1111111111111111", &registro("1111111111111111", "p1")).unwrap();
    armazenar::gravar(&ap, "2222222222222222", &registro("2222222222222222", "outro")).unwrap();
    let chamadas: RefCell<Vec<(PathBuf, String)>> = RefCell::new(vec![]);
    let falso = |p: &Path, _: &Path, a: &str| {
        chamadas.borrow_mut().push((p.into(), a.into()));
        Ok("wt")
    };
    let corpo = |extra: Value| {
        let mut c = json!({"workspaceId": "sm", "blueprintId": "p1", "apresentacaoId": "1111111111111111", "texto": "troque o título"});
        for (k, v) in extra.as_object().unwrap() {
            c[k] = v.clone();
        }
        c
    };
    let (status, r) = com(Idioma::En, || ajustar_com(&ap, &d, &corpo(json!({})), &falso)).unwrap();
    assert_eq!((status, r["terminal"].clone(), r["ok"].clone()), (200, json!("wt"), json!(true)), "{r}");
    let c = chamadas.borrow();
    let normal = |p: &Path| p.to_string_lossy().to_lowercase().replace('\\', "/");
    assert_eq!(normal(&c[0].0), normal(&pasta), "a pasta ligada");
    assert!(c[0].1.starts_with("The user asks for a change to presentation «Plano do Portal»") && c[0].1.contains("«troque o título»"), "{}", c[0].1);
    drop(c);
    // o terminal do app manda `ajuste` no lugar de `texto`
    let (status, _) = ajustar_com(&ap, &d, &corpo(json!({"texto": null, "ajuste": "outra coisa"})), &falso).unwrap();
    assert_eq!((status, chamadas.borrow()[1].1.contains("«outra coisa»")), (200, true));
    let nunca = |_: &Path, _: &Path, _: &str| -> Result<&'static str, String> { panic!("não podia abrir terminal") };
    let erro = |c: Value| {
        let e = ajustar_com(&ap, &d, &c, &nunca).unwrap_err();
        (e.status, e.codigo)
    };
    assert_eq!(erro(corpo(json!({"texto": "  \n "}))), (400, "falta-texto"));
    assert_eq!(erro(corpo(json!({"apresentacaoId": "../x"}))), (404, "apresentacao-nao-encontrada"));
    assert_eq!(erro(corpo(json!({"apresentacaoId": "2222222222222222"}))), (404, "apresentacao-nao-encontrada"), "a de outro blueprint não vale aqui");
    assert_eq!(erro(corpo(json!({"workspaceId": ""}))), (400, "faltam-ids"));
    let sem_terminal = |_: &Path, _: &Path, _: &str| -> Result<&'static str, String> { Err("não há terminal".into()) };
    let (status, r) = ajustar_com(&ap, &d, &corpo(json!({})), &sem_terminal).unwrap();
    assert_eq!((status, r["codigo"].clone()), (501, json!("sem-terminal")));
    for id in ["1111111111111111", "2222222222222222"] {
        armazenar::remover(&ap, id);
    }
}

#[test]
fn o_pedido_de_ajuste_vai_a_uma_sessao_aberta_ou_devolve_o_texto() {
    let (m, d) = (mcp(), dados());
    let pasta = pasta_real("sessao");
    let cwd = pasta.to_str().unwrap().to_string();
    travado(&d).vincular_pasta(&Contexto::local(), &cwd, "sm", "p1").unwrap();
    super::hooks::sessao(&m, &d, &json!({"session_id": "s1", "cwd": cwd})).unwrap();
    let _guarda = m.abrir_canal("abc123", &cwd).unwrap();
    let eventos = m.assinar();
    armazenar::gravar(&m.pasta_apresentacoes(), "0123456789abcdef", &registro("0123456789abcdef", "p1")).unwrap();
    let base = json!({"workspaceId": "sm", "blueprintId": "p1", "apresentacaoId": "0123456789abcdef", "texto": "deixe o tom mais leve"});
    let (status, r) = com(Idioma::En, || ajustar(&m, &d, &json!({"workspaceId": "sm", "blueprintId": "p1", "apresentacaoId": "0123456789abcdef", "texto": "deixe o tom mais leve", "sessaoId": "abc123"}))).unwrap();
    assert_eq!((status, r["terminal"].clone()), (200, json!("canal")), "{r}");
    let e = eventos.try_recv().unwrap();
    assert_eq!(e.nome, "iniciar-acao", "o mesmo evento do pedido de iniciar: só o canal endereçado o repassa como ordem do usuário");
    assert!(e.dados.unwrap()["texto"].as_str().unwrap().contains("«deixe o tom mais leve»"));
    let mut copiar = base.clone();
    copiar["sessaoId"] = json!("s1");
    copiar["semCanal"] = json!(true);
    let (status, r) = ajustar(&m, &d, &copiar).unwrap();
    assert_eq!((status, r["terminal"].clone(), eventos.try_recv().is_err()), (200, json!("copiar"), true));
    assert!(r["prompt"].as_str().unwrap().contains("editar_apresentacao"));
    armazenar::remover(&m.pasta_apresentacoes(), "0123456789abcdef");
}

fn chamada(m: &super::Mcp, d: &std::sync::Mutex<crate::dados::Dados>, nome: &str, args: Value) -> String {
    let via = |metodo: &str, rota: &str, corpo: Option<&str>| -> Result<(u16, String), String> {
        let p = Pedido::novo(metodo, &format!("/api/mcp/{rota}"), vec![("content-type".into(), "application/json".into()), ("Host".into(), "127.0.0.1:3000".into())], corpo.unwrap_or("").as_bytes().to_vec());
        let r = rota_com(m, d, &p, &Contexto::local());
        Ok((r.status, String::from_utf8_lossy(r.bytes()).to_string()))
    };
    let r = com(Idioma::En, || tratar_via(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": nome, "arguments": args}}), Some(&via))).unwrap();
    let texto = r["result"]["content"][0]["text"].as_str().unwrap().to_string();
    if r["result"]["isError"] == true { format!("ERRO: {texto}") } else { texto }
}

#[test]
fn as_ferramentas_leem_e_editam_em_passos_pequenos() {
    let (m, d) = (mcp(), dados());
    let anexo = chamada(&m, &d, "anexar_apresentacao", json!({"workspace_id": "sm", "blueprint_id": "p1", "conteudo": registro("x", "p1")["conteudo"], "figuras": registro("x", "p1")["figuras"]}));
    let id = anexo.split("apresentacao_id: ").nth(1).unwrap_or_else(|| panic!("{anexo}")).split(')').next().unwrap().to_string();
    let lido = chamada(&m, &d, "ler_apresentacao", json!({"workspace_id": "sm", "blueprint_id": "p1", "apresentacao_id": id}));
    assert!(lido.contains("audience cliente, level rapido") && lido.contains("\"titulo\":\"Plano do Portal\"") && lido.contains("codigo 50 chars"), "{lido}");
    assert!(!lido.contains("function (L, c)"), "o código só vem para o id pedido");
    let com_codigo = chamada(&m, &d, "ler_apresentacao", json!({"workspace_id": "sm", "blueprint_id": "p1", "apresentacao_id": id, "figuras_codigo": ["capa", "nada"]}));
    assert!(com_codigo.contains("codigo of figure \"capa\":\nfunction (L, c)") && com_codigo.contains("figure \"nada\" does not exist"), "{com_codigo}");
    let editado = chamada(&m, &d, "editar_apresentacao", json!({"workspace_id": "sm", "blueprint_id": "p1", "apresentacao_id": id, "operacoes": [{"op": "definir", "caminho": "etapas.0.titulo", "valor": "Cadastro e base"}, {"op": "figura_trecho", "figura": "capa", "de": "<svg></svg>", "para": "<svg id=\"a\"></svg>"}]}));
    assert!(editado.contains("2 operation(s) applied together") && editado.contains("do not regenerate"), "{editado}");
    let depois = chamada(&m, &d, "ler_apresentacao", json!({"workspace_id": "sm", "blueprint_id": "p1", "apresentacao_id": id, "figuras_codigo": ["capa"]}));
    assert!(depois.contains("Cadastro e base") && depois.contains("<svg id=\"a\"></svg>") && depois.contains("last edited") && depois.contains("by agente"), "{depois}");
    // o erro chega ao agente com o motivo e nada muda
    let ruim = chamada(&m, &d, "editar_apresentacao", json!({"workspace_id": "sm", "blueprint_id": "p1", "apresentacao_id": id, "operacoes": [{"op": "definir", "caminho": "titulo", "valor": "Perdido"}, {"op": "figura_trecho", "figura": "capa", "de": "nao-existe", "para": "x"}]}));
    assert!(ruim.starts_with("ERRO:") && ruim.contains("operation 1") && ruim.contains("0 matches"), "{ruim}");
    assert!(chamada(&m, &d, "ler_apresentacao", json!({"workspace_id": "sm", "blueprint_id": "p1", "apresentacao_id": id})).contains("\"titulo\":\"Plano do Portal\""), "tudo ou nada");
    assert!(chamada(&m, &d, "ler_apresentacao", json!({"workspace_id": "sm", "blueprint_id": "p1", "apresentacao_id": "ffffffffffffffff"})).starts_with("ERRO:"));
    assert!(chamada(&m, &d, "editar_apresentacao", json!({"workspace_id": "sm", "blueprint_id": "p1", "apresentacao_id": id})).starts_with("ERRO:"), "sem operações não há o que editar");
    armazenar::remover(&m.pasta_apresentacoes(), &id);
}
