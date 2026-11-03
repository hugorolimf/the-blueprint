//! A cena: junta sólidos, fios e pontos que quem chama desenha (a figura de um agente, um rótulo). Desenha de trás para a frente e responde qual peça está sob o mouse.
//! Cada coisa tem uma `Camada` (o chão vem antes do que está em cima dele) e, dentro dela, a profundidade decide. É o algoritmo do pintor: simples e bom para peças separadas.

use super::camera::Camera;
use super::malha::Malha;
use super::tinta::Tinta;
use super::vetor::V3;
use egui::{Painter, Pos2, Rect, Shape, Stroke};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Camada {
    Base,
    Chao,
    Dutos,
    DutosTopo,
    Pecas,
    Acima,
}

enum Coisa {
    Solido(Malha, Tinta),
    Fio { pontos: Vec<V3>, traco: Stroke, tracos: Option<(f32, f32, f32)> },
    /// o número é devolvido ao `extra` de `desenhar`, na vez dele
    Extra(usize),
}

struct Item {
    coisa: Coisa,
    camada: Camada,
    centro: V3,
    /// o grupo vem antes da camada: quem está em cima de outra coisa, como as bandejas de um desenho explodido, pinta depois dela inteira
    grupo: i32,
    /// a chave do clique; zero: não responde
    chave: u32,
    /// um raio em pixels em volta de `centro` que também responde: peça fina fica fácil de acertar
    folga: f32,
}

/// Uma vez do desenho da cena: uma forma pronta, ou o lugar do `extra` de número tal.
pub enum Gravado {
    Forma(Shape),
    Extra(usize),
}

#[derive(Default)]
pub struct Cena {
    itens: Vec<Item>,
    grupo: i32,
}

impl Cena {
    /// O que entrar daqui em diante pinta neste grupo, depois de todo o grupo de número menor.
    pub fn grupo(&mut self, n: i32) {
        self.grupo = n;
    }

    pub fn solido(&mut self, camada: Camada, malha: Malha, tinta: Tinta, chave: u32) {
        let centro = malha.centro();
        self.itens.push(Item { coisa: Coisa::Solido(malha, tinta), camada, centro, grupo: self.grupo, chave, folga: 0.0 });
    }

    /// Um sólido com o clique mais fácil de acertar: `folga` pixels em volta do meio.
    pub fn solido_folgado(&mut self, camada: Camada, malha: Malha, tinta: Tinta, chave: u32, folga: f32) {
        let centro = malha.centro();
        self.itens.push(Item { coisa: Coisa::Solido(malha, tinta), camada, centro, grupo: self.grupo, chave, folga });
    }

    /// Um fio. `tracos` é `(traço, vão, deslocamento)`, para ele andar.
    pub fn fio(&mut self, camada: Camada, pontos: Vec<V3>, traco: Stroke, tracos: Option<(f32, f32, f32)>) {
        let meio = pontos.get(pontos.len() / 2).copied().unwrap_or([0.0; 3]);
        self.itens.push(Item { coisa: Coisa::Fio { pontos, traco, tracos }, camada, centro: meio, grupo: self.grupo, chave: 0, folga: 0.0 });
    }

    /// Um lugar na ordem de desenho para o que só quem chama sabe pintar.
    pub fn extra(&mut self, camada: Camada, centro: V3, numero: usize, chave: u32, folga: f32) {
        self.itens.push(Item { coisa: Coisa::Extra(numero), camada, centro, grupo: self.grupo, chave, folga });
    }

    fn ordem(&self, c: &Camera) -> Vec<usize> {
        // a profundidade de cada item uma vez só, e não duas a cada comparação: o `sort_by` estável deixa a mesma ordem
        let fundo: Vec<f32> = self.itens.iter().map(|x| c.profundidade(x.centro)).collect();
        let mut v: Vec<usize> = (0..self.itens.len()).collect();
        v.sort_by(|a, b| {
            let (x, y) = (&self.itens[*a], &self.itens[*b]);
            x.grupo.cmp(&y.grupo).then(x.camada.cmp(&y.camada)).then_with(|| fundo[*a].total_cmp(&fundo[*b]))
        });
        v
    }

