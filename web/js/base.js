const ROTULO = { concluido: tr("concluído"), andamento: tr("em desenvolvimento"), planejado: tr("planejado") };
const BARRA_MIN = 220; // largura mínima da barra da ação quando aberta (o degrau da escada deixaria blueprints longos com barras curtas)
const CHAR = 6.9; // largura de um caractere de 11 px no tipo mono

const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
const norm = (s) => String(s ?? "").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
const novoId = () => Math.random().toString(36).slice(2, 9);
// só http e https viram link: um endereço digitado nunca vira javascript: ou data:
const urlSegura = (u) => {
  const t = String(u ?? "").trim();
  if (!t) return null;
  try { const x = new URL(/^[a-z][a-z0-9+.-]*:/i.test(t) ? t : "https://" + t); return x.protocol === "http:" || x.protocol === "https:" ? x.href : null; } catch { return null; }
};
const rotuloDe = (l) => l.rotulo || new URL(urlSegura(l.url)).hostname.replace(/^www\./, "");
const pad = (n) => String(n).padStart(2, "0");
const corta = (s, max) => (s.length > max ? s.slice(0, Math.max(1, max - 1)) + "…" : s);
const umaLinha = (s, max) => corta(String(s ?? "").replace(/\s+/g, " ").trim(), max);
const reduz = () => matchMedia("(prefers-reduced-motion: reduce)").matches || document.hidden; // sem animação com movimento reduzido ou com a aba/janela oculta

const $ = (id) => document.getElementById(id);
