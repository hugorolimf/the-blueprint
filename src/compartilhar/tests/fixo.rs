//! O endereço fixo (Tailscale Funnel): as leituras do `tailscale status/serve --json`, o endereço impresso, a escolha guardada e o ciclo de ligar e desligar com um
//! `tailscale` falso (um .cmd): o mesmo endereço nas duas vezes, a primeira porta livre do Funnel, nada de rede e nada de Tailscale de verdade.
use super::*;
use crate::compartilhar::provedor::{self, Provedor};
use crate::compartilhar::tailscale;

const MAQUINA: &str = "maq.tail1234.ts.net";

#[test]
fn le_o_nome_da_maquina_so_com_o_tailscale_conectado() {
    let ok = |estado: &str, nome: &str| format!(r#"{{"BackendState":"{estado}","Self":{{"DNSName":"{nome}"}}}}"#);
    assert_eq!(tailscale::nome_da_maquina(&ok("Running", "Maq.Tail1234.ts.net.")).unwrap(), MAQUINA, "minúsculas e sem o ponto final");
    for (json, parte) in [(ok("Stopped", "m.t.ts.net."), "Stopped"), (ok("NeedsLogin", "m.t.ts.net."), "NeedsLogin"), (ok("Running", ""), "ts.net"), (ok("Running", "m.exemplo.com."), "ts.net"), (ok("Running", "m t.ts.net."), "ts.net"), ("nao json".into(), "JSON")] {
        let e = tailscale::nome_da_maquina(&json).unwrap_err();
        assert!(e.contains(parte), "{json}: {e}");
    }
}

#[test]
fn escolhe_a_primeira_porta_do_funnel_que_o_serve_nao_usa() {
    assert_eq!(tailscale::porta_livre("").unwrap(), 443);
    assert_eq!(tailscale::porta_livre("{}").unwrap(), 443);
    assert_eq!(tailscale::porta_livre(r#"{"TCP":{"443":{"HTTPS":true}}}"#).unwrap(), 8443, "443 em uso pelo Serve do usuário: nunca tomar a porta dele");
    assert_eq!(tailscale::porta_livre(r#"{"TCP":{"443":{"HTTPS":true},"8443":{"HTTPS":true}}}"#).unwrap(), 10000);
    assert!(tailscale::porta_livre(r#"{"TCP":{"443":{},"8443":{},"10000":{}}}"#).unwrap_err().contains("10000"));
    assert!(tailscale::porta_livre("lixo").is_err());
}

#[test]
fn o_endereco_impresso_so_vale_se_for_ts_net() {
    assert_eq!(tailscale::endereco("https://maq.tail1234.ts.net/").as_deref(), Some("https://maq.tail1234.ts.net"));
    assert_eq!(tailscale::endereco("Available on the internet: https://Maq.Tail1234.ts.net:8443/ (Funnel on)").as_deref(), Some("https://maq.tail1234.ts.net:8443"));
    assert_eq!(tailscale::endereco("https://maq.tail1234.ts.net:443").as_deref(), Some("https://maq.tail1234.ts.net"), "443 é a porta padrão");
    for linha in ["", "https://tailscale.com/kb/1223/funnel", "https://maq.exemplo.com/", "http://maq.tail1234.ts.net/", "https://.ts.net", "Funnel is not enabled on your tailnet."] {
        assert_eq!(tailscale::endereco(linha), None, "{linha}");
    }
}

#[test]
fn a_escolha_do_endereco_e_guardada_e_desconhecida_vira_temporario() {
    let dir = pasta("preferencia");
    let arq = dir.join("compartilhar.json");
    assert_eq!(provedor::ler_de(&arq), Provedor::Temporario, "sem arquivo");
    provedor::gravar_em(&arq, Provedor::Fixo).unwrap();
    assert_eq!(provedor::ler_de(&arq), Provedor::Fixo);
    std::fs::write(&arq, r#"{"provedor":"outro"}"#).unwrap();
    assert_eq!(provedor::ler_de(&arq), Provedor::Temporario);
    std::fs::write(&arq, "torto").unwrap();
    assert_eq!(provedor::ler_de(&arq), Provedor::Temporario);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Um `tailscale` falso: `status --json` e `serve status --json` devolvem os arquivos da pasta; `funnel` anota os argumentos, imprime o endereço e espera.
#[cfg(windows)]
fn tailscale_falso(dir: &Path, status: &str, serve: &str, imprime: &str) -> std::path::PathBuf {
    std::fs::write(dir.join("status.json"), status).unwrap();
    std::fs::write(dir.join("serve.json"), serve).unwrap();
    let caminho = dir.join("tailscale-falso.cmd");
    let corpo = format!("@echo off\r\nif \"%1\"==\"status\" (type \"%~dp0status.json\" & exit /b 0)\r\nif \"%1\"==\"serve\" (type \"%~dp0serve.json\" & exit /b 0)\r\nif \"%1\"==\"funnel\" goto funnel\r\nexit /b 1\r\n:funnel\r\necho %* > \"%~dp0args.txt\"\r\n{imprime}\r\nping -n 60 127.0.0.1 >nul\r\n");
    std::fs::write(&caminho, corpo).unwrap();
    caminho
}

#[cfg(windows)]
fn estado_com_fixo(dir: &Path, ts: std::path::PathBuf) -> Arc<Estado> {
    let e = Arc::new(crate::http::tests::estado(false));
    e.compartilhar.definir_estado(Arc::downgrade(&e));
    e.compartilhar.trocar_comando(Some(dir.join("nao-deve-rodar.exe"))); // o cloudflared não pode ser usado com o endereço fixo
    e.compartilhar.trocar_tailscale(Some(ts));
    e.compartilhar.trocar_preferencia(dir.join("compartilhar.json"));
    e
}

#[cfg(windows)]
const STATUS_OK: &str = r#"{"BackendState":"Running","Self":{"DNSName":"maq.tail1234.ts.net."}}"#;

#[cfg(windows)]
#[test]
fn o_endereco_fixo_e_o_mesmo_ao_desligar_e_ligar_e_a_escolha_fica_guardada() {
    let dir = pasta("fixo");
    let e = estado_com_fixo(&dir, tailscale_falso(&dir, STATUS_OK, "{}", &format!("echo https://{MAQUINA}/")));
    assert_eq!(e.compartilhar.json()["provedor"], "temporario", "o padrão de sempre");
    let r = pela_porta_local(&e, &local("POST", "/api/compartilhar/ligar", Some(json!({"workspaceId": "w2", "provedor": "fixo"}))));
    assert_eq!(r.status, 202, "{}", String::from_utf8_lossy(r.bytes()));
    let j = esperar(&e.compartilhar, "ligado");
    let (primeira, porta) = (j["url"].as_str().unwrap().to_string(), j["porta"].as_u64().unwrap());
    assert_eq!((primeira.as_str(), j["provedor"].as_str()), (format!("https://{MAQUINA}").as_str(), Some("fixo")));
    let args = std::fs::read_to_string(dir.join("args.txt")).unwrap();
    assert!(args.contains("funnel") && args.contains("--https=443") && args.contains(&format!("http://127.0.0.1:{porta}")), "{args}");
    // a porta dos convidados atende pelo endereço fixo
    assert!(e.compartilhar.porta_aberta().is_some_and(|(h, _)| h.url_publica == primeira));
    // desligar e ligar de novo (sem dizer o provedor: usa o guardado): o MESMO endereço
    pela_porta_local(&e, &local("POST", "/api/compartilhar/desligar", Some(json!({}))));
    assert_eq!(e.compartilhar.json()["provedor"], "fixo", "desligado, mostra a escolha guardada");
    pela_porta_local(&e, &local("POST", "/api/compartilhar/ligar", Some(json!({"workspaceId": "w2"}))));
    assert_eq!(esperar(&e.compartilhar, "ligado")["url"], primeira.as_str(), "o convidado continua entrando no mesmo endereço");
    pela_porta_local(&e, &local("POST", "/api/compartilhar/desligar", Some(json!({}))));
    assert_eq!(provedor::ler_de(&dir.join("compartilhar.json")), Provedor::Fixo);
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(windows)]
#[test]
fn usa_a_porta_8443_se_a_443_e_do_serve_e_o_endereco_leva_a_porta() {
    let dir = pasta("fixo-8443");
    let e = estado_com_fixo(&dir, tailscale_falso(&dir, STATUS_OK, r#"{"TCP":{"443":{"HTTPS":true}}}"#, &format!("echo https://{MAQUINA}:8443/")));
    e.compartilhar.ligar_com("w2", Some(Provedor::Fixo)).unwrap();
    let j = esperar(&e.compartilhar, "ligado");
    assert_eq!(j["url"], format!("https://{MAQUINA}:8443").as_str());
    assert!(std::fs::read_to_string(dir.join("args.txt")).unwrap().contains("--https=8443"));
    e.compartilhar.desligar();
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(windows)]
#[test]
fn recusas_do_endereco_fixo_viram_erro_com_o_motivo_e_nao_deixam_porta_aberta() {
    let casos = [
        (r#"{"BackendState":"NeedsLogin","Self":{"DNSName":"maq.tail1234.ts.net."}}"#, "{}", "echo https://maq.tail1234.ts.net/", "NeedsLogin"),
        (STATUS_OK, r#"{"TCP":{"443":{},"8443":{},"10000":{}}}"#, "echo https://maq.tail1234.ts.net/", "10000"),
        (STATUS_OK, "{}", "echo https://outra-maquina.tail1234.ts.net/", "outra-maquina"), // o programa imprimiu o endereço de outra coisa
        (STATUS_OK, "{}", "(echo Funnel is not enabled on your tailnet. 1>&2) & exit /b 1", "not enabled"),
    ];
    for (i, (status, serve, imprime, parte)) in casos.into_iter().enumerate() {
        let dir = pasta(&format!("fixo-erro-{i}"));
        let e = estado_com_fixo(&dir, tailscale_falso(&dir, status, serve, imprime));
        e.compartilhar.ligar_com("w2", Some(Provedor::Fixo)).unwrap();
        let j = esperar(&e.compartilhar, "erro");
        assert!(j["erro"].as_str().unwrap().contains(parte), "{parte}: {j}");
        assert!(e.compartilhar.porta_aberta().is_none() && j.get("porta").is_none(), "{j}");
        let _ = std::fs::remove_dir_all(&dir);
    }
    // sem o Tailscale instalado
    let dir = pasta("fixo-sem");
    let e = estado_com_fixo(&dir, dir.join("nao-existe.cmd"));
    e.compartilhar.ligar_com("w2", Some(Provedor::Fixo)).unwrap();
    assert!(esperar(&e.compartilhar, "erro")["erro"].as_str().unwrap().contains("tailscale"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn provedor_invalido_na_rota_e_400() {
    let e = crate::http::tests::estado(false);
    let r = pela_porta_local(&e, &local("POST", "/api/compartilhar/ligar", Some(json!({"workspaceId": "w2", "provedor": "zerotier"}))));
    assert_eq!((r.status, codigo(&r)), (400, "provedor-invalido".to_string()));
}