    /// De trás para a frente: cada forma pronta e, na vez dele, cada `extra`. O sólido todo fora de `recorte` não entrega nada.
    fn percorrer(&self, c: &Camera, recorte: Rect, por: &mut impl FnMut(Gravado)) {
        for i in self.ordem(c) {
            let item = &self.itens[i];
            match &item.coisa {
                Coisa::Solido(m, t) => m.formas_em(c, t, recorte, &mut |s| por(Gravado::Forma(s))),
                Coisa::Fio { pontos, traco, tracos } => {
                    if pontos.iter().all(|q| c.alcanca(*q)) {
                        let tela: Vec<Pos2> = pontos.iter().map(|q| c.ponto(*q)).collect();
                        match tracos {
                            Some((t, v, d)) => {
                                for s in Shape::dashed_line_with_offset(&tela, *traco, &[*t], &[*v], *d) {
                                    por(Gravado::Forma(s));
                                }
                            }
                            None => por(Gravado::Forma(Shape::line(tela, *traco))),
                        }
                    }
                }
                Coisa::Extra(n) => por(Gravado::Extra(*n)),
            }
        }
    }

    pub fn desenhar(&self, c: &Camera, p: &Painter, mut extra: impl FnMut(usize)) {
        self.percorrer(c, p.clip_rect(), &mut |g| match g {
            Gravado::Forma(s) => {
                p.add(s);
            }
            Gravado::Extra(n) => extra(n),
        });
    }

    /// O que `desenhar` pintaria com esta câmera, na mesma ordem, sem pintar: a cena parada (a miniatura da folha) guarda isto e só repinta.
    pub fn gravar(&self, c: &Camera) -> Vec<Gravado> {
        let mut v = Vec::new();
        // guardada para outros quadros, que podem ter outro recorte: entra tudo
        self.percorrer(c, Rect::EVERYTHING, &mut |g| v.push(g));
        v
    }

    /// A chave do que está na frente em `pos`; zero para nada.
    pub fn tocar(&self, c: &Camera, pos: Pos2) -> u32 {
        for i in self.ordem(c).into_iter().rev() {
            let item = &self.itens[i];
            if item.chave == 0 {
                continue;
            }
            let na_folga = item.folga > 0.0 && c.alcanca(item.centro) && c.ponto(item.centro).distance(pos) <= item.folga;
            let no_solido = matches!(&item.coisa, Coisa::Solido(m, _) if m.contem(c, pos));
            if na_folga || no_solido {
                return item.chave;
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::super::camera::{Camera, AZIMUTE, ELEVACAO};
    use super::*;
    use egui::pos2;

    #[test]
    fn o_clique_pega_a_peca_da_frente() {
        let c = Camera::nova(AZIMUTE, ELEVACAO, 20.0, [0.0; 3], pos2(300.0, 300.0), 0.0);
        let mut cena = Cena::default();
        cena.solido(Camada::Pecas, Malha::caixa([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]), Tinta::parede(), 1);
        cena.solido(Camada::Pecas, Malha::caixa([1.0, 1.0, 0.0], [2.0, 2.0, 2.0]), Tinta::parede(), 2);
        assert_eq!(cena.tocar(&c, c.ponto([2.0, 2.0, 2.0])), 2);
        assert_eq!(cena.tocar(&c, pos2(0.0, 0.0)), 0);
    }

    #[test]
    fn o_grupo_de_cima_ganha_de_qualquer_camada() {
        let c = Camera::nova(AZIMUTE, ELEVACAO, 20.0, [0.0; 3], pos2(300.0, 300.0), 0.0);
        let mut cena = Cena::default();
        cena.grupo(1);
        cena.solido(Camada::Chao, Malha::caixa([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]), Tinta::chao(), 1);
        cena.grupo(0);
        cena.solido(Camada::Acima, Malha::caixa([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]), Tinta::chao(), 2);
        assert_eq!(cena.tocar(&c, c.ponto([1.0, 1.0, 2.0])), 1);
    }

    #[test]
    fn a_camada_de_cima_ganha_mesmo_mais_longe() {
        let c = Camera::nova(AZIMUTE, ELEVACAO, 20.0, [0.0; 3], pos2(300.0, 300.0), 0.0);
        let mut cena = Cena::default();
        cena.solido(Camada::Chao, Malha::caixa([0.0, 0.0, 0.0], [4.0, 4.0, 1.0]), Tinta::chao(), 1);
        cena.solido(Camada::Pecas, Malha::caixa([0.0, 0.0, 1.0], [1.0, 1.0, 1.0]), Tinta::parede(), 2);
        assert_eq!(cena.tocar(&c, c.ponto([0.5, 0.5, 2.0])), 2);
    }
}
