//! Os textos do Planejar: cada arquivo de `plano/` traz os seus (`TEXTOS`), e `i18n.rs` procura em todos.

use super::{acao, agentes, caixinha, chegou, diff, escrita, figura, folha, linha, orquestrador, pasta, perguntas, rede, revisao};

pub const TODOS: &[&[(&str, &str)]] = &[acao::TEXTOS, agentes::TEXTOS, caixinha::TEXTOS, chegou::TEXTOS, diff::TEXTOS, escrita::TEXTOS, figura::TEXTOS, folha::TEXTOS, linha::TEXTOS, linha::bloco::TEXTOS, orquestrador::TEXTOS, pasta::TEXTOS, perguntas::TEXTOS, rede::TEXTOS, revisao::TEXTOS];
