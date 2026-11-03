//! O palpite da forma de um plano que não disse a sua (`forma` vazia ou desconhecida): olha a pasta que ele toca (`area`) e o que ele diz (`texto` e `entrega`), em português e em inglês.
//! As pistas vão das mais certas às mais largas: tradução, teste, dados, integração, inteligência, infraestrutura e reorganização de código vêm antes da regra de `ui/`; senão todo plano que toca a interface vira monitor, até o que só acrescenta textos ao catálogo.

use super::Forma;

/// Uma pista: a forma, os pedaços de caminho na `area` e as palavras do texto. Palavra com `*` no fim casa pelo começo (`test*`: teste, testes, tests); sem `*`, só a palavra inteira (`ci` não casa com «cidade»).
struct Pista {
    forma: Forma,
    area: &'static [&'static str],
    texto: &'static [&'static str],
    /// se não estiver vazia, o texto também precisa de uma destas: o verbo sozinho não diz que é reorganização («mover… em arquivos»)
    junto: &'static [&'static str],
}

const fn pista(forma: Forma, area: &'static [&'static str], texto: &'static [&'static str]) -> Pista {
    Pista { forma, area, texto, junto: &[] }
}

/// Na ordem: vale a primeira que casar.
const PISTAS: [Pista; 9] = [
    pista(
        Forma::Documento,
        &["i18n", "l10n", "locale", "translation", "tradu", "catalog", "catálogo", "textos", "docs", "doc/", "document", "readme", ".md"],
        &["i18n", "l10n", "locale*", "tradu*", "translat*", "catálogo*", "catalogo*", "catalog*", "document*", "readme*"],
    ),
    pista(Forma::Teste, &["test", "e2e"], &["test*", "e2e"]),
    pista(
        Forma::Dados,
        &["dados", "banco", "migr", "sql", "/db", "schema", "database"],
        &["migração", "migrações", "migracao", "migracoes", "migration*", "sql", "sqlite", "schema*", "banco*", "database*"],
    ),
    pista(Forma::Integracao, &["mcp", "api", "http", "integr", "webhook"], &["mcp", "api", "apis", "http*", "webhook*", "endpoint*", "integração", "integracao", "integration*"]),
    pista(Forma::Ia, &["agente", "orquestr", "prompt", "/ia"], &["agent*", "prompt*", "orquestr*", "orchestr*", "llm*", "ia", "ai"]),
    pista(
        Forma::Infra,
        &["infra", "deploy", "docker", ".github", "ci/", "build", "instalador", "installer"],
        &["infra*", "deploy*", "docker*", "ci", "build", "builds", "pipeline*", "instalador*", "installer*"],
    ),
    // reorganizar código não constrói nada novo: é a construção de sempre
    pista(Forma::Padrao, &[], &["refator*", "refactor*", "reorganiz*", "reorganis*", "modulariz*"]),
    Pista {
        forma: Forma::Padrao,
        area: &[],
        texto: &["separa*", "mover", "move", "moves", "split*", "extrai*", "extract*", "divid*", "quebr*"],
        junto: &["arquivo*", "file", "files", "módulo*", "modulo*", "module*", "pasta*", "folder*", "crate*"],
    },
    pista(Forma::Tela, &["ui/", "web/", "tela", "front", "screen", "/app"], &["tela", "telas", "screen*", "página*", "pagina*", "interface*"]),
];

fn bate(padrao: &str, palavra: &str) -> bool {
    match padrao.strip_suffix('*') {
        Some(inicio) => palavra.starts_with(inicio),
        None => palavra == padrao,
    }
}

impl Pista {
    fn casa(&self, area: &str, palavras: &[&str]) -> bool {
        let no_texto = |lista: &[&str]| lista.iter().any(|p| palavras.iter().any(|w| bate(p, w)));
        let achou = self.area.iter().any(|t| area.contains(t)) || no_texto(self.texto);
        achou && (self.junto.is_empty() || no_texto(self.junto))
    }
}

/// A forma pelo que o plano toca e diz. Sem pista nenhuma: o serviço, se há uma pasta; a construção de sempre, se nem isso.
pub fn palpite(area: &str, texto: &str, entrega: &str) -> Forma {
    let area = area.to_lowercase();
    let frase = format!("{texto} {entrega}").to_lowercase();
    let palavras: Vec<&str> = frase.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    let resto = if area.is_empty() { Forma::Padrao } else { Forma::Servico };
    PISTAS.iter().find(|p| p.casa(&area, &palavras)).map_or(resto, |p| p.forma)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plano::modelo::Plano;

    fn plano(area: &str, texto: &str) -> Plano {
        Plano { area: area.into(), texto: texto.into(), ..Plano::default() }
    }

    #[test]
    fn textos_novos_no_catalogo_sao_documento_e_nao_tela() {
        let q = plano("ui/assets/i18n-en", "Acrescentar ao catálogo em inglês os textos novos da tela");
        assert_eq!(Forma::do_plano(&q), Forma::Documento);
        assert_eq!(palpite("", &q.texto, ""), Forma::Documento);
    }

    #[test]
    fn separar_em_arquivos_e_a_construcao_de_sempre() {
        let q = plano("ui/src/folha/lista", "Separar a lista de blueprints da folha antiga em arquivos próprios");
        assert_eq!(Forma::do_plano(&q), Forma::Padrao);
    }

    #[test]
    fn o_verbo_sozinho_nao_faz_reorganizacao() {
        assert_eq!(Forma::do_plano(&plano("ui/src/plano", "Mover o botão para o topo")), Forma::Tela);
    }

    #[test]
    fn a_forma_dita_ganha_do_palpite() {
        let q = Plano { forma: "tela".into(), ..plano("ui/assets/i18n-en", "Acrescentar ao catálogo") };
        assert_eq!(Forma::do_plano(&q), Forma::Tela);
    }

    #[test]
    fn o_texto_basta_sem_area() {
        assert_eq!(palpite("", "Testes e2e do login", ""), Forma::Teste);
        assert_eq!(palpite("", "Criar a migração da tabela de convites", ""), Forma::Dados);
        assert_eq!(palpite("", "", "O MCP aceita a ação nova"), Forma::Integracao);
        assert_eq!(palpite("", "Publicar com Docker", ""), Forma::Infra);
        assert_eq!(palpite("", "Tela de entrada", ""), Forma::Tela);
        assert_eq!(palpite("", "Algo sem pista", ""), Forma::Padrao);
    }

    #[test]
    fn palavra_curta_so_casa_inteira() {
        assert_eq!(palpite("src/cidade", "Mostrar a cidade", ""), Forma::Servico);
        assert_eq!(palpite("src/x", "Rodar no CI", ""), Forma::Infra);
    }
}
