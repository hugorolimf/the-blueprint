use super::{atualizador, pasta, sha256, Falso};
use crate::atualizador::{Atualizador, Fase, Origem};
use std::sync::atomic::Ordering;

#[test]
fn versao_nova_e_baixada_conferida_e_fica_pronta_sem_instalar() {
    let (falso, p) = (Falso::novo(), pasta("pronta"));
    falso.publicar("0.2.0", b"conteudo do instalador", None);
    let a = atualizador(&falso, p.clone());
    a.verificar(false);
    assert_eq!(a.fase(), Fase::Pronta);
    assert_eq!(a.versao_pronta().as_deref(), Some("0.2.0"));
    assert_eq!(std::fs::read(p.join("Blueprint-0.2.0-instalador.exe")).unwrap(), b"conteudo do instalador");
    assert!(!p.join("Blueprint-0.2.0-instalador.exe.parcial").exists());
    let json = a.json();
    assert_eq!((json["fase"].as_str(), json["versao"].as_str(), json["notas"].as_str()), (Some("pronta"), Some("0.2.0"), Some("Notas da versão")));
    assert!(!a.passou_o_intervalo(crate::tempo::agora_ms()), "a verificação fica registrada");
}

#[test]
fn instalador_que_nao_confere_com_o_sha256sums_e_descartado() {
    let (falso, p) = (Falso::novo(), pasta("hash-errado"));
    falso.publicar("0.2.0", b"conteudo", Some(&sha256(b"outro conteudo")));
    let a = atualizador(&falso, p.clone());
    a.verificar(false);
    assert!(matches!(a.fase(), Fase::Erro(e) if e.contains("não confere")));
    assert!(!p.join("Blueprint-0.2.0-instalador.exe").exists() && !p.join("Blueprint-0.2.0-instalador.exe.parcial").exists());
    assert!(a.versao_pronta().is_none());
}

#[test]
fn sem_versao_nova_fica_em_dia() {
    let (falso, p) = (Falso::novo(), pasta("em-dia"));
    falso.publicar("0.1.0", b"x", None);
    let a = atualizador(&falso, p);
    a.verificar(false);
    assert_eq!(a.fase(), Fase::EmDia);
}

#[test]
fn falha_de_rede_vira_erro_e_nao_apaga_o_que_ja_estava_pronto() {
    let (falso, p) = (Falso::novo(), pasta("rede"));
    falso.publicar("0.2.0", b"instalador", None);
    let a = atualizador(&falso, p.clone());
    a.verificar(false);
    assert_eq!(a.fase(), Fase::Pronta);
    drop(falso); // o "GitHub" some
    let morto = Atualizador::novo(p, Origem { api: "http://127.0.0.1:1".into(), teste: true }, "0.1.0", false);
    morto.verificar(false);
    assert!(matches!(morto.fase(), Fase::Erro(_)), "sem nada pronto, a falha aparece");
}

#[test]
fn verificar_com_um_instalador_pronto_nao_baixa_de_novo_e_a_falha_nao_o_apaga() {
    let (falso, p) = (Falso::novo(), pasta("repetir"));
    falso.publicar("0.2.0", b"instalador", None);
    let a = atualizador(&falso, p.clone());
    a.verificar(false);
    let antes = falso.pedidos.load(Ordering::SeqCst);
    a.verificar(true);
    assert_eq!(falso.pedidos.load(Ordering::SeqCst), antes + 1, "só a lista de releases: nem o SHA256SUMS nem o instalador de novo");
    assert_eq!(a.fase(), Fase::Pronta);
    drop(falso);
    a.verificar(true);
    assert_eq!(a.fase(), Fase::Pronta, "o servidor sumiu, mas o instalador conferido continua pronto");
}

#[test]
fn ao_reabrir_o_app_o_instalador_baixado_volta_pronto_sem_rede() {
    let (falso, p) = (Falso::novo(), pasta("reabrir"));
    falso.publicar("0.2.0", b"instalador", None);
    atualizador(&falso, p.clone()).verificar(false);
    std::fs::write(p.join("Blueprint-0.0.1-instalador.exe"), "velho").unwrap();
    std::fs::write(p.join("Blueprint-0.3.0-instalador.exe.parcial"), "pela metade").unwrap();
    let outro = Atualizador::novo(p.clone(), Origem { api: "http://127.0.0.1:1".into(), teste: true }, "0.1.0", false);
    outro.reconstruir();
    assert_eq!(outro.versao_pronta().as_deref(), Some("0.2.0"));
    assert!(!p.join("Blueprint-0.0.1-instalador.exe").exists() && !p.join("Blueprint-0.3.0-instalador.exe.parcial").exists(), "lixo antigo sai");
}

