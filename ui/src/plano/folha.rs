//! A folha do blueprint, com a moldura, a régua de meses, o título e o carimbo da folha original (`folha/blueprint.rs`, `pagina.rs`, `regua.rs`): os projetos e o bloco de cada ação na data dela.
//! As linhas entram em cascata, a ação abre e fecha com a altura animada e a rolagem anda até ela; sem `BLUEPRINT_UI_SEM_ANIMACAO`, só pede quadro enquanto algo se mexe.

use super::caixinha::Lapis;
use super::escala::{Escala, LINHA_FOLHA as LINHA, MARGEM, X0};
use super::linha::{self, Pincel};
use super::modelo::{Acao as AcaoPlano, Folha};
use super::revisao::{self, Peca};
use super::{acao, escrita, pasta, Sub};
use crate::documento::Documento;
use crate::menu::{self, Alvo};
use crate::folha::cena::Cena;
use crate::folha::pagina::{self, Carimbo, Medidas, BAIXO_FAIXA, IL};
use crate::folha::regua::{self, Regua};
use crate::dominio::{self, Janela};
use crate::documento::{self, RAIZ};
use crate::i18n::{tr, tr_com};
use crate::planta::{self, botao, Botao, Papel};
use crate::telas::{Acao, Tela, Vista};
use egui::{pos2, vec2, Id, Rect, Sense, Ui, UiBuilder};
use serde_json::Value;

#[path = "folha_botoes.rs"]
pub mod botoes_topo;
#[path = "folha_topo.rs"]
mod cabeca;
#[path = "folha_disposicao.rs"]
mod disposicao;
#[path = "folha_estilo.rs"]
pub mod estilo;
pub use botoes_topo::Topo;
pub use cabeca::links;
pub use disposicao::{empilhar, progresso, Disposicao};

use botoes_topo::{fileira_atencao, fileira_principal, fileiras, largura_fileira, ALEM_DA_REGUA, TITULO_MAX};
use cabeca::{escrever_aviso, titulo_folha};
use disposicao::{decisoes, soltas, X_DECISAO};

pub const TEXTOS: &[(&str, &str)] = &[
    ("Aguardando você · {n}", "Waiting for you · {n}"),
    ("O que chegou", "What arrived"),
    ("Agentes", "Agents"),
    ("Agentes · {n}", "Agents · {n}"),
    ("Folha antiga", "Old sheet"),
    ("← Blueprints", "← Blueprints"),
    ("Planejar", "Plan"),
    ("Compartilhar", "Share"),
    ("Apresentar", "Present"),
    ("Folha {n} de {total}", "Sheet {n} of {total}"),
    ("{n} de {total}", "{n} of {total}"),
    ("Workspace", "Workspace"),
    ("Blueprint", "Blueprint"),
    ("Progresso", "Progress"),
    ("Situação", "Status"),
    ("Concluído", "Done"),
    ("Em andamento", "In progress"),
    ("Planejado", "Planned"),
    ("Meu workspace", "My workspace"),
    ("Ações", "Actions"),
    ("Folha", "Sheet"),
    ("concluída", "done"),
    ("em andamento", "in progress"),
    ("aguardando", "waiting"),
    ("planejada", "planned"),
    ("checkpoint", "checkpoint"),
    ("proposta", "proposal"),
    ("Decisões · {n}", "Decisions · {n}"),
    ("+{n} links", "+{n} links"),
];

/// As medidas da folha original.
const Y_REGUA: f32 = 204.0;
const Y_SEGUNDA: f32 = 124.0;
/// Onde o texto da ação aberta começa, depois da cota de dias.
const Y_TEXTO_ABERTA: f32 = 32.0;
const Y_AVISO: f32 = 112.0;
const LARGURA_MIN: f32 = 960.0;
const ALTURA_MIN: f32 = 560.0;
const VAO_FAIXA: f32 = 12.0;
/// A ação abre e fecha em 0,5 s.
const TEMPO_ABRIR: f32 = 0.5;

