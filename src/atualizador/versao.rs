//! Versões no formato semver (`1.2.3` e `1.2.3-beta.1`, com `v` opcional na frente): só o que o updater precisa para saber se há uma mais nova.
//! shortcut: ignora o sufixo de build (`+abc`); o projeto não o usa.
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Versao {
    numeros: [u64; 3],
    /// o que vem depois do `-` (vazio = versão final, que vale mais que qualquer pré-lançamento da mesma versão)
    pre: Vec<String>,
}

impl Versao {
    pub fn ler(texto: &str) -> Option<Versao> {
        let t = texto.trim().trim_start_matches(['v', 'V']);
        let t = t.split('+').next()?;
        let (base, pre) = t.split_once('-').map_or((t, ""), |(b, p)| (b, p));
        let mut partes = base.split('.');
        let mut numeros = [0u64; 3];
        for n in &mut numeros {
            *n = partes.next()?.parse().ok()?;
        }
        if partes.next().is_some() {
            return None;
        }
        let pre: Vec<String> = pre.split('.').filter(|p| !p.is_empty()).map(String::from).collect();
        if !t.contains('-') || !pre.is_empty() {
            Some(Versao { numeros, pre })
        } else {
            None
        }
    }
}

impl Ord for Versao {
    fn cmp(&self, outra: &Self) -> Ordering {
        self.numeros.cmp(&outra.numeros).then_with(|| match (self.pre.is_empty(), outra.pre.is_empty()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => comparar_pre(&self.pre, &outra.pre),
        })
    }
}

impl PartialOrd for Versao {
    fn partial_cmp(&self, outra: &Self) -> Option<Ordering> {
        Some(self.cmp(outra))
    }
}

/// Regra do semver: identificador numérico vale menos que o de letras; números comparam como números; o conjunto mais curto vale menos.
fn comparar_pre(a: &[String], b: &[String]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        let o = match (x.parse::<u64>(), y.parse::<u64>()) {
            (Ok(m), Ok(n)) => m.cmp(&n),
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
            (Err(_), Err(_)) => x.cmp(y),
        };
        if o != Ordering::Equal {
            return o;
        }
    }
    a.len().cmp(&b.len())
}

/// `candidata` é estritamente mais nova que `instalada`? Texto que não é versão nunca é mais novo.
pub fn mais_nova(instalada: &str, candidata: &str) -> bool {
    matches!((Versao::ler(instalada), Versao::ler(candidata)), (Some(i), Some(c)) if c > i)
}
