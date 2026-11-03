//! O blueprint aberto: a folha `n` do workspace, com a régua e as seções (`folhaBlueprint` em web/js/render.js). Sem projetos há uma
//! seção, a do blueprint, que nasce aberta; com projetos há uma seção por projeto (e o Geral), que nascem recolhidas.

use super::acoes;
use super::cena::{Ancora, Cena};
use super::disposicao::{self, FOLGA};
use super::pagina::{self, corta, Carimbo, Medidas, BAIXO_FAIXA, IL};
use super::regua::{self, Regua};
use super::secao::{self, Secao};
use crate::documento::{Documento, RAIZ};
use crate::dominio::{self, Acao as AcaoDom};
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com, trn};
use crate::pesquisa;
use crate::planta::{self, Papel};
use super::controles::Filtro;
use crate::telas::{Acao, Tela, Vista};
use crate::tema::{MEDIO, TINTA};
use egui::{vec2, Sense, Ui};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;

const Y_REGUA: f32 = 204.0;
/// O vão entre duas seções.
const GAP_SECAO: f32 = 28.0;

fn texto(v: &Value, campo: &str) -> String {
    v.get(campo).and_then(Value::as_str).unwrap_or("").to_string()
}

/// O que cada seção mostra, antes de medir: a chave de aberto/fechado, o número, o nome e as ações.
struct Modelo {
    chave: String,
    numero: String,
    nome: String,
    descricao: String,
    idxs: Vec<usize>,
    principal: bool,
    aberto: bool,
}

fn modelos(vista: &Vista, p: &Value, pi: usize, acoes: &[AcaoDom]) -> Vec<Modelo> {
    let bp = texto(p, "id");
    match dominio::grupos(p, acoes) {
        None => {
            let chave = format!("{bp}|");
            vec![Modelo { aberto: vista.abertas.get(&chave).copied().unwrap_or(true), chave, numero: format!("{:02}", pi + 1), nome: texto(p, "nome"), descricao: texto(p, "descricao"), idxs: (0..acoes.len()).collect(), principal: true }]
        }
        Some(gs) => gs.into_iter().enumerate().map(|(k, g)| {
            let chave = format!("{bp}|{}", g.id);
            let descricao = g.descricao.clone();
            Modelo { aberto: vista.abertas.get(&chave).copied().unwrap_or(false), chave, numero: format!("{:02}", k + 1), nome: if g.id.is_empty() { tr("Geral").to_string() } else { g.nome }, descricao, idxs: g.acoes, principal: false }
        }).collect(),
    }
}

/// O número de cada ação (por `idx`), o mesmo de `dominio::numero_da_acao`, contado uma vez para a folha inteira em vez de uma vez por ação:
/// sem projetos `<blueprint>.<posição>`; com projetos `<grupo>.<posição no grupo>`, valendo o primeiro grupo e o primeiro id repetidos.
fn numeros(p: &Value, pi: usize, acoes: &[AcaoDom]) -> Vec<String> {
    let Some(gs) = dominio::grupos(p, acoes) else {
        return acoes.iter().map(|f| format!("{}.{}", pi + 1, f.idx + 1)).collect();
    };
    let mut numeros = vec![String::new(); acoes.len()];
    for (k, g) in gs.iter().enumerate() {
        if gs[..k].iter().any(|o| o.id == g.id) {
            continue;
        }
        let mut primeiro: HashMap<&str, usize> = HashMap::new();
        for (pos, &i) in g.acoes.iter().enumerate() {
            let n = *primeiro.entry(acoes[i].id.as_str()).or_insert(pos);
            numeros[i] = format!("{}.{}", k + 1, n + 1);
        }
    }
    numeros
}