#[test]
fn instalador_adulterado_ou_ja_instalado_nao_volta_pronto() {
    let (falso, p) = (Falso::novo(), pasta("adulterado"));
    falso.publicar("0.2.0", b"instalador", None);
    atualizador(&falso, p.clone()).verificar(false);
    let arq = p.join("Blueprint-0.2.0-instalador.exe");
    std::fs::write(&arq, "trocado").unwrap();
    let a = Atualizador::novo(p.clone(), falso.origem(), "0.1.0", false);
    a.reconstruir();
    assert!(a.versao_pronta().is_none(), "o hash não bate");
    assert!(!arq.exists(), "e o arquivo suspeito é apagado");
    // a versão baixada já é a instalada: não é "pronta"
    atualizador(&falso, pasta("adulterado")).verificar(false);
    let b = Atualizador::novo(p, falso.origem(), "0.2.0", false);
    b.reconstruir();
    assert!(b.versao_pronta().is_none());
}

#[test]
fn desligada_nao_toca_na_rede_sozinha_mas_obedece_o_pedido_do_usuario() {
    let (falso, p) = (Falso::novo(), pasta("desligada"));
    falso.publicar("0.2.0", b"instalador", None);
    let a = atualizador(&falso, p);
    a.definir_automatica(false);
    assert!(!a.automatica());
    a.verificar(false);
    assert_eq!(falso.pedidos.load(Ordering::SeqCst), 0, "zero requisições");
    assert_eq!(a.fase(), Fase::Desligada);
    a.verificar(true);
    assert_eq!(a.fase(), Fase::Pronta, "o usuário pediu: vale");
    a.definir_automatica(true);
    assert!(a.automatica());
}

#[test]
fn endereco_fora_da_lista_nao_e_baixado() {
    let github = Origem { api: "https://api.github.com/repos/x/y".into(), teste: false };
    assert!(github.permite("https://github.com/x/y/releases/download/v1/i.exe"));
    assert!(github.permite("https://objects.githubusercontent.com/abc"));
    for ruim in ["http://github.com/x", "https://evil.example/i.exe", "https://github.com.evil.example/i.exe", "https://evilgithub.com/i.exe", "file:///C:/i.exe", "https://127.0.0.1/i.exe"] {
        assert!(!github.permite(ruim), "{ruim}");
    }
    let teste = Origem { api: "http://127.0.0.1:9".into(), teste: true };
    assert!(teste.permite("http://127.0.0.1:9/baixar/x") && !teste.permite("https://github.com/x") && !teste.permite("http://127.0.0.1.evil.example/x"));
}

#[test]
fn release_que_aponta_para_fora_e_recusada_sem_baixar_nada() {
    let (falso, p) = (Falso::novo(), pasta("fora"));
    falso.servir("/releases", serde_json::json!([{"tag_name": "v0.2.0", "draft": false, "prerelease": false, "body": "", "assets": [
        {"name": "Blueprint-0.2.0-instalador.exe", "size": 5, "browser_download_url": "https://evil.example/i.exe"},
        {"name": "SHA256SUMS", "size": 5, "browser_download_url": format!("{}/baixar/SHA256SUMS", falso.base)}]}]).to_string());
    let a = atualizador(&falso, p.clone());
    a.verificar(false);
    assert!(matches!(a.fase(), Fase::Erro(e) if e.contains("fora da lista")));
    assert!(!p.exists() || std::fs::read_dir(&p).unwrap().next().is_none());
}

#[test]
fn verifica_no_maximo_uma_vez_a_cada_24_horas() {
    let (falso, p) = (Falso::novo(), pasta("intervalo"));
    let a = atualizador(&falso, p.clone());
    let agora = crate::tempo::agora_ms();
    let horas = |h: i64| h * 60 * 60 * 1000;
    assert!(a.passou_o_intervalo(agora), "nunca verificou");
    std::fs::create_dir_all(&p).unwrap();
    let grava = |emms: i64| std::fs::write(a.arquivo("verificacao.json"), serde_json::json!({"emMs": emms}).to_string()).unwrap();
    grava(agora - horas(23));
    assert!(!a.passou_o_intervalo(agora), "23 h: ainda não");
    grava(agora - horas(25));
    assert!(a.passou_o_intervalo(agora), "25 h: verifica");
    grava(agora + horas(5));
    assert!(a.passou_o_intervalo(agora), "relógio voltou para trás: verifica em vez de ficar parado");
    std::fs::write(a.arquivo("verificacao.json"), "lixo").unwrap();
    assert!(a.passou_o_intervalo(agora), "arquivo ilegível conta como nunca");
}
