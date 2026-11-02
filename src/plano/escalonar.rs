//! Quem pode rodar agora, sem dois agentes pisarem um no outro (porte de `design/planejar/agentes.mjs`). Puro: a tela e o motor recebem a mesma resposta.
use super::ids::area_normal;
use std::collections::{BTreeMap, BTreeSet};

/// Sem `limite` na entrada, quantos agentes rodam juntos.
const LIMITE_PADRAO: usize = 4;

/// `chave`: `<bp>|<ref>`; `depende_de` em chaves; `exclusivo`: pasta sem git, um agente por pasta.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlanoEsc { pub chave: String, pub area: Option<String>, pub exclusivo: Option<String>, pub depende_de: Vec<String> }
/// `limite` None = 4. O plano que pediu ajuda vem em `bloqueados` e `rodando`: segue segurando vaga e área. `seguram_area`: o agente saiu, o trabalho ficou na cópia; conflita, mas não ocupa vaga.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EntradaEsc { pub iniciados: Vec<String>, pub prontos: Vec<String>, pub bloqueados: Vec<String>, pub causas: BTreeMap<String, Causa>, pub travados: Vec<String>, pub rodando: Vec<String>, pub seguram_area: Vec<String>, pub limite: Option<usize> }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Causa { Ajuda, Sumiu, Falhou(String) }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situacao { Nao, Revisao, Trabalhando, Fila, Dependencia, Area, Parado, Bloqueado, Pronto }
/// `por`: o plano que segura este. `area`: o que está em disputa, só em `Area` (a pasta, quando o conflito é de `exclusivo`). `causa`: só em `Bloqueado`.
#[derive(Debug, Clone, PartialEq)]
pub struct EstadoEsc { pub situacao: Situacao, pub por: Option<String>, pub area: Option<String>, pub sem_iniciar: Vec<String>, pub causa: Option<Causa> }

impl EstadoEsc {
    fn so(situacao: Situacao) -> Self {
        Self { situacao, por: None, area: None, sem_iniciar: Vec::new(), causa: None }
    }
}

/// Iguais ou uma dentro da outra (`app` e `app/barra`), comparadas por `area_normal`. Sem área, ou com área que normaliza para nada, não conflita com ninguém.
pub fn areas_conflitam(a: Option<&str>, b: Option<&str>) -> bool {
    let (Some(a), Some(b)) = (a, b) else { return false };
    let (x, y) = (area_normal(a), area_normal(b));
    if x.is_empty() || y.is_empty() {
        return false;
    }
    x == y || x.starts_with(&format!("{y}/")) || y.starts_with(&format!("{x}/"))
}

fn conjunto(lista: &[String]) -> BTreeSet<&str> {
    lista.iter().map(String::as_str).collect()
}

/// O que `q` disputa com `outro`: a área ou, se for o caso, a pasta sem git. `None` = podem rodar juntos.
fn disputa(q: &PlanoEsc, outro: &PlanoEsc) -> Option<String> {
    if areas_conflitam(q.area.as_deref(), outro.area.as_deref()) {
        return q.area.clone();
    }
    q.exclusivo.clone().filter(|pasta| outro.exclusivo.as_ref() == Some(pasta))
}

/// A situação de cada plano. Precedência: pronto > bloqueado > rodando > travado > iniciado > não iniciado. Os iniciados disputam vaga em ordem de documento.
pub fn escalonar(planos: &[PlanoEsc], e: &EntradaEsc) -> BTreeMap<String, EstadoEsc> {
    let (iniciados, prontos, bloqueados) = (conjunto(&e.iniciados), conjunto(&e.prontos), conjunto(&e.bloqueados));
    let (travados, rodando, seguram) = (conjunto(&e.travados), conjunto(&e.rodando), conjunto(&e.seguram_area));
    let limite = e.limite.unwrap_or(LIMITE_PADRAO).max(1);

    let mut agentes = planos.iter().filter(|q| rodando.contains(q.chave.as_str())).count();
    // Quem já segura algo. Inclui o próprio plano: cancelado e marcado de novo, espera o agente antigo sair da cópia.
    let mut ativos: Vec<&PlanoEsc> = planos.iter().filter(|q| rodando.contains(q.chave.as_str()) || seguram.contains(q.chave.as_str())).collect();
    let mut saida = BTreeMap::new();

    for q in planos {
        let k = q.chave.as_str();
        let fixo = if prontos.contains(k) {
            Some(EstadoEsc::so(Situacao::Pronto))
        } else if bloqueados.contains(k) {
            Some(EstadoEsc { causa: e.causas.get(k).cloned(), ..EstadoEsc::so(Situacao::Bloqueado) })
        } else if rodando.contains(k) {
            Some(EstadoEsc::so(Situacao::Trabalhando))
        } else if travados.contains(k) {
            Some(EstadoEsc::so(Situacao::Revisao))
        } else if !iniciados.contains(k) {
            Some(EstadoEsc::so(Situacao::Nao))
        } else {
            None
        };
        if let Some(estado) = fixo {
            saida.insert(q.chave.clone(), estado);
        }
    }

    for q in planos {
        if saida.contains_key(&q.chave) {
            continue;
        }
        let faltam: Vec<&String> = q.depende_de.iter().filter(|d| !prontos.contains(d.as_str())).collect();
        let estado = if let Some(preso) = faltam.iter().find(|d| bloqueados.contains(d.as_str())) {
            EstadoEsc { por: Some((*preso).clone()), ..EstadoEsc::so(Situacao::Parado) }
        } else if let Some(primeiro) = faltam.first() {
            // Quem já roda foi iniciado, embora `iniciados` traga só as marcadas (`entrada_esc`).
            let sem_iniciar = faltam.iter().filter(|d| !iniciados.contains(d.as_str()) && !rodando.contains(d.as_str())).map(|d| (*d).clone()).collect();
            EstadoEsc { por: Some((*primeiro).clone()), sem_iniciar, ..EstadoEsc::so(Situacao::Dependencia) }
        } else if let Some((ocupa, area)) = ativos.iter().find_map(|o| disputa(q, o).map(|d| (o.chave.clone(), d))) {
            EstadoEsc { por: Some(ocupa), area: Some(area), ..EstadoEsc::so(Situacao::Area) }
        } else if agentes >= limite {
            EstadoEsc::so(Situacao::Fila)
        } else {
            agentes += 1;
            ativos.push(q);
            EstadoEsc::so(Situacao::Trabalhando)
        };
        saida.insert(q.chave.clone(), estado);
    }
    saida
}
