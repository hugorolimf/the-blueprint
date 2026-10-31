//! O app com janela fora do Windows: o servidor e a interface nativa (`ui/`, egui) no mesmo processo. A janela é só a tela: tudo (dados, MCP,
//! protótipos) é do servidor, que cai junto com ela. Ainda sem bandeja, sem atualizador e sem a janela de convite de equipe (essas são do Windows).

use crate::erro::Resultado;
use crate::http::{self, Config};
use crate::instancia::{self, Instancia};
use std::path::PathBuf;

/// `dev` (`blueprint dev`): o mesmo app, com a página web antiga (a folha, enquanto não migra) lida da pasta do projeto, sem compilar.
/// `link`: um `blueprint://…` que o navegador entregou; só a janela do Windows sabe abri-lo, aqui ele abre o app sem link.
pub fn abrir(dev: bool, link: Option<String>) -> i32 {
    let _ = link;
    if instancia::garantir() == Instancia::JaAberta {
        if dev {
            // mesmo banco e mesma porta: dois processos teriam filas do MCP separadas e a janela ficaria com dados velhos
            eprintln!("O The Blueprint já está aberto. O modo dev usa o mesmo banco e a mesma porta: feche o app e rode de novo.");
            return 1;
        }
        instancia::pedir_que_mostre(None);
        return 0;
    }
    match executar(dev) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("O The Blueprint (beta) não conseguiu abrir: {e}");
            1
        }
    }
}

fn raiz_do_projeto() -> Option<PathBuf> {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    raiz.join("blueprint.html").is_file().then_some(raiz)
}

fn executar(dev: bool) -> Resultado<()> {
    let mut config = Config::do_app();
    if dev {
        match raiz_do_projeto() {
            Some(raiz) => {
                println!("dev: a página web antiga é lida de {}; a interface nativa é a do ui/ (recompile para ver mudanças)", raiz.display());
                config.assets_dir.get_or_insert(raiz);
            }
            None => eprintln!("aviso: o modo dev precisa do binário compilado na pasta do projeto; seguindo sem a página web da pasta"),
        }
    }
    let servidor = http::iniciar_ou_porta_livre(config)?;
    instancia::registrar_porta(servidor.porta);
    let banco = servidor.estado.config.banco.clone();
    std::thread::spawn(move || {
        // no máximo uma cópia por dia; falhar aqui nunca impede o app de abrir
        let pasta = crate::dados::backups::pasta_backups(&banco).join("automaticos");
        if let Err(e) = crate::dados::backups::backup_diario(&banco, &pasta, crate::tempo::agora_ms()) {
            eprintln!("aviso: backup automático do banco: {e}");
        }
    });
    // a segunda execução pede (rota /api/app/mostrar) que a janela venha à frente
    let estado = servidor.estado.clone();
    let resultado = blueprint_ui::executar(servidor.url.clone(), move |mostrar| {
        *estado.ao_mostrar.lock().unwrap_or_else(|e| e.into_inner()) = Some(Box::new(move |_link| mostrar()));
    });
    servidor.parar();
    resultado.map_err(|e| crate::erro::Erro::Io(std::io::Error::other(e)))
}
