//! `blueprint atualizar [--verificar | --instalar]`: o updater pela linha de comando.
//! Sem argumento: procura e baixa (e confere) a versão nova, sem instalar. `--verificar`: só consulta. `--instalar`: instala a que já está baixada; digitar o
//! comando é o mesmo clique em Atualizar da interface. Com o app aberto, quem instala é ele (confere o terminal e o compartilhamento, copia o banco e sai); sem o app, instala daqui.
use crate::atualizador::{copiar_banco, lancar_instalador, Atualizador, Fase};
use crate::dados::Dados;
use crate::http::Config;
use crate::mcp::cliente::requisitar_em;
use serde_json::Value;

pub fn executar(modo: Option<&str>) -> i32 {
    let banco = Config::do_app().banco;
    let a = Atualizador::do_ambiente(&banco);
    match modo {
        Some("--verificar") => match a.consultar() {
            Ok(Some(r)) => {
                println!("Nova versão {} disponível (você tem a {}). Rode `blueprint atualizar` para baixar.\n{}", r.versao, a.versao_atual(), r.notas.lines().next().unwrap_or(""));
                0
            }
            Ok(None) => {
                println!("O The Blueprint (beta) {} está em dia.", a.versao_atual());
                0
            }
            Err(e) => erro(&e),
        },
        Some("--instalar") => instalar(&a, &banco),
        _ => {
            a.recuperar();
            a.verificar(true);
            match a.fase() {
                Fase::Pronta => {
                    println!("A versão {} está baixada e conferida (SHA-256). Para instalar: blueprint atualizar --instalar", a.versao_pronta().unwrap_or_default());
                    0
                }
                Fase::EmDia => {
                    println!("O The Blueprint (beta) {} está em dia.", a.versao_atual());
                    0
                }
                Fase::Erro(e) => erro(&e),
                _ => erro("a verificação não terminou"),
            }
        }
    }
}

fn erro(texto: &str) -> i32 {
    eprintln!("erro: {texto}");
    1
}

fn instalar(a: &Atualizador, banco: &std::path::Path) -> i32 {
    a.recuperar();
    if a.versao_pronta().is_none() {
        return erro("não há atualização baixada; rode `blueprint atualizar` primeiro");
    }
    // o app aberto decide (ele sabe do terminal, do compartilhamento e do banco aberto); se não responde, instala daqui
    if let Ok((status, corpo)) = requisitar_em(&crate::mcp::stdio::base(), "POST", "/api/atualizacao/instalar", Some("{}")) {
        if (200..300).contains(&status) {
            println!("Atualizando: o app fecha e abre de novo sozinho.");
            return 0;
        }
        let motivo = serde_json::from_str::<Value>(&corpo).ok().and_then(|v| v["erro"].as_str().map(String::from)).unwrap_or(corpo);
        return erro(&motivo);
    }
    let backup = |versao: &str| {
        let dados = Dados::abrir(banco).map_err(|e| format!("não consegui abrir o banco para copiá-lo: {e}"))?;
        copiar_banco(dados.conexao(), &banco.parent().map_or_else(|| "backups".into(), |p| p.join("backups")), versao).map(|_| ())
    };
    match a.instalar(None, &backup, &lancar_instalador) {
        Ok(_) => {
            println!("Instalador iniciado: a versão nova abre sozinha quando terminar.");
            0
        }
        Err(r) => erro(&r.mensagem),
    }
}
