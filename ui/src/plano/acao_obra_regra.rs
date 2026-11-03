//! O que a planta sabe sem ninguém perguntar: quem ainda não pode iniciar, qual é o próximo passo e o que cada tipo de peça pede da pessoa.
//! Só regra, sem desenho. O servidor continua aceitando iniciar qualquer plano, e põe na fila; quem barra antes, e leva até a causa, é a interface.

use super::super::caixinha::{clique_de, Clique};
use super::super::modelo::{Folha, Plano, Situacao};
use super::blocos::Bloco;
use super::decidir;
use super::figura::{feito, Marca, Peca};
use super::obra_fora as fora;
use super::obra_pecas::{pecas_de, PecaObra};

/// Os planos anteriores que nem começaram, em qualquer ação da folha. Enquanto houver algum, este não inicia.
pub fn sem_iniciar<'q>(f: &Folha, q: &'q Plano) -> Vec<&'q str> {
    q.depende_de.iter().map(String::as_str).filter(|r| fora::achar(f, r).is_some_and(|(_, d)| matches!(d.situacao, Situacao::Nao | Situacao::Revisao) && !feito(d))).collect()
}

/// Dá para iniciar agora? Se o anterior já começou, dá: o servidor põe este na fila.
pub fn pode_iniciar(f: &Folha, q: &Plano) -> bool {
    clique_de(q) == Some(Clique::Marcar) && sem_iniciar(f, q).is_empty()
}

/// Os planos que «Liberar» inicia, pelo id.
pub fn liberaveis<'a>(f: &Folha, planos: &'a [Plano]) -> Vec<&'a str> {
    planos.iter().filter(|q| pode_iniciar(f, q)).map(|q| q.id.as_str()).collect()
}

/// Alguém está neste plano: trabalhando ou parado nele.
pub fn em_obra(q: &Plano) -> bool {
    !feito(q) && matches!(q.situacao, Situacao::Trabalhando | Situacao::Parado | Situacao::Bloqueado)
}

/// O ponto em que a folha está: o primeiro plano em obra; sem nenhum, o primeiro que dá para iniciar. É ele que leva a cor de destaque.
pub fn ponto_da_folha(f: &Folha) -> Option<&Plano> {
    let planos = || f.projetos.iter().flat_map(|p| &p.acoes).flat_map(|a| &a.planos);
    planos().find(|q| em_obra(q)).or_else(|| planos().find(|q| !feito(q) && pode_iniciar(f, q)))
}

/// O que a pessoa pode fazer agora para a obra andar.
#[derive(Clone, Debug, PartialEq)]
pub enum Passo {
    /// um agente parou e precisa dela
    Parou(usize),
    /// falta decidir esta peça
    Decidir(usize, String),
    Iniciar(usize),
    EmObra(usize),
    /// a obra espera um plano de outra ação
    Espera(String),
    /// já pediu tudo: os agentes é que estão na vez
    Aguarda,
    Pronta,
}

impl Passo {
    /// A construção de que o passo fala.
    pub fn indice(&self) -> Option<usize> {
        match self {
            Passo::Parou(i) | Passo::Decidir(i, _) | Passo::Iniciar(i) | Passo::EmObra(i) => Some(*i),
            _ => None,
        }
    }
}

/// O próximo passo, nesta ordem: quem parou; depois, seguindo a escada, o que decidir ou iniciar; depois quem está trabalhando; por fim, o que vem de fora.
pub fn proximo(f: &Folha, lista: &[(&Plano, &Bloco)]) -> Passo {
    if let Some(i) = lista.iter().position(|(q, _)| matches!(q.situacao, Situacao::Parado | Situacao::Bloqueado)) {
        return Passo::Parou(i);
    }
    for (i, (q, b)) in lista.iter().enumerate().filter(|(_, (q, _))| !feito(q)) {
        if let Some(chave) = decidir::primeira_pendente(b) {
            return Passo::Decidir(i, chave);
        }
        if pode_iniciar(f, q) {
            return Passo::Iniciar(i);
        }
    }
    if let Some(i) = lista.iter().position(|(q, _)| q.situacao == Situacao::Trabalhando) {
        return Passo::EmObra(i);
    }
    if lista.iter().all(|(q, _)| feito(q)) {
        return Passo::Pronta;
    }
    let daqui = |r: &str| lista.iter().any(|(q, _)| q.referencia == r);
    let de_fora = lista.iter().filter(|(q, _)| !feito(q)).flat_map(|(q, _)| &q.depende_de).find(|r| !daqui(r) && !fora::pronto(f, r));
    de_fora.map_or(Passo::Aguarda, |r| Passo::Espera(r.clone()))
}

/// Um tipo de peça, somado: quantas pedem a pessoa, a cor que manda e para onde o clique leva.
#[derive(Clone, Debug, PartialEq)]
pub struct Casa {
    pub tipo: Peca,
    pub pede: usize,
    pub total: usize,
    pub marca: Marca,
    /// o plano da peça; vazio é da ação inteira
    pub plano: Option<String>,
    pub chave: String,
}

