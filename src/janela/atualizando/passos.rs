//! A lógica pura da janela de progresso da atualização (sem janela, para testar): a versão a partir do nome do instalador, os textos e a espera pelo app novo.
use std::time::{Duration, Instant};

/// `Blueprint-0.2.0-instalador.exe` vira `0.2.0`. Sem o padrão, `None` (a janela diz só "Atualizando").
pub fn versao_do_instalador(nome: &str) -> Option<String> {
    let resto = nome.strip_prefix("Blueprint-")?;
    let v = resto.split("-instalador").next().filter(|v| *v != resto)?;
    (!v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+')).then(|| v.to_string())
}

/// O instalador terminou bem? Código 0 é sucesso; qualquer outro (ou falha ao rodar) vira a mensagem da janela.
pub fn resultado(saida: std::io::Result<Option<i32>>) -> Result<(), String> {
    match saida {
        Ok(Some(0)) => Ok(()),
        Ok(Some(c)) => Err(crate::tr!("The installer stopped with code {c}. The previous version stays installed and is opening again.", "O instalador parou com o código {c}. A versão anterior continua instalada e vai abrir de novo.")),
        Ok(None) => Err(crate::tr!("The installer was interrupted. The previous version is opening again.", "O instalador foi interrompido. A versão anterior vai abrir de novo.")),
        Err(e) => Err(crate::tr!("Could not run the installer: {e}", "Não consegui rodar o instalador: {e}")),
    }
}

/// Espera `pronto()` ficar verdadeiro (o app novo no ar), de `passo` em `passo`, até `prazo`. Devolve se chegou a tempo.
pub fn esperar(prazo: Duration, passo: Duration, mut pronto: impl FnMut() -> bool) -> bool {
    let fim = Instant::now() + prazo;
    loop {
        if pronto() {
            return true;
        }
        if Instant::now() >= fim {
            return false;
        }
        std::thread::sleep(passo);
    }
}

fn escapa(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// A página já com os textos (no idioma do Windows).
pub fn pagina(versao: Option<&str>) -> String {
    let (titulo, p2) = match versao {
        Some(v) => (crate::tr!("Updating to v{v}", "Atualizando para a v{v}"), crate::tr!("Installing version {v}", "Instalando a versão {v}")),
        None => (crate::tr!("Updating", "Atualizando"), crate::tr!("Installing the new version", "Instalando a versão nova")),
    };
    let textos = [
        ("{{lang}}", if crate::idioma::ingles() { "en".to_string() } else { "pt-BR".to_string() }),
        ("{{titulo}}", titulo),
        ("{{p0}}", crate::tr!("Installer checked (SHA-256)", "Instalador conferido (SHA-256)")),
        ("{{p1}}", crate::tr!("Database backup saved", "Cópia de segurança do banco guardada")),
        ("{{p2}}", p2),
        ("{{p3}}", crate::tr!("Reopening the app", "Reabrindo o app")),
        ("{{nota}}", crate::tr!("Please wait. The app reopens by itself.", "Aguarde. O app reabre sozinho.")),
        ("{{pronto}}", crate::tr!("Done. Opening The Blueprint.", "Pronto. Abrindo o The Blueprint.")),
        ("{{fechar}}", crate::tr!("Close", "Fechar")),
    ];
    textos.iter().fold(include_str!("pagina.html").to_string(), |h, (chave, valor)| h.replace(chave, &escapa(valor)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_versao_vem_do_nome_do_instalador() {
        assert_eq!(versao_do_instalador("Blueprint-0.2.0-instalador.exe").as_deref(), Some("0.2.0"));
        assert_eq!(versao_do_instalador("Blueprint-1.0.0-beta.1-instalador.exe").as_deref(), Some("1.0.0-beta.1"));
        assert_eq!(versao_do_instalador("setup.exe"), None);
        assert_eq!(versao_do_instalador("Blueprint-<x>-instalador.exe"), None);
    }

    #[test]
    fn so_o_codigo_zero_e_sucesso() {
        assert!(resultado(Ok(Some(0))).is_ok());
        assert!(resultado(Ok(Some(5))).unwrap_err().contains('5'));
        assert!(resultado(Ok(None)).is_err());
        assert!(resultado(Err(std::io::Error::other("negado"))).unwrap_err().contains("negado"));
    }

    #[test]
    fn a_espera_acaba_quando_o_app_responde_ou_no_prazo() {
        let mut n = 0;
        assert!(esperar(Duration::from_secs(2), Duration::from_millis(1), || {
            n += 1;
            n == 3
        }));
        assert!(!esperar(Duration::from_millis(20), Duration::from_millis(5), || false));
    }

    #[test]
    fn a_pagina_traz_a_versao_e_nenhum_marcador_sobra() {
        let h = pagina(Some("0.2.0"));
        assert!(h.contains("0.2.0") && !h.contains("{{"));
        assert!(!pagina(None).contains("{{"));
        assert!(!pagina(Some("<b>")).contains("<b>"), "a versão é escapada");
    }
}
