//! Testar sem mouse: `BLUEPRINT_UI_ROTEIRO=nome:Ana,criar:Alfa,exemplo,excluir:Alfa,novo` roda as ações em ordem, cada uma quando a anterior terminou
//! de gravar, e `BLUEPRINT_UI_ABRIR=novo` abre o diálogo. Junto com `BLUEPRINT_UI_CAPTURA` dá para ver o resultado sem tocar na tela.

use crate::documento::Documento;
use crate::telas::{Acao, Novo, Vista};
use std::collections::VecDeque;

/// Os passos que faltam e, durante um `pausa:ms`, até quando esperar (para ver uma animação no meio).
pub struct Roteiro(VecDeque<String>, Option<std::time::Instant>);

pub fn do_ambiente() -> Option<Roteiro> {
    let texto = std::env::var("BLUEPRINT_UI_ROTEIRO").ok().filter(|v| !v.is_empty())?;
    Some(Roteiro(texto.split(',').map(str::to_string).collect(), None))
}

/// `BLUEPRINT_UI_PPP=1`: força a escala a cada quadro (a do monitor, 1,25 neste notebook, muda o tamanho em pontos); a comparação com a web é a 1.
pub fn escala(ctx: &egui::Context) {
    if let Some(ppp) = std::env::var("BLUEPRINT_UI_PPP").ok().and_then(|v| v.parse::<f32>().ok()) {
        ctx.set_zoom_factor(ppp / ctx.native_pixels_per_point().unwrap_or(1.0));
    }
}

/// Abre o que `BLUEPRINT_UI_ABRIR` pede (só o diálogo `novo` por enquanto).
pub fn abrir(vista: &mut Vista) {
    if std::env::var("BLUEPRINT_UI_ABRIR").as_deref() == Ok("novo") {
        vista.novo = Some(Novo::vazio());
    }
}