pub fn tela(ui: &mut Ui, vista: &mut Vista, doc: &Documento, id: &str, bp: &str, servidor: &str) -> Option<Acao> {
    let Some(ws) = doc.workspace(id) else {
        vista.tela = Tela::Entrada;
        return None;
    };
    let blueprints = ws.get("blueprints").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let Some((pi, p)) = blueprints.iter().enumerate().find(|(_, b)| texto(b, "id") == bp) else {
        vista.tela = Tela::Workspace(id.to_string());
        return None;
    };
    let empresa = if id == RAIZ { texto(ws, "empresa") } else { texto(ws, "nome") };
    let acoes = dominio::acoes_de(p);
    // o filtro «Pendentes» muda o que a folha desenha; o resumo das seções continua sobre tudo. Sem filtro, as mesmas ações, sem cópia
    let pendentes = vista.pendentes.contains(bp);
    let visiveis: Cow<[AcaoDom]> = if pendentes { Cow::Owned(dominio::so_pendentes(&acoes)) } else { Cow::Borrowed(acoes.as_slice()) };
    let janela = dominio::janela(&visiveis);
    // a pesquisa: o que casou (só nas ações que a folha mostra) e as seções que precisam estar abertas para o achado aparecer
    let termos = pesquisa::termos(&vista.consulta);
    let ids_visiveis: Vec<&str> = visiveis.iter().map(|f| f.id.as_str()).collect();
    let res = pesquisa::blueprint(p, &termos, Some(&ids_visiveis));
    let mut modelos = modelos(vista, p, pi, &acoes);
    if !termos.is_empty() {
        for m in &mut modelos {
            if m.idxs.iter().any(|k| res.acoes.iter().any(|a| a.id == acoes[*k].id)) {
                m.aberto = true;
            }
        }
    }
    // «Ir para esta ação»: a seção dela abre, para a ação não ficar compacta (`irPara`)
    if let Some(alvo) = vista.ir_para.clone() {
        if let Some(m) = modelos.iter_mut().find(|m| m.idxs.iter().any(|k| acoes[*k].id == alvo)) {
            m.aberto = true;
            vista.abertas.insert(m.chave.clone(), true);
        }
    }
    let contador = (!termos.is_empty()).then(|| if res.total() == 0 { tr("nenhum resultado").to_string() } else { trn(res.total(), "{n} resultado", "{n} resultados") });
    if pendentes {
        // com filtro, a seção sem nada a mostrar some (só quando há projetos: o blueprint inteiro continua com a mensagem)
        let tem_projetos = !modelos.iter().any(|m| m.principal);
        let ids: std::collections::HashSet<&str> = visiveis.iter().map(|f| f.id.as_str()).collect();
        if tem_projetos {
            modelos.retain(|m| m.idxs.iter().any(|k| ids.contains(acoes[*k].id.as_str())));
        }
    }
    let (viewport_h, viewport_w) = (ui.available_height(), ui.available_width());
    let (mut pedido, mut alternar, mut comando, mut filtro) = (None, None::<String>, None, None);
    let mut area = egui::ScrollArea::both().auto_shrink([false, false]);
    if let Some(y) = vista.rolagem.take() {
        area = area.vertical_scroll_offset(y);
    }
    crate::rolagem::mostrar(ui, area, |ui| {
        let w = viewport_w.max(960.0);
        let (l, r) = (IL + 34.0, w - IL - 50.0);
        let colw = (((r - l - 30.0) / 2.0).floor()).clamp(240.0, 430.0);
        let y0 = Y_REGUA + 90.0;
        // mede cada seção e empilha: a próxima começa depois da altura da anterior (sem a folga) e do vão
        let subconjuntos: Vec<Vec<&AcaoDom>> = modelos.iter().map(|m| {
            let ids: std::collections::HashSet<&str> = m.idxs.iter().map(|k| acoes[*k].id.as_str()).collect();
            visiveis.iter().filter(|f| ids.contains(f.id.as_str())).collect()
        }).collect();
        let ancoras: super::claude::Ancoras = Default::default();
        let aberta: std::cell::RefCell<Option<Acao>> = Default::default();
        let mut disps = Vec::new();
        let mut ys = Vec::new();
        let mut y = y0;
        let numeros = numeros(p, pi, &acoes);
        for (m, refs) in modelos.iter().zip(&subconjuntos) {
            let d = disposicao::dispor(refs, m.aberto, |f| numeros[f.idx].clone(), &janela, l, r, colw);
            ys.push(y);
            y += d.alto - FOLGA + GAP_SECAO;
            disps.push(d);
        }
        // rola até a ação no próximo quadro, com ela a pouco mais de um quinto da altura da janela (`rolarParaAcao`)
        if let Some(alvo) = vista.ir_para.take() {
            for (k, (d, refs)) in disps.iter().zip(&subconjuntos).enumerate() {
                if let Some(pos) = d.posicoes.iter().find(|q| refs[q.i].id == alvo) {
                    vista.rolagem = Some((ys[k] + disposicao::PH + d.linha_y[pos.linha] - viewport_h * 0.22).max(0.0));
                    crate::anima::mexer(ui.ctx(), 350.0);
                }
            }
        }
        let alto = if modelos.is_empty() { disposicao::PH + 36.0 + FOLGA } else { y - y0 - GAP_SECAO + FOLGA };
        let h = viewport_h.max(560.0).max(y0 + alto + 40.0 + BAIXO_FAIXA + IL);
        let (pagina_r, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
        let painter = ui.painter().clone();
        planta::papel(&painter, pagina_r, Papel::Entrada);
        crate::menu::marcar(ui, pagina_r, || crate::menu::Alvo::Folha { ws: id.to_string(), bp: bp.to_string(), nome: texto(p, "nome"), chaves: modelos.iter().map(|m| m.chave.clone()).collect(), aberto: modelos.iter().any(|m| m.aberto) });
        let c = Cena { p: &painter, o: pagina_r.min };
        let regua = Regua { janela: &janela, l, r, y: Y_REGUA, fundo: h - IL - BAIXO_FAIXA - 20.0 };
        let medidas = Medidas { w, h, l, r, meses: regua.marcas() };
        pagina::moldura(&c, &medidas);
        let titulo = format!("{empresa} · {}", texto(ws, "titulo"));
        // as larguras do título e do subtítulo (em caixa alta, `.06em`) dizem onde cabem os controles, no fim
        let titulo_w = c.texto(&titulo, l, IL + 36.0, 18.0, Peso::Negrito, TINTA, Ancora::Inicio, true);
        let sub = tr_com("Folha {n} de {total} · Blueprint {bp} · {nome}", &[("n", &(pi + 2).to_string()), ("total", &(1 + blueprints.len()).to_string()), ("bp", &format!("{:02}", pi + 1)), ("nome", &corta(&texto(p, "nome"), 60))]);
        let sub_w = c.texto(&sub, l, IL + 56.0, 11.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        regua::desenhar(&c, ui, &regua);
        // uma vez por quadro, e não uma por seção
        let concluido = dominio::concluido(p);
        for (k, m) in modelos.iter().enumerate() {
            let refs = &subconjuntos[k];
            let total = if m.principal { p.get("acoes").and_then(Value::as_array).map_or(0, Vec::len) } else { m.idxs.len() };
            let s = Secao { ws: id, p, acoes: refs, numero: m.numero.clone(), nome: m.nome.clone(), descricao: m.descricao.clone(), total, ok: m.principal && concluido, aberto: m.aberto, principal: m.principal, resumo: dominio::resumo(m.idxs.iter().map(|k| &acoes[*k])), ancoras: &ancoras, aberta: &aberta, pesq: (!termos.is_empty()).then_some((&res, termos.as_slice())), disp: &disps[k], y: ys[k], colw, r, l };
            // a seção anda até o lugar novo quando outra acima abre ou fecha; a que surge (filtro) aparece aos poucos
            let (v, nasce) = crate::anima::deslize(ui.ctx(), ("secao", m.chave.as_str()), &[ys[k]]);
            let pincel = super::barra::opaco(&painter, nasce);
            let cs = Cena { p: &pincel, o: c.o + vec2(0.0, v[0] - ys[k]) };
            if secao::cabecalho(&cs, ui, &s) {
                alternar = Some(m.chave.clone());
            }
            if let Some(a) = acoes::desenhar(&cs, ui, &s) {
                pedido = Some(a);
            }
        }
        if let Some(a) = aberta.borrow_mut().take() {
            pedido = Some(a);
        }
        // o Claude: o cabeçalho do blueprint é o da primeira seção; as figuras vão por cima de tudo
        ancoras.borrow_mut().insert(crate::agentes::Lugar::Cab(bp.to_string()), (l, y0 - 8.0, r - l, 86.0));
        let ags: Vec<_> = crate::agentes::ativos(&vista.agentes, id, crate::agentes::agora_ms(), vista.agentes_vida, &vista.vistos).into_iter().filter(|a| a.blueprint_id == bp).collect();
        if let Some(n) = super::claude::desenhar(&c, ui, bp, bp, &ags, &ancoras, "cab") {
            vista.agente_aberto = Some(n);
        }
        let (feitas, total) = (acoes.iter().filter(|f| f.status == dominio::Situacao::Concluido).count(), acoes.len());
        let nome = texto(p, "nome");
        let dica = tr("Marque planos e checkpoints clicando neles · passe o mouse sobre uma ação para ver os detalhes.");
        pagina::rodape(&c, &medidas, &dica, &Carimbo { empresa: &empresa, tela: if nome.is_empty() { tr("Sem nome") } else { &nome }, progresso: dominio::progresso(p), folha: pi + 2, folhas: 1 + blueprints.len(), legenda: false, feitas, acoes: total });
        // a barra cheia só cabe se sobrar espaço depois do título e do subtítulo (24 px de respiro)
        let (titulo_fim, sub_fim) = (l + titulo_w, l + sub_w);
        let mut abrir_web = None;
        (comando, filtro) = super::controles::blueprint(ui, vista, &c, w, id, titulo_fim.max(sub_fim) + 24.0 - c.o.x, pendentes, contador.clone(), &mut abrir_web);
        if let Some(destino) = abrir_web {
            ui.ctx().open_url(egui::OpenUrl::new_tab(format!("{servidor}/{destino}")));
        }
    });
    // o que mexe na folha anda em vez de pular (anima::mexer); trocar o filtro também reescreve a folha
    if alternar.is_some() || comando.is_some() || matches!(pedido, Some(Acao::Marcar(..))) {
        crate::anima::mexer(ui.ctx(), 350.0);
    }
    if filtro.is_some() {
        crate::anima::reiniciar(ui.ctx());
        crate::anima::mexer(ui.ctx(), 650.0);
    }
    if let Some(chave) = alternar {
        let aberto = modelos.iter().find(|m| m.chave == chave).is_some_and(|m| m.aberto);
        vista.abertas.insert(chave, !aberto);
    }
    // os botões «Todo o blueprint» e «Pendentes»
    match filtro {
        Some(Filtro::Todo) => {
            vista.pendentes.remove(bp);
        }
        Some(Filtro::Pendentes) => {
            if !vista.pendentes.remove(bp) {
                vista.pendentes.insert(bp.to_string());
            }
        }
        None => {}
    }
    // «expandir tudo» e «recolher tudo»
    if let Some(c) = comando {
        for m in &modelos {
            vista.abertas.insert(m.chave.clone(), c == "expandir");
        }
    }
    pedido
}

#[cfg(test)]
mod testes {
    use super::*;

    /// `numeros` dá de uma vez o mesmo número que `dominio::numero_da_acao` dá ação por ação: sem projetos, com projetos e com o Geral.
    #[test]
    fn numeros_batem_com_numero_da_acao() {
        let com = serde_json::json!({ "projetos": [{ "id": "A" }, { "id": "B" }], "acoes": [
            { "id": "a1", "projetoId": "A" }, { "id": "a2", "projetoId": "B" }, { "id": "a3", "projetoId": "A" },
            { "id": "a4" }, { "id": "a5", "projetoId": "inexistente" } ] });
        let sem = serde_json::json!({ "acoes": [{ "id": "x" }, { "id": "y" }] });
        for (p, pi) in [(&com, 0), (&sem, 3)] {
            let fs = dominio::acoes_de(p);
            let esperado: Vec<String> = fs.iter().map(|f| dominio::numero_da_acao(p, f, pi, &fs)).collect();
            assert_eq!(numeros(p, pi, &fs), esperado);
        }
        assert_eq!(numeros(&com, 0, &dominio::acoes_de(&com)), ["1.1", "2.1", "1.2", "3.1", "3.2"]);
    }
}
