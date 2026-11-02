//! O que o menu mostra para cada alvo (`contexto-itens.js` e `acao-situacao.js` da página). Só regras: quem desenha é `desenho.rs`.
//! O que depende de tela que a interface nativa ainda não tem (editar, a tela de proposta, conversar, iniciar com o Claude) vira «Abrir na versão web».

use super::{Comando, Grupo, Icone, Item};
use crate::agentes::Agente;
use crate::dominio::Situacao;
use crate::folha::corta;
use crate::i18n::tr;
use crate::telas::Acao;

/// O que está sob o cursor. `tipo` de `Item` é `planos` ou `checkpoints`.
#[derive(Clone, Debug)]
pub enum Alvo {
    Entrada,
    Workspace { id: String, nome: String },
    Lista { ws: String, nome: String },
    Cartao { ws: String, id: String, nome: String },
    Folha { ws: String, bp: String, nome: String, chaves: Vec<String>, aberto: bool },
    Acao { ws: String, bp: String, id: String, rotulo: String, status: Situacao, itens: (usize, usize) },
    Item { ws: String, bp: String, acao: String, tipo: &'static str, id: String, texto: String, feito: bool },
    Nota { ws: String, bp: String, acao: Option<String> },
    Link(String),
    Figura(String),
}

/// O título (o alvo mais específico) e os itens, já na ordem dos grupos.
pub fn montar(alvos: &[Alvo], agentes: &[Agente], tela_cheia: bool, criando: bool) -> (String, Vec<Item>) {
    let mut itens = Vec::new();
    let mut add = |grupo, icone, rotulo: &str, comando| itens.push(Item { grupo, icone, rotulo: rotulo.to_string(), comando: Some(comando), perigo: grupo == Grupo::Perigo, motivo: None });
    let mut desligados = Vec::new();
    let mut titulo = None::<(u8, String)>;
    let mut propor = |nivel: u8, texto: String| {
        if titulo.as_ref().is_none_or(|(n, _)| nivel > *n) {
            titulo = Some((nivel, texto));
        }
    };
    let cartao_ou_workspace = alvos.iter().any(|a| matches!(a, Alvo::Workspace { .. } | Alvo::Cartao { .. }));
    for alvo in alvos {
        match alvo {
            Alvo::Entrada => add(Grupo::Editar, Icone::Mais, tr("Criar workspace"), Comando::CriarWorkspace),
            Alvo::Workspace { id, nome } => {
                propor(3, format!("Workspace · {nome}"));
                add(Grupo::Abrir, Icone::Abrir, tr("Abrir workspace"), Comando::AbrirWorkspace(id.clone()));
                add(Grupo::Perigo, Icone::Lixo, tr("Excluir workspace…"), Comando::ExcluirWorkspace(id.clone()));
            }
            Alvo::Lista { ws, nome } => {
                propor(1, format!("Workspace · {nome}"));
                if !cartao_ou_workspace {
                    add(Grupo::Editar, Icone::Mais, tr("Novo blueprint"), Comando::NovoBlueprint(ws.clone()));
                    add(Grupo::Abrir, Icone::Externo, tr("Abrir na versão web"), Comando::Web(format!("#/c/{ws}")));
                }
                add(Grupo::Geral, Icone::Voltar, tr("Voltar aos espaços"), Comando::Entrada);
            }
            Alvo::Cartao { ws, id, nome } => {
                propor(2, format!("{} · {}", tr("Blueprint"), nome_ou_vazio(nome)));
                add(Grupo::Abrir, Icone::Abrir, tr("Abrir blueprint"), Comando::AbrirBlueprint(ws.clone(), id.clone()));
                add(Grupo::Abrir, Icone::Externo, tr("Abrir na versão web"), Comando::Web(format!("#/p/{id}")));
                add(Grupo::Perigo, Icone::Lixo, tr("Excluir blueprint…"), Comando::ExcluirItem(ws.clone(), id.clone(), None));
            }
            Alvo::Folha { ws, bp, nome, chaves, aberto } => {
                propor(2, format!("{} · {}", tr("Blueprint"), nome_ou_vazio(nome)));
                add(Grupo::Abrir, Icone::Externo, tr("Abrir na versão web"), Comando::Web(format!("#/p/{bp}")));
                if !chaves.is_empty() {
                    let (icone, rotulo) = if *aberto { (Icone::Recolher, tr("Recolher ações")) } else { (Icone::Expandir, tr("Expandir ações")) };
                    add(Grupo::Geral, icone, rotulo, Comando::Secoes(chaves.clone(), !aberto));
                }
                add(Grupo::Geral, Icone::Voltar, tr("Voltar aos blueprints"), Comando::AbrirWorkspace(ws.clone()));
            }
            Alvo::Acao { ws, bp, id, rotulo, status, itens: (feitos, total) } => {
                propor(4, format!("{} {rotulo}", tr("Ação")));
                add(Grupo::Abrir, Icone::Alvo, tr("Ir para esta ação"), Comando::IrPara(id.clone()));
                add(Grupo::Perigo, Icone::Lixo, tr("Excluir ação…"), Comando::ExcluirItem(ws.clone(), bp.clone(), Some(id.clone())));
                let dados = |campo: Campo| match campo {
                    Campo::Situacao(s) => Comando::Dados(Acao::Situacao(ws.clone(), bp.clone(), id.clone(), s)),
                    Campo::Prorrogar(d) => Comando::Dados(Acao::Prorrogar(ws.clone(), bp.clone(), id.clone(), d)),
                };
                // concluída por planos e checkpoints: só desmarcá-los muda isso
                if !(*status == Situacao::Concluido && *total > 0) {
                    if *total == 0 || *feitos == 0 {
                        if *status != Situacao::Planejado {
                            add(Grupo::Estado, Icone::Circulo, tr("Marcar como planejada"), dados(Campo::Situacao("planejado")));
                        }
                        if *status != Situacao::Andamento {
                            add(Grupo::Estado, Icone::Relogio, tr("Marcar como em andamento"), dados(Campo::Situacao("andamento")));
                        }
                    }
                    if *status != Situacao::Concluido {
                        if *total > 0 {
                            desligados.push(Item { grupo: Grupo::Estado, icone: Icone::Check, rotulo: tr("Marcar como concluída").to_string(), comando: None, perigo: false, motivo: Some(tr("Conclui sozinha quando todos os planos e checkpoints estiverem marcados")) });
                        } else {
                            add(Grupo::Estado, Icone::Check, tr("Marcar como concluída"), dados(Campo::Situacao("concluido")));
                        }
                        add(Grupo::Prazo, Icone::Calendario, tr("Prorrogar o fim em 1 dia"), dados(Campo::Prorrogar(1)));
                        add(Grupo::Prazo, Icone::Calendario, tr("Prorrogar o fim em 1 semana"), dados(Campo::Prorrogar(7)));
                    }
                }
            }
            Alvo::Item { ws, bp, acao, tipo, id, texto, feito } => {
                let nome = if texto.is_empty() { tr("sem nome") } else { texto.as_str() };
                propor(5, format!("{} · {nome}", if *tipo == "planos" { tr("Plano") } else { tr("Checkpoint") }));
                let (icone, rotulo) = if *feito { (Icone::Circulo, tr("Marcar como pendente")) } else { (Icone::Check, tr("Marcar como feito")) };
                add(Grupo::Estado, icone, rotulo, Comando::Dados(Acao::Marcar(ws.clone(), bp.clone(), acao.clone(), tipo, id.clone())));
                if *tipo == "planos" {
                    if let Some(a) = agentes.iter().find(|a| a.plano_id.as_deref() == Some(id.as_str())) {
                        add(Grupo::Abrir, Icone::Alvo, tr("Acompanhar este agente"), Comando::Agente(a.agente.clone()));
                    }
                }
            }
            Alvo::Nota { ws, bp, acao } => {
                let rotulo = if acao.is_some() { tr("Abrir instruções") } else { tr("Abrir observações") };
                add(Grupo::Abrir, Icone::Doc, rotulo, Comando::Dados(Acao::AbrirNota(ws.clone(), bp.clone(), acao.clone())));
            }
            Alvo::Link(url) => add(Grupo::Abrir, Icone::Externo, tr("Abrir link em nova aba"), Comando::Link(url.clone())),
            Alvo::Figura(nome) => add(Grupo::Abrir, Icone::Alvo, tr("Acompanhar este agente"), Comando::Agente(nome.clone())),
        }
    }
    if criando {
        add(Grupo::Geral, Icone::Voltar, tr("Voltar aos espaços"), Comando::FecharNovo);
    }
    if !cfg!(target_arch = "wasm32") {
        add(Grupo::Geral, Icone::TelaCheia, if tela_cheia { tr("Sair da tela cheia") } else { tr("Entrar em tela cheia") }, Comando::TelaCheia);
    }
    itens.append(&mut desligados);
    itens.sort_by_key(|i| i.grupo); // estável: dentro do grupo vale a ordem em que foram somados
    let titulo = titulo.map_or_else(|| "The Blueprint".to_string(), |(_, t)| t);
    (corta(&titulo, 48), itens)
}

/// O item que troca o blueprint aberto entre o Planejar e a folha antiga: é o caminho de volta, porque a folha antiga não tem botão para isso.
pub fn trocar_folha(itens: &mut Vec<Item>, bp: &str, antiga: bool) {
    let rotulo = if antiga { tr("Planejar") } else { tr("Folha antiga") };
    itens.push(Item { grupo: Grupo::Abrir, icone: Icone::Doc, rotulo: rotulo.to_string(), comando: Some(Comando::FolhaAntiga(bp.to_string(), !antiga)), perigo: false, motivo: None });
    itens.sort_by_key(|i| i.grupo);
}

enum Campo {
    Situacao(&'static str),
    Prorrogar(i64),
}

fn nome_ou_vazio(nome: &str) -> &str {
    if nome.is_empty() {
        tr("sem nome")
    } else {
        nome
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn acao(status: Situacao, itens: (usize, usize)) -> Alvo {
        Alvo::Acao { ws: "sm".into(), bp: "b".into(), id: "a".into(), rotulo: "1.1 · Teste".into(), status, itens }
    }

    fn rotulos(alvos: &[Alvo]) -> Vec<String> {
        montar(alvos, &[], false, false).1.into_iter().map(|i| i.rotulo).collect()
    }

    #[test]
    fn concluida_por_itens_nao_muda_de_situacao() {
        let l = rotulos(&[acao(Situacao::Concluido, (2, 2))]);
        assert!(!l.iter().any(|r| r.starts_with("Marcar") || r.starts_with("Prorrogar")), "{l:?}");
    }

    #[test]
    fn sem_itens_a_situacao_e_escolha_da_pessoa() {
        let l = rotulos(&[acao(Situacao::Planejado, (0, 0))]);
        assert!(l.contains(&"Marcar como em andamento".to_string()) && l.contains(&"Marcar como concluída".to_string()) && !l.contains(&"Marcar como planejada".to_string()), "{l:?}");
        let (_, itens) = montar(&[acao(Situacao::Andamento, (1, 3))], &[], false, false);
        let concluir = itens.iter().find(|i| i.rotulo == "Marcar como concluída").expect("aparece desligado");
        assert!(concluir.comando.is_none() && concluir.motivo.is_some());
        assert!(!itens.iter().any(|i| i.rotulo == "Marcar como planejada"), "com item feito, a situação vem deles");
    }

    #[test]
    fn os_grupos_ficam_em_ordem_e_o_titulo_e_o_mais_especifico() {
        let folha = Alvo::Folha { ws: "sm".into(), bp: "b".into(), nome: "Obra".into(), chaves: vec!["b|".into()], aberto: true };
        let item = Alvo::Item { ws: "sm".into(), bp: "b".into(), acao: "a".into(), tipo: "planos", id: "p".into(), texto: "Pintar".into(), feito: false };
        let (titulo, itens) = montar(&[folha, acao(Situacao::Andamento, (0, 1)), item], &[], false, false);
        assert_eq!(titulo, "Plano · Pintar");
        assert!(itens.windows(2).all(|p| p[0].grupo <= p[1].grupo));
        assert!(itens.iter().any(|i| i.rotulo == "Marcar como feito"));
    }

    #[test]
    fn todo_workspace_se_exclui_inclusive_o_principal() {
        let l = rotulos(&[Alvo::Entrada, Alvo::Workspace { id: crate::documento::RAIZ.into(), nome: "Raiz".into() }]);
        assert!(l.iter().any(|r| r == "Excluir workspace…"), "{l:?}");
    }

    #[test]
    fn o_plano_com_agente_oferece_acompanhar() {
        let item = Alvo::Item { ws: "sm".into(), bp: "b".into(), acao: "a".into(), tipo: "planos", id: "p".into(), texto: String::new(), feito: true };
        let agente = Agente { agente: "ana".into(), plano_id: Some("p".into()), ..Default::default() };
        let (titulo, itens) = montar(&[item], &[agente], false, false);
        assert_eq!(titulo, "Plano · sem nome");
        assert!(itens.iter().any(|i| i.rotulo == "Acompanhar este agente") && itens.iter().any(|i| i.rotulo == "Marcar como pendente"));
    }
}
