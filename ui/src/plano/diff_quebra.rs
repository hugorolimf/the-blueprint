//! A quebra de linha do diff: põe os pedaços em linhas de uma largura e remapeia o destaque para cada linha.

/// Uma linha já quebrada: `trechos` destacados, em letras de `texto`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fileira {
    pub texto: String,
    pub trechos: Vec<(usize, usize)>,
}

#[derive(Default)]
struct Montagem {
    prontas: Vec<Fileira>,
    atual: Fileira,
    letras: usize,
    usado: f32,
    /// o destaque anterior continua: o que veio depois dele também é destaque
    ponte: bool,
}

impl Montagem {
    fn fechar(&mut self) {
        let fim = self.atual.texto.trim_end().len();
        self.atual.texto.truncate(fim);
        self.prontas.push(std::mem::take(&mut self.atual));
        (self.letras, self.usado, self.ponte) = (0, 0.0, false);
    }

    fn palavra(&mut self, s: &str, w: f32, forte: bool) {
        let de = self.letras;
        self.letras += s.chars().count();
        self.atual.texto.push_str(s);
        self.usado += w;
        if forte {
            match self.atual.trechos.last_mut() {
                Some(ultimo) if self.ponte => ultimo.1 = self.letras,
                _ => self.atual.trechos.push((de, self.letras)),
            }
        }
        self.ponte = forte;
    }

    /// O espaço no começo da linha some, e o destaque só o cobre entre duas palavras destacadas, como no protótipo.
    fn espaco(&mut self, s: &str, w: f32, forte: bool) {
        if self.atual.texto.is_empty() {
            return;
        }
        self.letras += s.chars().count();
        self.atual.texto.push_str(s);
        self.usado += w;
        self.ponte &= forte;
    }
}

/// Parte em espaços e palavras, sem perder nada.
fn pedras(s: &str) -> impl Iterator<Item = &str> {
    let mut resto = s;
    std::iter::from_fn(move || {
        let primeira = resto.chars().next()?;
        let fim = resto.find(|c: char| c.is_whitespace() != primeira.is_whitespace()).unwrap_or(resto.len());
        let (pedra, depois) = resto.split_at(fim);
        resto = depois;
        Some(pedra)
    })
}

/// Quebra os pedaços `(texto, destacado)` em linhas de até `largura`, remapeando o destaque para cada linha. Palavra maior que a linha é cortada por letra.
pub fn quebrar(pedacos: &[(String, bool)], largura: f32, medir: &dyn Fn(&str) -> f32) -> Vec<Fileira> {
    let mut m = Montagem::default();
    for (texto, forte) in pedacos {
        for pedra in pedras(texto) {
            let w = medir(pedra);
            if pedra.starts_with(char::is_whitespace) {
                m.espaco(pedra, w, *forte);
                continue;
            }
            if m.usado + w > largura && !m.atual.texto.is_empty() {
                m.fechar();
            }
            if w <= largura {
                m.palavra(pedra, w, *forte);
                continue;
            }
            for (i, c) in pedra.char_indices() {
                let letra = &pedra[i..i + c.len_utf8()];
                let wl = medir(letra);
                if m.usado + wl > largura && !m.atual.texto.is_empty() {
                    m.fechar();
                }
                m.palavra(letra, wl, *forte);
            }
        }
    }
    if !m.atual.texto.is_empty() || m.prontas.is_empty() {
        m.fechar();
    }
    m.prontas
}