/// Abre a ação e rola até ela. A posição sai da disposição com ela já aberta: o painel que estava aberto acima deixa de contar.
fn abrir(vista: &mut Vista, ctx: &egui::Context, f: &Folha, alvo: &AcaoPlano, dispor: &dyn Fn(Option<&str>) -> Disposicao, altura: f32) {
    let d = dispor(Some(&alvo.referencia));
    if let Some((_, _, y)) = d.linhas.iter().find(|(p, a, _)| f.projetos[*p].acoes[*a].id == alvo.id) {
        vista.rolagem = Some((y - altura * 0.22).max(0.0));
    }
    vista.plano.aberta = Some(alvo.referencia.clone());
    ctx.request_repaint();
}

/// O que o clique num botão do topo faz. `Web`: as telas que ainda são da página.
#[allow(clippy::too_many_arguments)]
fn aplicar(vista: &mut Vista, ui: &Ui, f: &Folha, dispor: &dyn Fn(Option<&str>) -> Disposicao, t: Topo, ws: &str, bp: &str, servidor: &str, altura: f32) {
    match t {
        Topo::Aguardando => {
            // a primeira ação com o que aprovar abre no lugar; sem nenhuma (só decisão solta), vai ao resumo do que chegou
            match f.projetos.iter().flat_map(|p| &p.acoes).find(|a| a.numeros.aprovar > 0) {
                Some(a) => abrir(vista, ui.ctx(), f, a, dispor, altura),
                None => vista.plano.tela = Sub::Chegou,
            }
        }
        Topo::Chegou => vista.plano.tela = Sub::Chegou,
        Topo::Agentes => vista.plano.tela = Sub::Agentes,
        Topo::FolhaAntiga => {
            vista.folha_antiga.insert(bp.to_string());
        }
        Topo::Voltar => vista.tela = Tela::Workspace(ws.to_string()),
        Topo::Web => ui.ctx().open_url(egui::OpenUrl::new_tab(format!("{servidor}/#/c/{ws}"))),
    }
}

pub fn tela(ui: &mut Ui, vista: &mut Vista, doc: &Documento, ws: &str, bp: &str, servidor: &str) -> Option<Acao> {
    let Some(w) = doc.workspace(ws) else {
        vista.tela = Tela::Entrada;
        return None;
    };
    let lista = w.get("blueprints").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    if !lista.iter().any(|b| b.get("id").and_then(Value::as_str) == Some(bp)) {
        vista.tela = Tela::Workspace(ws.to_string());
        return None;
    }
    // a folha sai do estado enquanto desenha, para a tela poder mudar o resto de `vista`, sem copiá-la a cada quadro
    let f = vista.plano.folha.take()?;
    let Some((pi, p)) = lista.iter().enumerate().find(|(_, b)| b.get("id").and_then(Value::as_str) == Some(bp)) else { return None };
    let campo = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let empresa = if ws == RAIZ { campo(w, "empresa") } else { campo(w, "nome") };
    let hoje_iso = documento::hoje();
    let hoje = documento::dia(&hoje_iso).unwrap_or(0);
    let acoes = dominio::acoes_de(p);
    let escolhido = vista.periodos.get(bp).and_then(|q| if q.aguardando { q.anterior.clone() } else { Some((q.inicio.clone(), q.fim.clone())) });
    let janela = dominio::janela_com(&acoes, hoje, escolhido.as_ref().map(|(a, b)| (a.as_str(), b.as_str())));
    vista.janelas.insert(bp.to_string(), (dominio::mes_iso(janela.inicio), dominio::mes_iso(janela.fim)));
    let nome = campo(p, "nome");
    let sub = tr_com("Folha {n} de {total} · Blueprint {bp} · {nome}", &[("n", &(pi + 2).to_string()), ("total", &(1 + lista.len()).to_string()), ("bp", &format!("{:02}", pi + 1)), ("nome", &pagina::corta(&nome, 60))]);
    // sem nome de workspace (a raiz nova), o título é o do blueprint
    let partes: Vec<String> = [empresa.clone(), campo(w, "titulo")].into_iter().filter(|t| !t.is_empty()).collect();
    let titulo = if partes.is_empty() { nome.clone() } else { partes.join(" · ") };
    let topo = Cabecalho { titulo, sub, empresa, nome, data: hoje_iso.split('-').rev().collect::<Vec<_>>().join("/"), folha: pi + 2, folhas: 1 + lista.len(), janela, hoje };
    let pedido = desenhar(ui, vista, &f, ws, bp, servidor, &topo);
    vista.plano.folha = Some(f);
    pedido
}