impl Roteiro {
    /// O próximo passo, se houver. `excluir:Nome` procura o workspace pelo nome.
    pub fn proximo(&mut self, ctx: &egui::Context, doc: &Documento, vista: &mut Vista) -> Option<Acao> {
        if let Some(ate) = self.1 {
            if std::time::Instant::now() < ate {
                return None;
            }
            self.1 = None;
        }
        let passo = self.0.pop_front()?;
        let (verbo, arg) = passo.split_once(':').unwrap_or((passo.as_str(), ""));
        match verbo {
            "nome" => Some(Acao::DefinirNome(arg.into())),
            "criar" => Some(Acao::CriarWorkspace { nome: arg.into(), descricao: String::new(), icone: "chip".into() }),
            "exemplo" => Some(Acao::AbrirExemplo),
            "excluir" => doc.workspaces().into_iter().find(|w| w.nome == arg).map(|w| Acao::Excluir(w.id)),
            "mover" => doc.workspaces().into_iter().find(|w| w.nome == arg).map(|w| Acao::Mover(w.id, -1)),
            "blueprint" => {
                let (ws, bp) = arg.split_once('/').unwrap_or(("sm", arg));
                vista.tela = crate::telas::Tela::Blueprint(ws.into(), bp.into());
                None
            }
            "marcar" => {
                // marcar:workspace/blueprint/ação/planos|checkpoints/item
                let p: Vec<&str> = arg.split('/').collect();
                match p.as_slice() {
                    [ws, bp, acao, "planos", item] => Some(Acao::Marcar((*ws).into(), (*bp).into(), (*acao).into(), "planos", (*item).into())),
                    [ws, bp, acao, "checkpoints", item] => Some(Acao::Marcar((*ws).into(), (*bp).into(), (*acao).into(), "checkpoints", (*item).into())),
                    _ => None,
                }
            }
            "secao" => {
                // secao:blueprint|projeto abre a seção (testes)
                vista.abertas.insert(arg.into(), true);
                None
            }
            "pendentes" => {
                vista.pendentes.insert(arg.into());
                None
            }
            "buscar" => {
                vista.consulta = arg.into();
                None
            }
            "clicar" => {
                // clicar:x,y não serve (a vírgula separa os passos): clicar:x/y
                if let Some((x, y)) = arg.split_once('/') {
                    vista.clique_pedido = x.parse().ok().zip(y.parse().ok()).map(|(x, y)| (egui::pos2(x, y), egui::PointerButton::Primary));
                }
                None
            }
            "roda" => {
                // roda:x/y/d gira a roda do mouse em x/y, d pontos: para cima, positivo (a mesma conta do mouse de verdade)
                let v: Vec<f32> = arg.split('/').filter_map(|n| n.parse().ok()).collect();
                if let [x, y, d] = v[..] {
                    vista.roda_pedida = Some((egui::pos2(x, y), d));
                }
                None
            }
            "arrastar" | "arrastar-direito" => {
                // arrastar:x0/y0/x1/y1 aperta em x0/y0, anda até x1/y1 e solta
                let n: Vec<f32> = arg.split('/').filter_map(|v| v.parse().ok()).collect();
                if let [x0, y0, x1, y1] = n[..] {
                    let botao = if verbo == "arrastar" { egui::PointerButton::Primary } else { egui::PointerButton::Secondary };
                    vista.arrasto_pedido = Some((egui::pos2(x0, y0), egui::pos2(x1, y1), botao));
                }
                None
            }
            "direito" => {
                // direito:x/y clica com o botão direito (abre o menu)
                if let Some((x, y)) = arg.split_once('/') {
                    vista.clique_pedido = x.parse().ok().zip(y.parse().ok()).map(|(x, y)| (egui::pos2(x, y), egui::PointerButton::Secondary));
                }
                None
            }
            "mouse-voltar" | "mouse-avancar" => {
                // o botão lateral do mouse, como um clique de verdade no meio da janela
                let botao = if verbo == "mouse-voltar" { egui::PointerButton::Extra1 } else { egui::PointerButton::Extra2 };
                vista.clique_pedido = Some((egui::pos2(480.0, 300.0), botao));
                None
            }
            "abrir-acao" => {
                // abrir-acao:ref abre a ação na folha nova
                vista.plano.tela = crate::plano::Sub::Folha;
                vista.plano.aberta = Some(arg.into());
                None
            }
            "marcar-plano" => {
                // marcar-plano:id-ou-ref é o clique na caixinha (`gesto/marcar`): sai quando a folha chegar
                vista.marcar_pedido = Some(arg.into());
                None
            }
            "agentes" => {
                vista.plano.tela = crate::plano::Sub::Agentes;
                None
            }
            "chegou" => {
                vista.plano.tela = crate::plano::Sub::Chegou;
                None
            }
            "demo-escrita" => {
                // demo-escrita:etapa/minutos-parada põe a escrita ao vivo na folha (só com BLUEPRINT_UI_DEMO; nunca grava)
                let (etapa, parada) = arg.split_once('/').unwrap_or((arg, "0"));
                if let (Some(f), true) = (vista.plano.folha.as_mut(), crate::plano::agentes::demo::ativo("escrita")) {
                    crate::plano::agentes::demo::escrita(f, etapa.parse().unwrap_or(0), parada.parse().unwrap_or(0));
                }
                None
            }
            "demo-obra" => {
                // demo-obra:n deixa n planos feitos e o seguinte em andamento, para ver a obra subir (só com BLUEPRINT_UI_DEMO; nunca grava)
                if let (Some(f), true) = (vista.plano.folha.as_mut(), crate::plano::agentes::demo::ativo("obra")) {
                    crate::plano::agentes::demo::obra(f, arg.parse().unwrap_or(0));
                }
                None
            }
            "demo-depende" => {
                // demo-depende:4.1/1.2 faz o plano 4.1 depender do 1.2, para ver a ligação entre ações (só com BLUEPRINT_UI_DEMO; nunca grava)
                if let (Some(f), true, Some((q, d))) = (vista.plano.folha.as_mut(), crate::plano::agentes::demo::ativo("obra"), arg.split_once('/')) {
                    if let Some(p) = f.projetos.iter_mut().flat_map(|p| &mut p.acoes).flat_map(|a| &mut a.planos).find(|p| p.referencia == q) {
                        p.depende_de.push(d.to_string());
                    }
                    // a folha mudou na memória: o que a tela guardou dela (esteiras, miniaturas) é refeito
                    f.renovar();
                }
                None
            }
            "obra-orbita" => {
                // obra-orbita:az/el gira a obra da aba Planta, como o arrastar do mouse
                if let Some((x, y)) = arg.split_once('/') {
                    crate::plano::acao::girar_obra(ctx, x.parse().unwrap_or(0.0), y.parse().unwrap_or(0.0));
                }
                None
            }
            "agente" => {
                vista.agente_aberto = Some(arg.into());
                None
            }
            "rolar" => {
                vista.rolagem = arg.parse().ok();
                None
            }
            "abrir" => {
                vista.tela = crate::telas::Tela::Workspace(arg.into());
                None
            }
            "voltar" => {
                vista.tela = crate::telas::Tela::Entrada;
                None
            }
            "criarbp" => arg.split_once('/').map(|(ws, nome)| Acao::CriarBlueprint { workspace: ws.into(), nome: nome.into(), descricao: String::new() }),
            "novobp" => {
                vista.novo_bp = Some(crate::telas::NovoBp::em(arg));
                None
            }
            "pausa" => {
                // pausa:ms espera antes do próximo passo
                self.1 = arg.parse().ok().map(|ms| std::time::Instant::now() + std::time::Duration::from_millis(ms));
                None
            }
            "novo" => {
                vista.novo = Some(Novo::com_nome(arg));
                None
            }
            _ => None,
        }
    }
}
