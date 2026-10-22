//! O ícone do app (janela e bandeja), desenhado em código: uma folha de blueprint (papel azul, moldura clara e linhas de texto).
//! Um RGBA simples dispensa arquivo de imagem e deixa o binário pequeno.

/// A cor do papel do blueprint: é também o fundo da janela antes da página aparecer (sem flash branco).
pub const PAPEL: (u8, u8, u8, u8) = (0x0c, 0x3a, 0x78, 0xff);

/// `tam` × `tam` pixels em RGBA.
pub fn rgba(tam: u32) -> Vec<u8> {
    let (papel, tinta, trilho) = ([PAPEL.0, PAPEL.1, PAPEL.2, 0xff], [0xea, 0xf4, 0xff, 0xff], [0x5b, 0x93, 0xd0, 0xff]);
    let (margem, espessura) = (tam / 8, (tam / 16).max(1));
    let mut pixels = Vec::with_capacity((tam * tam * 4) as usize);
    for y in 0..tam {
        for x in 0..tam {
            let dentro = (margem..tam - margem).contains(&x) && (margem..tam - margem).contains(&y);
            let borda = dentro && (x < margem + espessura || x >= tam - margem - espessura || y < margem + espessura || y >= tam - margem - espessura);
            // três linhas de texto dentro da folha, a última mais curta
            let linha = [0.38, 0.56, 0.74].iter().enumerate().any(|(i, f)| {
                let ya = (tam as f32 * f) as u32;
                let fim = if i == 2 { tam * 11 / 20 } else { tam - margem * 2 - espessura };
                y >= ya && y < ya + espessura && x >= margem * 2 && x < fim
            });
            pixels.extend_from_slice(if borda { &tinta } else if linha { &trilho } else { &papel });
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_tamanho_e_as_cores_batem() {
        let p = rgba(32);
        assert_eq!(p.len(), 32 * 32 * 4);
        assert_eq!(&p[0..4], &[0x0c, 0x3a, 0x78, 0xff], "o canto é papel");
        let borda = (4 * 32 + 4) * 4;
        assert_eq!(&p[borda..borda + 4], &[0xea, 0xf4, 0xff, 0xff], "a moldura é a tinta");
        assert!(p.chunks(4).all(|px| px[3] == 0xff), "sem transparência");
    }
}