/// O que a ação ocupa além da linha agora: o painel e o vão até a próxima, na parte `t` que já abriu. Aberta por inteiro, segue a altura do painel,
/// que anda sozinha quando o conteúdo muda («ver tudo»).
fn ocupa(ctx: &egui::Context, p: &egui::Painter, f: &Folha, a: &AcaoPlano, x1: f32, t: f32) -> f32 {
    let (alto, tudo) = (acao::altura(p, f, a, x1), acao::ocupa(p, f, a, x1));
    if t < 0.999 {
        return tudo * t;
    }
    let vao = tudo - alto;
    (crate::anima::deslize(ctx, ("acao-altura", a.referencia.as_str()), &[alto]).0[0] + vao).max(vao)
}

/// O painel da ação, no `y` dela. Abrindo ou fechando (`abertura` < 1), o painel inteiro fica no lugar e só a janela que o mostra cresce de cima para baixo, com a opacidade subindo.
#[allow(clippy::too_many_arguments)]
fn painel(ui: &mut Ui, c: &Pincel, lugar: revisao::Alvo, f: &Folha, item: &AcaoPlano, y: f32, x1: f32, abertura: f32, altura: f32) -> Option<acao::Saida> {
    if abertura >= 0.999 {
        return acao::painel(ui, c, lugar, f, item, y, x1);
    }
    let janela = Rect::from_min_max(c.pt(0.0, y - 1.0), c.pt(x1 + MARGEM, y + altura * abertura));
    let mut filho = ui.new_child(UiBuilder::new().max_rect(ui.max_rect()).id_salt(("plano-painel-anima", item.id.as_str())));
    filho.set_clip_rect(janela.intersect(ui.clip_rect()));
    filho.set_opacity(estilo::suave((abertura * 1.6).min(1.0)));
    let pintor = filho.painter().clone();
    acao::painel(&mut filho, &Pincel { p: &pintor, o: c.o }, lugar, f, item, y, x1)
}

/// O que a folha mostra e vem do documento, não do plano: os nomes, a folha e a janela de meses da régua.
struct Cabecalho {
    titulo: String,
    sub: String,
    empresa: String,
    nome: String,
    data: String,
    folha: usize,
    folhas: usize,
    janela: Janela,
    hoje: i64,
}

