//! O Planejar: o formato `blueprint.plano/1` e as regras puras sobre ele (sem banco, sem HTTP, só `serde_json` e `sha2`). Quem grava é `dados::planejar`; quem atende é `mcp::planejar`.
pub mod arvore;
pub mod comparar;
pub mod entrada;
pub mod escalonar;
pub mod estilo;
pub mod formato;
mod formato_ler;
pub mod frases;
pub mod ids;
pub mod pacote;
pub mod svg;
pub mod validar;

#[cfg(test)]
mod arvore_tests;
#[cfg(test)]
mod comparar_tests;
#[cfg(test)]
mod entrada_tests;
#[cfg(test)]
mod escalonar_tests;
#[cfg(test)]
mod formato_tests;
#[cfg(test)]
mod ids_tests;
#[cfg(test)]
mod svg_tests;
#[cfg(test)]
mod validar_tests;
