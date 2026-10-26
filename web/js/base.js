const MESES = tr("JAN|FEV|MAR|ABR|MAI|JUN|JUL|AGO|SET|OUT|NOV|DEZ").split("|");
const ROTULO = { concluido: tr("concluído"), andamento: tr("em desenvolvimento"), planejado: tr("planejado") };
const BARRA_MIN = 220; // largura mínima da barra da ação quando aberta (a escala real da régua deixaria ações curtas com ~100 px)
const CHAR = 6.9; // largura de um caractere de 11 px no tipo mono

const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
const norm = (s) => String(s ?? "").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
const ISO = /^\d{4}-\d{2}-\d{2}$/;
const dia = (iso) => (ISO.test(iso) ? Date.parse(iso + "T00:00:00Z") / 864e5 : NaN);
const deDia = (d) => new Date(d * 864e5).toISOString().slice(0, 10);
const fmt = (iso) => iso.slice(8, 10) + "/" + iso.slice(5, 7);
const local = (d) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
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
