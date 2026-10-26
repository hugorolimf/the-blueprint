// ---- a tela Roadmap abre sob demanda (módulo ES roadmap.js): pelo botão da entrada e pela rota #/roadmap ----
let roadmapModulo = null;
function roadmapAbre(origem) {
  (roadmapModulo ||= import("/web/js/roadmap.js")).then((m) => m.abrirRoadmap(origem)).catch(() => { roadmapModulo = null; aviso(tr("Não foi possível abrir o roadmap.")); });
}
$("b-roadmap")?.addEventListener("click", (e) => roadmapAbre(e.currentTarget));
addEventListener("hashchange", () => { if (location.hash === "#/roadmap") roadmapAbre(null); });
if (location.hash === "#/roadmap") roadmapAbre(null);