fn pede(p: &PecaObra) -> bool {
    matches!(p.marca, Marca::Espera | Marca::Problema) || (p.tipo == Peca::Checkpoint && p.marca == Marca::Neutra)
}

/// Uma casa por tipo que existe nestes blocos, na ordem em que as peças são numeradas.
pub fn casas<'b, 'a: 'b>(blocos: impl Iterator<Item = &'b Bloco<'a>>) -> Vec<Casa> {
    let todas: Vec<(Option<String>, PecaObra)> = blocos.flat_map(|b| {
        let dono = b.plano.as_ref().map(|q| q.referencia.clone());
        pecas_de(b).into_iter().map(move |p| (dono.clone(), p))
    }).collect();
    [Peca::Decisao, Peca::Pergunta, Peca::Tela, Peca::Checkpoint].into_iter().filter_map(|tipo| {
        let dele: Vec<&(Option<String>, PecaObra)> = todas.iter().filter(|(_, p)| p.tipo == tipo).collect();
        let primeira = dele.iter().find(|(_, p)| pede(p)).or(dele.first())?;
        let tem = |m: Marca| dele.iter().any(|(_, p)| p.marca == m);
        let marca = if tem(Marca::Problema) {
            Marca::Problema
        } else if tem(Marca::Espera) {
            Marca::Espera
        } else if tem(Marca::Neutra) {
            Marca::Neutra
        } else {
            Marca::Ok
        };
        Some(Casa { tipo, pede: dele.iter().filter(|(_, p)| pede(p)).count(), total: dele.len(), marca, plano: primeira.0.clone(), chave: primeira.1.chave.clone() })
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plano::modelo::{Acao, Pergunta, Projeto};

    fn plano(r: &str, dep: &[&str], s: Situacao) -> Plano {
        Plano { id: r.into(), referencia: r.into(), situacao: s, depende_de: dep.iter().map(|d| d.to_string()).collect(), ..Plano::default() }
    }

    fn folha(planos: Vec<Plano>) -> Folha {
        let acao = Acao { planos, ..Acao::default() };
        Folha { projetos: vec![Projeto { acoes: vec![acao], ..Projeto::default() }], ..Folha::default() }
    }

    #[test]
    fn so_inicia_depois_que_o_anterior_comecou() {
        let f = folha(vec![plano("1.1", &[], Situacao::Nao), plano("1.2", &["1.1"], Situacao::Nao)]);
        let (a, b) = (plano("1.1", &[], Situacao::Nao), plano("1.2", &["1.1"], Situacao::Nao));
        assert!(pode_iniciar(&f, &a));
        assert_eq!(sem_iniciar(&f, &b), vec!["1.1"]);
        assert!(!pode_iniciar(&f, &b));
        let f = folha(vec![plano("1.1", &[], Situacao::Trabalhando), plano("1.2", &["1.1"], Situacao::Nao)]);
        assert!(pode_iniciar(&f, &b));
    }

    #[test]
    fn o_proximo_passo_decide_antes_de_iniciar_e_quem_parou_vem_primeiro() {
        let (a, b) = (plano("1.1", &[], Situacao::Nao), plano("1.2", &["1.1"], Situacao::Nao));
        let f = folha(vec![a.clone(), b.clone()]);
        let pergunta = Pergunta { id: "p".into(), ..Pergunta::default() };
        let com_pergunta = Bloco { plano: Some(a.clone()), perguntas: vec![&pergunta], ..Bloco::default() };
        let vazio = Bloco { plano: Some(b.clone()), ..Bloco::default() };
        assert_eq!(proximo(&f, &[(&a, &com_pergunta), (&b, &vazio)]), Passo::Decidir(0, "q:p".into()));
        assert_eq!(proximo(&f, &[(&a, &vazio), (&b, &vazio)]), Passo::Iniciar(0));
        let parado = plano("1.2", &["1.1"], Situacao::Parado);
        assert_eq!(proximo(&f, &[(&a, &com_pergunta), (&parado, &vazio)]), Passo::Parou(1));
    }

    #[test]
    fn a_casa_diz_quantas_pedem_e_leva_a_primeira_pendente() {
        let (p1, p2) = (Pergunta { id: "a".into(), resposta: "sim".into(), ..Pergunta::default() }, Pergunta { id: "b".into(), ..Pergunta::default() });
        let b = Bloco { plano: Some(plano("1.1", &[], Situacao::Nao)), perguntas: vec![&p1, &p2], ..Bloco::default() };
        let c = casas(std::iter::once(&b));
        assert_eq!(c.len(), 1);
        assert_eq!((c[0].pede, c[0].total, c[0].marca, c[0].chave.as_str()), (1, 2, Marca::Espera, "q:b"));
    }
}