#[allow(clippy::too_many_arguments)]
fn desenhar(ui: &mut Ui, vista: &mut Vista, f: &Folha, ws: &str, bp: &str, servidor: &str, k: &Cabecalho) -> Option<Acao> {
    let ctx = ui.ctx().clone();
    let (alto_janela, largura_janela) = (ui.available_height(), ui.available_width());
    let w = largura_janela.max(LARGURA_MIN);
    let x1 = w - MARGEM;
    escrita::acompanhar(&ctx, &mut vista.plano, f);
    let notas: f32 = soltas(f).map(|k| revisao::altura_nota(ui.painter(), &Peca::decisao(f, k), x1 - X_DECISAO)).sum();
    // as faixas de `semPasta` e da escrita ficam entre a régua e o primeiro projeto
    let y_faixa = Y_REGUA + 44.0;
    let y_escrita = y_faixa + if f.sem_pasta { pasta::ALTURA + VAO_FAIXA } else { 0.0 };
    let y_fim_faixas = y_escrita + if f.escrita.is_some() { escrita::altura(f) + VAO_FAIXA } else { 0.0 };
    let y0 = if y_fim_faixas > y_faixa { y_fim_faixas + 16.0 } else { Y_REGUA + 90.0 };
    let medida = ui.painter().clone();
    let aberta = vista.plano.aberta.clone();
    // quanto cada ação já abriu (0 a 1): o painel cresce e encolhe pela curva do Anatomy
    let abertura = |a: &AcaoPlano| estilo::tempo(&ctx, Id::unique(("plano-abertura", a.id.as_str())), aberta.as_deref() == Some(a.referencia.as_str()), TEMPO_ABRIR);
    // o destino: a ação pedida já aberta por inteiro (para rolar até ela); e o que se vê agora, com as alturas a meio caminho
    let dispor = |alvo: Option<&str>| empilhar(f, y0, &|a| if alvo == Some(a.referencia.as_str()) { acao::ocupa(&medida, f, a, x1) } else { 0.0 }, notas);
    let agora = || empilhar(f, y0, &|a| { let t = abertura(a); if t > 0.0 { ocupa(&ctx, &medida, f, a, x1, t) } else { 0.0 } }, notas);
    // «Ir para esta ação»: abre a ação e rola até ela
    if let Some(alvo) = vista.ir_para.take() {
        if let Some(a) = f.projetos.iter().flat_map(|p| &p.acoes).find(|a| a.id == alvo) {
            abrir(vista, &ctx, f, a, &dispor, alto_janela);
        }
    }
    let rolagem = vista.rolagem.take();
    let d = agora();
    let aviso = vista.plano.aviso.clone();
    let trabalhando = vista.plano.agentes.as_ref().map(|a| a.numeros.trabalhando);
    let entrada = estilo::cascata(&ctx, bp, d.linhas.len() + d.projetos.len());
    let antes: Vec<usize> = f.projetos.iter().scan(0, |n, p| Some(std::mem::replace(n, *n + p.acoes.len()))).collect();
    let (mut pedido, mut alternar, mut clique_topo, mut saida, mut mes_clicado) = (None, None::<String>, None, None, None);
    let aguardando = vista.periodos.get(bp).filter(|q| q.aguardando).map(|q| q.inicio.clone());
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        let h = alto_janela.max(ALTURA_MIN).max(d.fim + 40.0 + BAIXO_FAIXA + IL);
        let (pagina, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
        if let Some(y) = rolagem {
            let vis = ui.clip_rect();
            let alvo = Rect::from_min_max(pos2(vis.min.x, pagina.min.y + y), pos2(vis.max.x, pagina.min.y + y + 1.0));
            let anda = if estilo::ligado() { egui::style::ScrollAnimation::new(1800.0, egui::Rangef::new(0.3, 0.65)) } else { egui::style::ScrollAnimation::none() };
            ui.scroll_to_rect_animation(alvo, Some(egui::Align::Min), anda);
        }
        let painter = ui.painter().clone();
        planta::papel(&painter, pagina, Papel::Entrada);
        menu::marcar(ui, pagina, || Alvo::Folha { ws: ws.into(), bp: bp.into(), nome: f.blueprint.nome.clone(), chaves: Vec::new(), aberto: false });
        let c = Pincel { p: &painter, o: pagina.min };
        let recorte = Rect::from_min_max(pos2(pagina.min.x + X0, pagina.min.y), pos2(pagina.min.x + x1, pagina.max.y)).intersect(painter.clip_rect());
        let e = Escala::da_janela(&k.janela, k.hoje, X0, x1);
        let (parte, feitas, acoes) = progresso(f);
        let cena = Cena { p: &painter, o: pagina.min };
        // o eixo do hoje para antes do texto da ação aberta e volta depois dela
        let sem_eixo = d.linhas.iter().find_map(|(p, a, y)| {
            let item = &f.projetos[*p].acoes[*a];
            let t = abertura(item);
            (t > 0.0).then(|| (y + LINHA / 2.0 + Y_TEXTO_ABERTA, y + LINHA / 2.0 + acao::altura(&painter, f, item, x1) * t))
        });
        let g = Regua { janela: &k.janela, l: X0, r: x1, y: Y_REGUA, fundo: h - IL - BAIXO_FAIXA - 20.0, hoje: k.hoje, sem_eixo };
        let nm = (k.janela.fim - k.janela.inicio + 1) as usize;
        let medidas = Medidas { w, h, l: X0, r: x1, meses: (0..=nm).map(|m| g.mes(m)).collect() };
        pagina::moldura(&cena, &medidas);
        let (principal, atencao) = (fileira_principal(), fileira_atencao(f, trabalhando));
        let max_titulo = (x1 + ALEM_DA_REGUA - largura_fileira(ui, &principal) - 24.0 - X0).min(TITULO_MAX);
        pedido = titulo_folha(ui, &c, f, &k.titulo, &k.sub, max_titulo, ws, bp);
        let antiga = Botao { texto: tr("Folha antiga"), tam: 10.0, cheio: false, ativo: true, altura: 27.0, pad_x: 12.0 };
        let largura_antiga = antiga.largura(&painter);
        if botao(ui, c.pt(X0, Y_SEGUNDA), &antiga).clicked() {
            clique_topo = Some(Topo::FolhaAntiga);
        }
        if !f.blueprint.links.is_empty() {
            links(ui, &c, X0 + largura_antiga + 14.0, Y_SEGUNDA + 18.0, x1, &f.blueprint.links, "bp");
        }
        if !aviso.is_empty() {
            escrever_aviso(ui, &c, &aviso, Y_AVISO);
        }
        mes_clicado = regua::desenhar(&cena, ui, &g, aguardando.as_deref());
        let visivel = ui.clip_rect();
        let lugar = revisao::Alvo { estado: &vista.plano, servidor, ws, bp };
        // cada peça entra na sua vez: sobe uns pontos enquanto aparece, e o cronograma dela se traça da esquerda para a direita
        let com_vez = |vez: f32| {
            let mut p = painter.clone();
            p.multiply_opacity(vez);
            let traco = Rect::from_min_max(recorte.min, pos2(recorte.min.x + recorte.width() * vez, recorte.max.y));
            (p.clone(), p.with_clip_rect(traco), pagina.min + vec2(0.0, (1.0 - vez) * 10.0))
        };
        for (n, (p, y)) in f.projetos.iter().zip(&d.projetos).enumerate() {
            let (pa, pr, o) = com_vez(entrada(antes[n] + n));
            linha::cabecalho(&Pincel { p: &pa, o }, &Pincel { p: &pr, o }, &e, p, &f.blueprint.nome, n + 1, *y);
        }
        for (p, a, y) in &d.linhas {
            let item = &f.projetos[*p].acoes[*a];
            let t = abertura(item);
            // a linha aberta pode ter saído da tela e o painel dela, não
            let alto_painel = if t > 0.0 { acao::altura(&painter, f, item, x1) } else { 0.0 };
            if !visivel.intersects(c.caixa(0.0, y - LINHA, w, LINHA * 2.0 + alto_painel * t)) {
                continue;
            }
            let (pa, pr, o) = com_vez(entrada(antes[*p] + p + 1 + a));
            let (l, r) = (Pincel { p: &pa, o }, Pincel { p: &pr, o });
            if linha::desenhar(ui, &l, &r, &e, item, ws, bp, *y, t) {
                alternar = Some(item.referencia.clone());
            }
            escrita::etiqueta(&r, &e, f, item, *y);
            if t > 0.0 {
                if let Some(s) = painel(ui, &c, lugar, f, item, y + LINHA / 2.0, x1, t, alto_painel) {
                    // fechando, o painel já não responde: só a ação que está aberta decide
                    if aberta.as_deref() == Some(item.referencia.as_str()) {
                        saida = Some((item, s));
                    }
                }
            }
        }
        pasta::faixa(Lapis { p: &painter, o: pagina.min }, ui, &vista.plano, servidor, ws, bp, f.sem_pasta, X0, y_faixa, x1 - X0);
        escrita::faixa(ui, &c, &vista.plano, servidor, f, X0, y_escrita, x1 - X0);
        decisoes(ui, &c, f, lugar, d.y_decisoes, x1);
        pagina::rodape(&cena, &medidas, "", &Carimbo { empresa: &k.empresa, tela: if k.nome.is_empty() { tr("Sem nome") } else { &k.nome }, data: &k.data, progresso: parte, folha: k.folha, folhas: k.folhas, legenda: false, feitas, acoes });
        if let Some(t) = fileiras(ui, &c, x1, &principal, &atencao) {
            clique_topo = Some(t);
        }
    });
    if let Some(mes) = mes_clicado {
        crate::folha::blueprint::selecionar_mes(vista, bp, mes);
        ctx.request_repaint();
    }
    if let Some(t) = clique_topo {
        aplicar(vista, ui, f, &dispor, t, ws, bp, servidor, alto_janela);
    }
    if let Some((item, s)) = saida {
        pedido = acao::aplicar(&mut vista.plano, item, s).or(pedido);
    }
    if let Some(referencia) = alternar {
        vista.plano.aberta = if vista.plano.aberta.as_deref() == Some(referencia.as_str()) { None } else { Some(referencia) };
        ctx.request_repaint();
    }
    pedido
}
