//! Vetores 3D como `[x, y, z]`, com o eixo `z` para cima, como no kit.

pub type V3 = [f32; 3];

pub fn soma(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn vezes(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cruz(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

pub fn unit(a: V3) -> V3 {
    let n = dot(a, a).sqrt();
    if n > 0.0 { vezes(a, 1.0 / n) } else { a }
}

pub fn mistura(a: V3, b: V3, t: f32) -> V3 {
    soma(a, vezes(sub(b, a), t))
}
