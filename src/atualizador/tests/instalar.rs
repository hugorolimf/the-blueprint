use super::{atualizador, pasta, Falso};
use crate::atualizador::instalar::argumentos;
use crate::atualizador::Fase;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

fn pronta(nome: &str) -> (Falso, crate::atualizador::Atualizador, PathBuf) {
    let (falso, p) = (Falso::novo(), pasta(nome));
    falso.publicar("0.2.0", b"instalador de mentira", None);
    let a = atualizador(&falso, p.clone());
    a.verificar(false);
    assert_eq!(a.fase(), Fase::Pronta);
    (falso, a, p)
}

fn ok(_: &str) -> Result<(), String> {
    Ok(())
}

#[test]
fn sem_o_clique_nada_e_executado_so_o_instalar_executa() {
    let (_falso, a, p) = pronta("clique");
    let (chamadas, usado) = (Cell::new(0), RefCell::new(None));
    let lancar = |arq: &Path, args: &[String]| {
        chamadas.set(chamadas.get() + 1);
        *usado.borrow_mut() = Some((arq.to_path_buf(), args.to_vec()));
        Ok(true)
    };
    assert_eq!(chamadas.get(), 0, "baixar e conferir não lançam o instalador");
    assert!(a.instalar(None, &ok, &lancar).unwrap(), "o instalador roda e o app deve sair");
    assert_eq!(chamadas.get(), 1);
    let (arq, args) = usado.borrow().clone().unwrap();
    assert_eq!(arq, p.join("Blueprint-0.2.0-instalador.exe"));
    assert!(args.contains(&"/SUPPRESSMSGBOXES".to_string()) && args.contains(&"/CLOSEAPPLICATIONS".to_string()) && args.contains(&"/ATUALIZACAO=1".to_string()));
    assert!(args.iter().any(|x| x == "/VERYSILENT" || x == "/SILENT"));
    assert_eq!(a.fase(), Fase::Instalando);
}

#[test]
fn recusa_com_trabalho_do_agente_e_continua_pronta() {
    let (_falso, a, _p) = pronta("recusa");
    let r = a.instalar(Some("Há 2 marcações do agente que a tela ainda não aplicou.".into()), &ok, &|_, _| panic!("não pode instalar")).unwrap_err();
    assert_eq!((r.status, r.codigo), (409, "agora-nao"));
    assert_eq!(a.fase(), Fase::Pronta);
    assert_eq!(a.json()["motivo"], "Há 2 marcações do agente que a tela ainda não aplicou.");
}

#[test]
fn sem_atualizacao_pronta_nao_ha_o_que_instalar() {
    let falso = Falso::novo();
    let a = atualizador(&falso, pasta("nada"));
    let r = a.instalar(None, &ok, &|_, _| panic!("não pode instalar")).unwrap_err();
    assert_eq!((r.status, r.codigo), (409, "sem-atualizacao"));
}

#[test]
fn arquivo_trocado_depois_do_download_nao_e_executado() {
    let (_falso, a, p) = pronta("trocado");
    std::fs::write(p.join("Blueprint-0.2.0-instalador.exe"), "outro programa").unwrap();
    let r = a.instalar(None, &ok, &|_, _| panic!("não pode instalar")).unwrap_err();
    assert_eq!(r.codigo, "arquivo-alterado");
    assert!(!p.join("Blueprint-0.2.0-instalador.exe").exists(), "o arquivo suspeito é apagado");
    assert!(matches!(a.fase(), Fase::Erro(_)));
}

#[test]
fn falha_no_backup_impede_a_instalacao() {
    let (_falso, a, _p) = pronta("falha-backup");
    let r = a.instalar(None, &|_| Err("disco cheio".into()), &|_, _| panic!("não pode instalar")).unwrap_err();
    assert_eq!((r.status, r.codigo), (500, "backup"));
    assert_eq!(a.fase(), Fase::Pronta, "o usuário pode tentar de novo");
}

#[test]
fn falha_ao_lancar_volta_a_ficar_pronta() {
    let (_falso, a, _p) = pronta("lancar");
    let r = a.instalar(None, &ok, &|_, _| Err("bloqueado pelo antivírus".into())).unwrap_err();
    assert_eq!(r.codigo, "instalador");
    assert_eq!((a.fase(), a.json()["motivo"].as_str()), (Fase::Pronta, Some("bloqueado pelo antivírus")));
}

#[cfg(windows)] // o caminho de "Program Files" e a variável ProgramFiles só fazem sentido no Windows
#[test]
fn instalacao_para_todos_mostra_o_uac_em_vez_de_rodar_calada() {
    let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".into());
    assert!(argumentos(&Path::new(&pf).join("Blueprint").join("blueprint.exe")).contains(&"/SILENT".to_string()));
    assert!(argumentos(Path::new("C:\\Users\\alguem\\AppData\\Local\\Programs\\Blueprint\\blueprint.exe")).contains(&"/VERYSILENT".to_string()));
}
