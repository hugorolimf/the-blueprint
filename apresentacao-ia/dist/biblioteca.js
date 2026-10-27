// ARQUIVO GERADO por apresentacao-ia/empacotar.mjs. Não edite: mude kit/ ou biblioteca/*.js e rode `node apresentacao-ia/empacotar.mjs`.
// Por ser gerado, fica fora do limite de 350 linhas (teto 500): o limite vale para as fontes, que são kit/**/*.mjs e biblioteca/*.js (cada uma até 350).
// `node apresentacao-ia/empacotar.mjs --conferir` e empacotar.test.mjs falham se este arquivo estiver diferente do que o empacotador geraria agora.
const M0 = (() => { // iso-kit/interno.mjs
// Auxiliares compartilhados entre as partes do kit (não são reexportados pelo índice iso-kit.mjs).
const RADIANS = Math.PI / 180;
const tenth = (value) => {
    const text = (Math.round(value * 10) / 10).toFixed(1).replace(/\.0$/, "");
    return text === "-0" ? "0" : text;
};
function signedArea(points) {
    let area = 0;
    for (let index = 0; index < points.length; index++) {
        const [ax, ay] = points[index];
        const [bx, by] = points[(index + 1) % points.length];
        area += ax * by - bx * ay;
    }
    return area / 2;
}
const oriented = (points) => (signedArea(points) < 0 ? [...points].reverse() : points);
function runsOf(flags, want) {
    const count = flags.length;
    const runs = [];
    const start = flags.findIndex((flag, index) => flag !== flags[(index + count - 1) % count]);
    if (start < 0)
        return flags[0] === want ? [Array.from({ length: count + 1 }, (_, index) => index % count)] : [];
    let run = null;
    for (let step = 0; step < count; step++) {
        const index = (start + step) % count;
        if (flags[index] === want) {
            if (!run)
                run = [index];
            run.push((index + 1) % count);
        }
        else if (run) {
            runs.push(run);
            run = null;
        }
    }
    if (run)
        runs.push(run);
    return runs;
}
const escape = (text) => text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const attrs = (pairs) => Object.entries(pairs)
    .filter(([, value]) => value !== undefined && value !== false)
    .map(([key, value]) => ` ${key}="${escape(String(value))}"`)
    .join("");

return { RADIANS, tenth, signedArea, oriented, runsOf, escape, attrs };
})();
const M1 = (() => { // iso-kit/camera.mjs
const { RADIANS } = M0;
// Câmera isométrica: ângulos, projeção 3D para 2D, profundidade e enquadramento.

const AZIMUTH = 45;
const ELEVATION = 30;
const SHADES = 4;
const UNIT = Math.sqrt(1.6);
function cameraOf({ scale, azimuth = AZIMUTH, elevation = ELEVATION }) {
    const a = azimuth * RADIANS;
    const e = elevation * RADIANS;
    return { sinA: Math.sin(a), cosA: Math.cos(a), sinE: Math.sin(e), cosE: Math.cos(e), k: scale * UNIT };
}
function iso([x, y, z], projection) {
    const { origin } = projection;
    const { sinA, cosA, sinE, cosE, k } = cameraOf(projection);
    return [origin[0] + (x * sinA - y * cosA) * k, origin[1] + ((x * cosA + y * sinA) * sinE - z * cosE) * k];
}
function towardViewer(projection) {
    const { sinA, cosA } = cameraOf(projection);
    return [cosA, sinA];
}
function depthOf([x, y, z], projection) {
    const { sinA, cosA, sinE, cosE } = cameraOf(projection);
    return (x * cosA + y * sinA) * cosE + z * sinE;
}
function screenRise(height, projection) {
    const { cosE, k } = cameraOf(projection);
    return height * cosE * k;
}
function fitProjection(points, width, height, { pad = 24, headroom = 0, azimuth = AZIMUTH, elevation = ELEVATION } = {}) {
    const unit = { origin: [0, 0], scale: 1, azimuth, elevation };
    const flat = points.map((point) => iso(point, unit));
    const xs = flat.map(([x]) => x);
    const ys = flat.map(([, y]) => y);
    const left = Math.min(...xs);
    const right = Math.max(...xs);
    const up = Math.min(...ys);
    const down = Math.max(...ys);
    const scale = Math.min((width - pad * 2) / (right - left), (height - pad * 2 - headroom) / (down - up));
    return {
        origin: [width / 2 - ((left + right) / 2) * scale, headroom + (height - headroom) / 2 - ((up + down) / 2) * scale],
        scale,
        azimuth,
        elevation,
    };
}

return { AZIMUTH, ELEVATION, SHADES, cameraOf, iso, towardViewer, depthOf, screenRise, fitProjection };
})();
const M2 = (() => { // iso-kit/caminho.mjs
const { tenth } = M0;
// Caminhos SVG: pontos em texto de path.

function pathOf(points, closed = false) {
    if (!points.length)
        return "";
    return points.map(([x, y], at) => `${at ? "L" : "M"}${tenth(x)} ${tenth(y)}`).join("") + (closed ? "Z" : "");
}
const joinPaths = (paths) => paths.filter(Boolean).join("");

return { pathOf, joinPaths };
})();
const M3 = (() => { // iso-kit/plano.mjs
const { RADIANS } = M0;
// Plantas e anéis no plano (retângulo arredondado, círculo, recuo, cantos).

function roundedPlan({ x, y, w, d, r }, steps = 8) {
    const radius = Math.max(0, Math.min(r, w / 2, d / 2));
    const corners = [
        [x + w - radius, y + radius, -90],
        [x + w - radius, y + d - radius, 0],
        [x + radius, y + d - radius, 90],
        [x + radius, y + radius, 180],
    ];
    const out = [];
    for (const [cx, cy, start] of corners) {
        const count = radius > 0 ? steps : 0;
        for (let step = 0; step <= count; step++) {
            const angle = (start + (count ? (90 * step) / count : 45)) * RADIANS;
            out.push([cx + radius * Math.cos(angle), cy + radius * Math.sin(angle)]);
        }
    }
    return out;
}
const ringOf = (plan, steps = 8) => roundedPlan(plan, steps);
function circleRing(cx, cy, radius, steps = 40) {
    return Array.from({ length: steps }, (_, step) => {
        const angle = (step / steps) * Math.PI * 2;
        return [cx + radius * Math.cos(angle), cy + radius * Math.sin(angle)];
    });
}
const insetPlan = ({ x, y, w, d, r }, by) => ({ x: x + by, y: y + by, w: w - by * 2, d: d - by * 2, r: Math.max(0, r - by) });
function corners({ x, y, w, d }, inset = 0) {
    return [
        [x + inset, y + inset],
        [x + w - inset, y + inset],
        [x + inset, y + d - inset],
        [x + w - inset, y + d - inset],
    ];
}
function boxCorners({ x, y, w, d }, z0, z1) {
    const out = [];
    for (const px of [x, x + w])
        for (const py of [y, y + d])
            for (const z of [z0, z1])
                out.push([px, py, z]);
    return out;
}

return { roundedPlan, ringOf, circleRing, insetPlan, corners, boxCorners };
})();
const M4 = (() => { // iso-kit/extrusao.mjs
const { SHADES, cameraOf, iso, towardViewer } = M1;
const { pathOf } = M2;
const { circleRing, ringOf } = M3;
const { oriented, runsOf, signedArea } = M0;
// Extrusão de anéis em sólidos (lajes, cilindros), com faces sombreadas, vincos e chanfro.




function convexHull(points) {
    const sorted = [...points].sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    if (sorted.length < 3)
        return sorted;
    const cross = (o, a, b) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
    const lower = [];
    for (const point of sorted) {
        while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], point) <= 0)
            lower.pop();
        lower.push(point);
    }
    const upper = [];
    for (let index = sorted.length - 1; index >= 0; index--) {
        const point = sorted[index];
        while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], point) <= 0)
            upper.pop();
        upper.push(point);
    }
    return [...lower.slice(0, -1), ...upper.slice(0, -1)];
}
function extrude(ring, z, height, projection, options = {}) {
    const convex = options.convex ?? true;
    const count = ring.length;
    const counter = signedArea(ring) > 0;
    const [vx, vy] = towardViewer(projection);
    const { sinA, cosA } = cameraOf(projection);
    const normals = ring.map(([ax, ay], index) => {
        const [bx, by] = ring[(index + 1) % count];
        const dx = bx - ax;
        const dy = by - ay;
        const length = Math.hypot(dx, dy) || 1;
        return (counter ? [dy / length, -dx / length] : [-dy / length, dx / length]);
    });
    const facing = normals.map(([nx, ny]) => nx * vx + ny * vy > 1e-9);
    const top = ring.map(([x, y]) => iso([x, y, z + height], projection));
    const bottom = ring.map(([x, y]) => iso([x, y, z], projection));
    const outline = [];
    const crease = [];
    for (const run of runsOf(facing, false))
        outline.push(pathOf(run.map((index) => top[index])));
    for (const run of runsOf(facing, true)) {
        crease.push(pathOf(run.map((index) => top[index])));
        if (height > 0)
            outline.push(pathOf(run.map((index) => bottom[index])));
    }
    if (height > 0) {
        for (let index = 0; index < count; index++) {
            if (facing[(index + count - 1) % count] !== facing[index])
                outline.push(pathOf([top[index], bottom[index]]));
        }
    }
    let fill;
    if (convex) {
        fill = pathOf(convexHull([...top, ...bottom]), true);
    }
    else {
        const parts = [pathOf(oriented(top), true), pathOf(oriented(bottom), true)];
        for (let index = 0; index < count; index++) {
            if (!facing[index])
                continue;
            const next = (index + 1) % count;
            parts.push(pathOf(oriented([top[index], top[next], bottom[next], bottom[index]]), true));
        }
        fill = parts.join("");
    }
    const buckets = Array.from({ length: SHADES }, () => []);
    if (height > 0) {
        const [lx, ly] = [-sinA, cosA];
        for (let index = 0; index < count; index++) {
            if (!facing[index])
                continue;
            const [nx, ny] = normals[index];
            const lit = (nx * lx + ny * ly + 1) / 2;
            const shade = Math.min(SHADES - 1, Math.max(0, Math.floor(lit * SHADES)));
            const next = (index + 1) % count;
            buckets[shade].push(pathOf(oriented([top[index], top[next], bottom[next], bottom[index]]), true));
        }
    }
    let bevel;
    if (options.bevel && options.bevel > 0) {
        const by = options.bevel;
        const inset = ring.map(([x, y], index) => {
            const [ax, ay] = normals[(index + count - 1) % count];
            const [bx, by2] = normals[index];
            const mx = ax + bx;
            const my = ay + by2;
            const length = Math.hypot(mx, my) || 1;
            return [x - (mx / length) * by, y - (my / length) * by];
        });
        const insetTop = inset.map(([x, y]) => iso([x, y, z + height], projection));
        bevel = runsOf(facing, true)
            .map((run) => pathOf([top[run[0]], ...run.slice(1, -1).map((index) => insetTop[index]), top[run[run.length - 1]]]))
            .join("");
    }
    return {
        fill,
        outline: outline.join(""),
        crease: crease.join(""),
        top: pathOf(oriented(top), true),
        shades: buckets.map((paths) => paths.join("")),
        bevel,
    };
}
function slabOf(plan, z, height, projection, steps = 8, bevel = 0) {
    return extrude(ringOf(plan, steps), z, height, projection, { convex: true, bevel });
}
function cylinder(cx, cy, radius, z, height, projection, steps = 32, bevel = 0) {
    return extrude(circleRing(cx, cy, radius, steps), z, height, projection, { convex: true, bevel });
}
function ringSeam(ring, z, projection) {
    const count = ring.length;
    const counter = signedArea(ring) > 0;
    const [vx, vy] = towardViewer(projection);
    const facing = ring.map(([ax, ay], index) => {
        const [bx, by] = ring[(index + 1) % count];
        const dx = bx - ax;
        const dy = by - ay;
        const [nx, ny] = counter ? [dy, -dx] : [-dy, dx];
        return nx * vx + ny * vy > 1e-9;
    });
    return runsOf(facing, true).map((run) => pathOf(run.map((index) => iso([ring[index][0], ring[index][1], z], projection)))).join("");
}
const sideSeam = (plan, z, projection, steps = 8) => ringSeam(ringOf(plan, steps), z, projection);
const haloOf = (plan, z, height, projection) => slabOf(plan, z, height, projection).fill;

return { extrude, slabOf, cylinder, ringSeam, sideSeam, haloOf };
})();
const M5 = (() => { // iso-kit/linhas.mjs
const { depthOf, iso, towardViewer } = M1;
const { pathOf } = M2;
const { circleRing, ringOf } = M3;
// Linhas, pontos, grades e curvas projetadas sobre o plano ou as laterais.



function onTop(points, z, projection, closed = false) {
    return pathOf(points.map(([x, y]) => iso([x, y, z], projection)), closed);
}
const planOutline = (plan, z, projection, steps = 8) => onTop(ringOf(plan, steps), z, projection, true);
const lineOnTop = (from, to, z, projection) => onTop([from, to], z, projection);
const segment = (from, to, projection) => pathOf([iso(from, projection), iso(to, projection)]);
const rise = (at, from, to, projection) => segment([at[0], at[1], from], [at[0], at[1], to], projection);
const ring = (cx, cy, radius, z, projection, steps = 40) => onTop(circleRing(cx, cy, radius, steps), z, projection, true);
const pointsAt = (points, projection) => points.map((point) => iso(point, projection));
function dotGrid(x, y, columns, rows, gap, z, projection) {
    const dots = [];
    for (let row = 0; row < rows; row++) {
        for (let column = 0; column < columns; column++)
            dots.push(iso([x + column * gap, y + row * gap, z], projection));
    }
    return dots;
}
function knurl(cx, cy, radius, z0, z1, count, projection, inset = 0.4) {
    const [vx, vy] = towardViewer(projection);
    const lines = [];
    for (let index = 0; index < count; index++) {
        const angle = (index / count) * Math.PI * 2;
        const nx = Math.cos(angle);
        const ny = Math.sin(angle);
        if (nx * vx + ny * vy < 0.15)
            continue;
        lines.push(segment([cx + radius * nx, cy + radius * ny, z0 + inset], [cx + radius * nx, cy + radius * ny, z1 - inset], projection));
    }
    return lines.join("");
}
function sideArc(cx, cy, radius, z, projection, steps = 24) {
    const [vx, vy] = towardViewer(projection);
    const facing = Math.atan2(vy, vx);
    const points = [];
    for (let step = 0; step <= steps; step++) {
        const angle = facing - Math.PI / 2 + (step / steps) * Math.PI;
        points.push(iso([cx + radius * Math.cos(angle), cy + radius * Math.sin(angle), z], projection));
    }
    return pathOf(points);
}
function sideRing(center, radius, axis, projection, steps = 20) {
    const [px, py, pz] = center;
    const points = [];
    for (let step = 0; step < steps; step++) {
        const angle = (step / steps) * Math.PI * 2;
        const u = radius * Math.cos(angle);
        const v = radius * Math.sin(angle);
        points.push(iso(axis === "x" ? [px, py + u, pz + v] : [px + u, py, pz + v], projection));
    }
    return pathOf(points, true);
}
function coil(from, to, center, radius, turns, axis, projection, perTurn = 16) {
    const count = Math.max(1, Math.round(turns * perTurn));
    const back = [];
    const front = [];
    let run = [];
    let near = null;
    for (let index = 0; index <= count; index++) {
        const share = index / count;
        const angle = share * turns * Math.PI * 2;
        const u = radius * Math.cos(angle);
        const v = radius * Math.sin(angle);
        const along = from + (to - from) * share;
        const point = axis === "x" ? [along, center[0] + u, center[1] + v] : [center[0] + u, along, center[1] + v];
        const normal = axis === "x" ? [0, Math.cos(angle), Math.sin(angle)] : [Math.cos(angle), 0, Math.sin(angle)];
        const facing = depthOf(normal, projection) > 0;
        const flat = iso(point, projection);
        if (near === null)
            near = facing;
        if (facing !== near) {
            run.push(flat);
            (near ? front : back).push(pathOf(run));
            run = [flat];
            near = facing;
            continue;
        }
        run.push(flat);
    }
    if (run.length > 1)
        (near ? front : back).push(pathOf(run));
    return { back: back.join(""), front: front.join("") };
}

return { onTop, planOutline, lineOnTop, segment, rise, ring, pointsAt, dotGrid, knurl, sideArc, sideRing, coil };
})();
const M6 = (() => { // iso-kit/reguas.mjs
const { lineOnTop, segment } = M5;
// Marcas de régua (ticks) no topo, na lateral e radiais.

function topTicks(from, to, step, every, edge, z, lengths, projection, axis = "x") {
    const minor = [];
    const major = [];
    const first = Math.ceil(from / step) * step;
    for (let at = first, index = Math.round(first / step); at <= to + 1e-9; at += step, index++) {
        const big = index % every === 0;
        const length = big ? lengths[1] : lengths[0];
        const a = axis === "x" ? [at, edge] : [edge, at];
        const b = axis === "x" ? [at, edge + length] : [edge + length, at];
        (big ? major : minor).push(lineOnTop(a, b, z, projection));
    }
    return { minor: minor.join(""), major: major.join("") };
}
function sideTicks(from, to, step, every, face, top, lengths, projection, axis = "x") {
    const minor = [];
    const major = [];
    const first = Math.ceil(from / step) * step;
    for (let at = first, index = Math.round(first / step); at <= to + 1e-9; at += step, index++) {
        const big = index % every === 0;
        const length = big ? lengths[1] : lengths[0];
        const a = axis === "x" ? [at, face, top] : [face, at, top];
        const b = axis === "x" ? [at, face, top - length] : [face, at, top - length];
        (big ? major : minor).push(segment(a, b, projection));
    }
    return { minor: minor.join(""), major: major.join("") };
}
function radialTicks(cx, cy, radius, count, every, z, lengths, projection, from = 0, span = Math.PI * 2) {
    const minor = [];
    const major = [];
    const closed = Math.abs(span - Math.PI * 2) < 1e-9;
    const steps = closed ? count : count + 1;
    for (let index = 0; index < steps; index++) {
        const angle = from + (index / count) * span;
        const big = index % every === 0;
        const inner = radius - (big ? lengths[1] : lengths[0]);
        const sx = Math.sin(angle);
        const sy = -Math.cos(angle);
        (big ? major : minor).push(lineOnTop([cx + sx * inner, cy + sy * inner], [cx + sx * radius, cy + sy * radius], z, projection));
    }
    return { minor: minor.join(""), major: major.join("") };
}

return { topTicks, sideTicks, radialTicks };
})();
const M7 = (() => { // iso-kit/transformacao.mjs
const { iso } = M1;
// Transformações SVG (translate/matrix) para fixar textos e figuras nas faces do sólido.

function axisVector(projection, [dx, dy, dz]) {
    const origin = iso([0, 0, 0], projection);
    const moved = iso([dx, dy, dz], projection);
    return [moved[0] - origin[0], moved[1] - origin[1]];
}
const fine = (value) => Math.round(value * 100) / 100;
function translateAlong(projection, by) {
    const [x, y] = axisVector(projection, by);
    return `translate(${fine(x)} ${fine(y)})`;
}
const thousandth = (value) => Math.round(value * 1000) / 1000;
function axesAt(at, z, projection) {
    const origin = iso([at[0], at[1], z], projection);
    const along = (point) => {
        const [px, py] = iso(point, projection);
        return [px - origin[0], py - origin[1]];
    };
    return { origin, x: along([at[0] + 1, at[1], z]), y: along([at[0], at[1] + 1, z]), z: along([at[0], at[1], z + 1]) };
}
const matrixOf = (u, v, origin) => `matrix(${thousandth(u[0])} ${thousandth(u[1])} ${thousandth(v[0])} ${thousandth(v[1])} ${thousandth(origin[0])} ${thousandth(origin[1])})`;
function holdStill(element, [dx, dy]) {
    const text = dx || dy ? `translate(${-dx} ${-dy})` : "";
    element.querySelectorAll("[data-hold]").forEach((node) => (text ? node.setAttribute("transform", text) : node.removeAttribute("transform")));
}
function topMatrix(at, z, projection, along = "x") {
    const { origin, x, y } = axesAt(at, z, projection);
    return along === "x" ? matrixOf(x, y, origin) : matrixOf([-y[0], -y[1]], x, origin);
}
function sideMatrix(at, z, projection, face) {
    const { origin, x, y, z: up } = axesAt(at, z, projection);
    const down = [-up[0], -up[1]];
    return face === "left" ? matrixOf(x, down, origin) : matrixOf([-y[0], -y[1]], down, origin);
}

return { axisVector, translateAlong, holdStill, topMatrix, sideMatrix };
})();
const M8 = (() => { // iso-kit/css.mjs
// Folha de estilo base do kit (temas, sólidos, linhas, placa).
const ISO_CSS = `
.iso{
  --anatomy-page:#0b0b0b;--anatomy-card:#131313;--anatomy-text:#a3a3a3;--anatomy-strong:#e6e6e6;--anatomy-muted:#6e6e6e;
  --anatomy-paper:#131313;--anatomy-face:#1b1b1b;--anatomy-top:#1c1c1c;
  --anatomy-shade-0:#0f0f0f;--anatomy-shade-1:#121212;--anatomy-shade-2:#151515;--anatomy-shade-3:#181818;
  --anatomy-lit-top:#262626;--anatomy-lit-shade:#1d1d1d;
  --anatomy-hi:#d4d4d4;--anatomy-mid:#6a6a6a;--anatomy-lo:#3e3e3e;--anatomy-faint:#2a2a2a;--anatomy-lit:#ffffff;
  --anatomy-halo:#050505;--anatomy-halo-opacity:.9;--anatomy-dot:#e6e6e6;
  --anatomy-red:#ff6b6b;--anatomy-green:#6be0a0;--anatomy-blue:#6b9bff;
  --anatomy-weight:.6px;--anatomy-ease:cubic-bezier(.32,.72,0,1);
  --anatomy-mono:ui-monospace,"SF Mono",Menlo,Consolas,monospace;
  --anatomy-lift:inset 0 1px 0 0 #ffffff0a,inset 0 0 0 1px #ffffff06,0 0 0 1px #00000026,0 1px 1px -.5px #00000014,0 3px 3px -1.5px #00000014,0 6px 6px -3px #00000014,0 12px 12px -6px #00000014;
}
.iso[data-theme="light"]{
  --anatomy-page:#f2f2ef;--anatomy-card:#fbfbfa;--anatomy-text:#57575a;--anatomy-strong:#18181a;--anatomy-muted:#8c8c90;
  --anatomy-paper:#fbfbfa;--anatomy-face:#ffffff;--anatomy-top:#ffffff;
  --anatomy-shade-0:#dcdcd9;--anatomy-shade-1:#e5e5e2;--anatomy-shade-2:#ededea;--anatomy-shade-3:#f4f4f1;
  --anatomy-lit-top:#ffffff;--anatomy-lit-shade:#eeeeeb;
  --anatomy-hi:#26262a;--anatomy-mid:#8a8a8f;--anatomy-lo:#b4b4b8;--anatomy-faint:#d4d4d6;--anatomy-lit:#000000;
  --anatomy-halo:#8a8a80;--anatomy-halo-opacity:.32;--anatomy-dot:#3a3a3e;
  --anatomy-red:#e5484d;--anatomy-green:#30a46c;--anatomy-blue:#3e63dd;
  --anatomy-lift:inset 0 1px 0 0 #ffffff,0 0 0 1px #0000000f,0 1px 1px -.5px #0000000a,0 3px 3px -1.5px #0000000a,0 6px 6px -3px #0000000a,0 12px 12px -6px #0000000a;
}
.iso-svg{display:block;width:100%;height:auto;overflow:visible}
.iso-svg path,.iso-svg circle{vector-effect:non-scaling-stroke}
.iso-fill{fill:var(--anatomy-paper);stroke:none}
.iso-solid[data-lit]>.iso-fill{fill:var(--anatomy-face)}
.iso-shade,.iso-top{stroke:none}
.iso-shade[data-shade="0"]{fill:var(--anatomy-shade-0)}
.iso-shade[data-shade="1"]{fill:var(--anatomy-shade-1)}
.iso-shade[data-shade="2"]{fill:var(--anatomy-shade-2)}
.iso-shade[data-shade="3"]{fill:var(--anatomy-shade-3)}
.iso-top{fill:var(--anatomy-top)}
.iso-solid[data-lit]>.iso-top{fill:var(--anatomy-lit-top)}
.iso-solid[data-lit]>.iso-shade{fill:var(--anatomy-lit-shade)}
.iso-solid[data-seamless]>.iso-shade,.iso-solid[data-seamless]>.iso-top{stroke-width:.4;stroke-linejoin:round}
.iso-solid[data-seamless]>.iso-shade[data-shade="0"]{stroke:var(--anatomy-shade-0)}
.iso-solid[data-seamless]>.iso-shade[data-shade="1"]{stroke:var(--anatomy-shade-1)}
.iso-solid[data-seamless]>.iso-shade[data-shade="2"]{stroke:var(--anatomy-shade-2)}
.iso-solid[data-seamless]>.iso-shade[data-shade="3"]{stroke:var(--anatomy-shade-3)}
.iso-solid[data-seamless]>.iso-top{stroke:var(--anatomy-top)}
.iso-solid[data-seamless][data-lit]>.iso-top{stroke:var(--anatomy-lit-top)}
.iso-solid[data-seamless][data-lit]>.iso-shade{stroke:var(--anatomy-lit-shade)}
.iso-solid[data-lit]>.iso-edge{stroke:var(--anatomy-lit)}
.iso-halo{fill:var(--anatomy-halo);stroke:none;filter:blur(9px);opacity:var(--anatomy-halo-opacity);transform:translateY(3px)}
.iso-line{fill:none;stroke:var(--anatomy-lo);stroke-width:var(--anatomy-weight);stroke-linecap:round;stroke-linejoin:round}
.iso-line[data-tone="hi"]{stroke:var(--anatomy-hi)}
.iso-line[data-tone="mid"]{stroke:var(--anatomy-mid)}
.iso-line[data-tone="lo"]{stroke:var(--anatomy-lo)}
.iso-line[data-tone="faint"]{stroke:var(--anatomy-faint)}
.iso-line[data-tone="lit"]{stroke:var(--anatomy-lit)}
.iso-line[data-tone="red"]{stroke:var(--anatomy-red)}
.iso-line[data-tone="green"]{stroke:var(--anatomy-green)}
.iso-line[data-tone="blue"]{stroke:var(--anatomy-blue)}
.iso-line[data-dotted]{stroke-dasharray:.01 3.5;stroke-width:1}
.iso-line[data-dashed]{stroke-dasharray:3 3}
.iso-line[data-flow]{stroke-dasharray:2 6}
.iso-wire-end{fill:var(--anatomy-paper);stroke:var(--anatomy-lo);stroke-width:var(--anatomy-weight)}
.iso-wire[data-tone="hi"] .iso-wire-end{stroke:var(--anatomy-hi)}
.iso-wire[data-tone="mid"] .iso-wire-end{stroke:var(--anatomy-mid)}
.iso-wire[data-tone="faint"] .iso-wire-end{stroke:var(--anatomy-faint)}
.iso-wire[data-tone="lit"] .iso-wire-end{stroke:var(--anatomy-lit)}
.iso-dots circle{fill:var(--anatomy-dot);stroke:none;opacity:.32}
.iso-dots[data-tone="hi"] circle{opacity:.85}
.iso-dots[data-tone="mid"] circle{opacity:.45}
.iso-dots[data-tone="lo"] circle{opacity:.22}
.iso-dots[data-tone="faint"] circle{opacity:.12}
.iso-face-text{fill:var(--anatomy-mid);font-family:var(--anatomy-mono);letter-spacing:.04em}
.iso-face-text[data-tone="hi"]{fill:var(--anatomy-hi)}
.iso-face-text[data-tone="lo"]{fill:var(--anatomy-lo)}
.iso-face-text[data-tone="faint"]{fill:var(--anatomy-faint)}
.tb-body{fill:var(--anatomy-shade-2);stroke:none}
.tb-shine{fill:var(--anatomy-top);stroke:none}
.tb-shade{fill:var(--anatomy-shade-0);stroke:none}
.tb-ring{fill:none;stroke:var(--anatomy-lo);stroke-width:.45;stroke-linecap:butt;stroke-linejoin:round}
.tb-ring[data-tone="mid"]{stroke:var(--anatomy-mid)}
.tb-ring[data-tone="faint"]{stroke:var(--anatomy-faint)}
.tb-ring[data-tone="hi"]{stroke:var(--anatomy-hi)}
@keyframes iso-flow{to{stroke-dashoffset:-32}}
@keyframes iso-pulse{0%,100%{opacity:.18}12%{opacity:1}40%{opacity:.18}}
.iso-line[data-flow]{animation:iso-flow 1600ms linear infinite}
.iso-dots circle[data-pulse]{animation:iso-pulse 1600ms steps(1,end) infinite;animation-delay:var(--pulse-at,0ms)}
.iso-figure{margin:32px 0 36px}
.iso-plate{position:relative;padding:52px 24px;border-radius:0;background:var(--anatomy-card);box-shadow:0 0 0 1px var(--anatomy-mid);--anatomy-paper:var(--anatomy-card)}
.iso-plate-corners{position:absolute;inset:18px 20px;pointer-events:none}
.iso-plate-corner{position:absolute;color:var(--anatomy-muted);font-family:var(--anatomy-mono);font-size:10.5px;line-height:1;letter-spacing:.04em;white-space:nowrap}
.iso-plate-corner[data-corner="fig"]{top:0;left:0;color:var(--anatomy-text)}
.iso-plate-corner[data-corner="title"]{top:0;right:0;text-transform:uppercase;letter-spacing:.1em}
.iso-plate-corner[data-corner="hint"]{bottom:0;left:0;text-transform:uppercase;letter-spacing:.1em}
.iso-plate-corner[data-corner="readout"]{right:0;bottom:0;color:var(--anatomy-text);font-variant-numeric:tabular-nums}
.iso-plate .iso-svg{margin:0 auto}
.iso-legend{display:grid;gap:8px;margin-top:12px;color:var(--anatomy-muted);font-size:13px;line-height:1.55}
.iso-keys{display:flex;flex-wrap:wrap;gap:6px 18px}
.iso-key{display:inline-flex;align-items:center;gap:7px}
.iso-swatch{width:16px;height:10px;flex:none;overflow:visible}
.iso-swatch path{fill:none;stroke:var(--anatomy-hi);stroke-width:1;vector-effect:non-scaling-stroke;stroke-linejoin:round;stroke-linecap:round}
.iso-swatch[data-mark="raised"] path,.iso-swatch[data-mark="flat"] path{stroke:var(--anatomy-mid)}
.iso-swatch[data-mark="lit"] path{stroke:var(--anatomy-lit)}
.iso-swatch[data-mark="dim"] path{stroke:var(--anatomy-lo)}
.iso-swatch[data-mark="dotted"] path{stroke-dasharray:.01 3;stroke-width:1.4}
.iso-swatch[data-mark="dashed"] path{stroke-dasharray:3 2.5}
.iso-swatch[data-mark="red"] path{stroke:var(--anatomy-red)}
.iso-swatch[data-mark="green"] path{stroke:var(--anatomy-green)}
.iso-swatch[data-mark="blue"] path{stroke:var(--anatomy-blue)}
.iso-swatch-crease{stroke:var(--anatomy-lo)!important}
@media (max-width:560px){.iso-plate{padding:48px 12px 70px}.iso-plate-corner[data-corner="hint"]{bottom:18px}}
@media (prefers-reduced-motion:reduce){.iso-line[data-flow],.iso-dots circle[data-pulse]{animation:none}}
`;

return { ISO_CSS };
})();
const M9 = (() => { // iso-kit/svg.mjs
const { pathOf } = M2;
const { attrs, escape, tenth } = M0;
const { ISO_CSS } = M8;
// Saída SVG: sólidos, linhas, pontos, fios, textos, grupos e a figura completa.



function solidSvg(paths, { tone = "hi", crease = "faint", halo = false, lit = false, flat = false, className, inner = "" } = {}) {
    const parts = [];
    if (halo)
        parts.push(`<path class="iso-halo" d="${paths.fill}"/>`);
    parts.push(`<path class="iso-fill" d="${paths.fill}"/>`);
    if (!flat) {
        paths.shades.forEach((d, shade) => {
            if (d)
                parts.push(`<path class="iso-shade" data-shade="${shade}" d="${d}"/>`);
        });
        parts.push(`<path class="iso-top" d="${paths.top}"/>`);
    }
    if (crease !== "none" && paths.crease)
        parts.push(`<path class="iso-line iso-crease" data-tone="${crease}" d="${paths.crease}"/>`);
    if (crease !== "none" && paths.bevel)
        parts.push(`<path class="iso-line iso-bevel" data-tone="${crease}" d="${paths.bevel}"/>`);
    if (inner)
        parts.push(inner);
    parts.push(`<path class="iso-line iso-edge" data-tone="${tone}" d="${paths.outline}"/>`);
    return `<g${attrs({ class: className ? `iso-solid ${className}` : "iso-solid", "data-lit": lit ? "" : undefined, "data-seamless": paths.seamless ? "" : undefined })}>${parts.join("")}</g>`;
}
function lineSvg(d, { tone = "lo", dotted = false, dashed = false, flow = false, className, free } = {}) {
    if (!d)
        return "";
    return `<path${attrs({
        class: className ? `iso-line ${className}` : "iso-line",
        "data-tone": tone,
        "data-free": free === true ? "" : free || undefined,
        "data-dotted": dotted ? "" : undefined,
        "data-dashed": dashed ? "" : undefined,
        "data-flow": flow ? "" : undefined,
        d,
    })}/>`;
}
function dotsSvg(points, { size = 0.5, tone = "mid", pulse = false } = {}) {
    const dots = points
        .map(([x, y, alpha], index) => `<circle cx="${tenth(x)}" cy="${tenth(y)}" r="${size}"${alpha !== undefined && alpha < 1 ? ` fill-opacity="${Math.max(0, alpha).toFixed(2)}"` : ""}${pulse ? ` data-pulse="" style="--pulse-at:${(index * 137) % 1600}ms"` : ""}/>`)
        .join("");
    return `<g class="iso-dots" data-tone="${tone}">${dots}</g>`;
}
function fadedSvg(list, style = {}) {
    const groups = new Map();
    for (const entry of list) {
        const [d, alpha] = Array.isArray(entry) ? entry : [entry.d, entry.alpha];
        if (!d || !(alpha > 0.02))
            continue;
        const level = Math.min(1, Math.round(alpha * 20) / 20);
        groups.set(level, (groups.get(level) ?? "") + d);
    }
    return [...groups].map(([alpha, d]) => (alpha >= 1 ? lineSvg(d, style) : `<g opacity="${alpha.toFixed(2)}">${lineSvg(d, style)}</g>`)).join("");
}
function fadeLineSvg(points, { fade = [0, 0], tone = "lo", className, stops = 6 } = {}) {
    if (points.length < 2)
        return "";
    const ease = (t) => t * t * (3 - 2 * t);
    const hashOf = (text) => {
        let hash = 2166136261;
        for (let index = 0; index < text.length; index++)
            hash = Math.imul(hash ^ text.charCodeAt(index), 16777619);
        return (hash >>> 0).toString(36);
    };
    const lengths = [0];
    for (let index = 1; index < points.length; index++)
        lengths.push(lengths[index - 1] + Math.hypot(points[index][0] - points[index - 1][0], points[index][1] - points[index - 1][1]));
    const at = (length) => {
        let index = 1;
        while (index < points.length - 1 && lengths[index] < length)
            index++;
        const span = lengths[index] - lengths[index - 1] || 1;
        const t = Math.max(0, Math.min(1, (length - lengths[index - 1]) / span));
        return [points[index - 1][0] + (points[index][0] - points[index - 1][0]) * t, points[index - 1][1] + (points[index][1] - points[index - 1][1]) * t];
    };
    const pieceOf = (from, to) => [at(from), ...points.filter((_, index) => lengths[index] > from + 1e-6 && lengths[index] < to - 1e-6), at(to)];
    const total = lengths[lengths.length - 1];
    if (total < 1e-6)
        return "";
    let [start, end] = Array.isArray(fade) ? fade : [fade, fade];
    if (start + end > total) {
        const share = total / (start + end);
        start *= share;
        end *= share;
    }
    const paint = `var(--anatomy-${tone})`;
    const ramp = (piece, rising) => {
        const d = pathOf(piece);
        const [x1, y1] = piece[0];
        const [x2, y2] = piece[piece.length - 1];
        const id = `iso-fade-${hashOf(`${d}|${tone}|${rising}`)}`;
        const marks = Array.from({ length: stops + 1 }, (_, index) => {
            const t = index / stops;
            const alpha = ease(rising ? t : 1 - t);
            return `<stop offset="${t.toFixed(3)}" stop-opacity="${alpha.toFixed(3)}" style="stop-color:${paint}"/>`;
        }).join("");
        const gradient = `<linearGradient id="${id}" gradientUnits="userSpaceOnUse" x1="${tenth(x1)}" y1="${tenth(y1)}" x2="${tenth(x2)}" y2="${tenth(y2)}">${marks}</linearGradient>`;
        return `<defs>${gradient}</defs><path${attrs({ class: className ? `iso-line ${className}` : "iso-line", "data-tone": tone, "data-free": rising ? "start" : "end", d, style: `stroke:url(#${id})` })}/>`;
    };
    const parts = [];
    if (start > 0.05)
        parts.push(ramp(pieceOf(0, start), true));
    if (total - end - start > 0.05)
        parts.push(lineSvg(pathOf(pieceOf(start, total - end)), { tone, className }));
    if (end > 0.05)
        parts.push(ramp(pieceOf(total - end, total), false));
    return `<g class="iso-fade">${parts.join("")}</g>`;
}
function wireSvg(from, to, { via = [], tone = "lo", dotted = false, dashed = false, flow = false, ends = "both", size = 1.2 } = {}) {
    const d = pathOf([from, ...via, to]);
    const end = ([x, y]) => `<circle class="iso-wire-end" cx="${tenth(x)}" cy="${tenth(y)}" r="${size}"/>`;
    return `<g class="iso-wire" data-tone="${tone}">${lineSvg(d, { tone, dotted, dashed, flow })}${ends === "both" || ends === "from" ? end(from) : ""}${ends === "both" || ends === "to" ? end(to) : ""}</g>`;
}
function faceTextSvg(transform, text, { size = 3, tone = "mid", anchor = "start" } = {}) {
    return `<text class="iso-face-text" data-tone="${tone}" transform="${transform}" font-size="${size}" text-anchor="${anchor}">${escape(text)}</text>`;
}
function groupSvg(children, extra = {}) {
    return `<g${attrs(extra)}>${Array.isArray(children) ? children.join("") : children}</g>`;
}
function figureSvg({ width, height, label, body, standalone = false, className }) {
    const style = standalone ? `<style>${ISO_CSS}</style>` : "";
    return `<svg${attrs({
        xmlns: "http://www.w3.org/2000/svg",
        class: `iso-svg${standalone ? " iso" : ""}${className ? ` ${className}` : ""}`,
        "data-theme": standalone || undefined,
        viewBox: `0 0 ${width} ${height}`,
        role: "img",
        "aria-label": label,
    })}>${style}${Array.isArray(body) ? body.join("") : body}</svg>`;
}

return { solidSvg, lineSvg, dotsSvg, fadedSvg, fadeLineSvg, wireSvg, faceTextSvg, groupSvg, figureSvg };
})();
const M10 = (() => { // iso-kit/placa.mjs
const { escape } = M0;
const { ISO_CSS } = M8;
// Placa (figure) com legenda e página HTML completa em volta do SVG.


function swatchSvg(mark) {
    if (mark === "raised" || mark === "flat" || mark === "lit") {
        const body = mark === "flat" ? "M2 6.5 8 3.5 14 6.5 8 9.5Z" : "M2 4.5 8 1.5 14 4.5 14 6 8 9 2 6Z";
        const crease = mark === "flat" ? "" : `<path class="iso-swatch-crease" d="M2 4.5 8 7.5 14 4.5"/>`;
        return `<svg class="iso-swatch" data-mark="${mark}" viewBox="0 0 16 10" aria-hidden="true"><path d="${body}"/>${crease}</svg>`;
    }
    return `<svg class="iso-swatch" data-mark="${mark}" viewBox="0 0 16 10" aria-hidden="true"><path d="M1.5 5h13"/></svg>`;
}
function plateHtml({ fig, title, hint, readout = "", keys = [], caption, body }) {
    const corner = (name, text, extra = "") => `<span class="iso-plate-corner" data-corner="${name}"${extra}>${escape(text)}</span>`;
    const legend = keys.length || caption
        ? `<figcaption class="iso-legend">${keys.length ? `<span class="iso-keys">${keys.map((key) => `<span class="iso-key">${swatchSvg(key.mark)}${escape(key.label)}</span>`).join("")}</span>` : ""}${caption ? `<span class="iso-caption">${escape(caption)}</span>` : ""}</figcaption>`
        : "";
    return `<figure class="iso-figure"><div class="iso-plate"><div class="iso-plate-corners" aria-hidden="true">${corner("fig", fig)}${title ? corner("title", title) : ""}${hint ? corner("hint", hint) : ""}${corner("readout", readout, " data-readout")}</div>${body}</div>${legend}</figure>`;
}
function pageHtml({ title, theme = "dark", body, script = "", width = 650 }) {
    return `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>${escape(title)}</title><style>${ISO_CSS}${PAGE_CSS.replace("__WIDTH__", String(width))}</style></head><body class="iso" data-theme="${theme}"><main class="iso-page">${body}</main>${script ? `<script type="module">${script.replace(/<\/(script)/gi, "<\\/$1")}</script>` : ""}</body></html>`;
}
const PAGE_CSS = `
*{box-sizing:border-box}
html{color-scheme:dark}
body[data-theme="light"]{color-scheme:light}
body{margin:0;background:var(--anatomy-page);color:var(--anatomy-text);font:14px/1.6 ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif;-webkit-font-smoothing:antialiased}
.iso-page{width:min(__WIDTH__px,100% - 32px);margin:64px auto 96px}
`;

return { plateHtml, pageHtml };
})();
const M11 = Object.assign({}, M1, M2, M3, M4, M5, M6, M7, M8, M9, M10); // iso-kit.mjs
const M12 = (() => { // lathe/vetor.mjs
// Vetores 3D como arrays [x, y, z].

const dot3 = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const add3 = (a, b) => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const sub3 = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const mul3 = (a, k) => [a[0] * k, a[1] * k, a[2] * k];
const len3 = (a) => Math.hypot(a[0], a[1], a[2]);
const unit3 = (a) => {
  const length = len3(a) || 1;
  return [a[0] / length, a[1] / length, a[2] / length];
};

const cross3 = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const lerp3 = (a, b, t) => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];

return { dot3, add3, sub3, mul3, len3, unit3, cross3, lerp3 };
})();
const M13 = (() => { // lathe/interno.mjs
// Constantes e auxiliares internos compartilhados entre as partes (não reexportado pelo índice).

const TAU = Math.PI * 2;
const BANDS = [0, 0.2, 0.42, 0.62];

function hullTagged(points) {
  const sorted = [...points].sort((a, b) => a.p[0] - b.p[0] || a.p[1] - b.p[1]);
  if (sorted.length < 3) return sorted;
  const cross = (o, a, b) => (a.p[0] - o.p[0]) * (b.p[1] - o.p[1]) - (a.p[1] - o.p[1]) * (b.p[0] - o.p[0]);
  const lower = [];
  for (const point of sorted) {
    while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], point) <= 1e-9) lower.pop();
    lower.push(point);
  }
  const upper = [];
  for (let index = sorted.length - 1; index >= 0; index--) {
    const point = sorted[index];
    while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], point) <= 1e-9) upper.pop();
    upper.push(point);
  }
  return [...lower.slice(0, -1), ...upper.slice(0, -1)];
}

function groupsOf(keys) {
  const count = keys.length;
  const out = [];
  const start = keys.findIndex((key, index) => key !== keys[(index + count - 1) % count]);
  if (start < 0) {
    if (keys[0] !== null) out.push({ key: keys[0], run: Array.from({ length: count + 1 }, (_, index) => index % count) });
    return out;
  }
  let current = null;
  for (let step = 0; step < count; step++) {
    const index = (start + step) % count;
    const key = keys[index];
    if (current && current.key === key) {
      current.run.push((index + 1) % count);
      continue;
    }
    if (current && current.key !== null) out.push(current);
    current = { key, run: [index, (index + 1) % count] };
  }
  if (current && current.key !== null) out.push(current);
  return out;
}

const signedArea = (points) => points.reduce((sum, [x, y], index) => {
  const [nx, ny] = points[(index + 1) % points.length];
  return sum + x * ny - nx * y;
}, 0);

const oriented = (points) => (signedArea(points) < 0 ? [...points].reverse() : points);

return { TAU, BANDS, hullTagged, groupsOf, signedArea, oriented };
})();
const M14 = (() => { // lathe/luz.mjs
const { cameraOf } = M11;
const { BANDS } = M13;
const { dot3, unit3 } = M12;
// Câmera, luz e tons: direção de vista, direção da luz, escala e faixas de tom.



const LIGHT_UP = 1.2;
function viewOf(P) {
  const { sinA, cosA, sinE, cosE } = cameraOf(P);
  return [cosA * cosE, sinA * cosE, sinE];
}

function lightOf(P) {
  const { sinA, cosA } = cameraOf(P);
  return unit3([-sinA, cosA, LIGHT_UP]);
}

const scaleOf = (P) => cameraOf(P).k;

function bandOf(score) {
  let tone = 0;
  while (tone < BANDS.length && score >= BANDS[tone]) tone++;
  return tone;
}

const toneOf = (normal, P) => bandOf(dot3(unit3(normal), lightOf(P)));

return { viewOf, lightOf, scaleOf, bandOf, toneOf };
})();
const M15 = (() => { // lathe/quadro.mjs
const { dot3, unit3, cross3 } = M12;
// Quadros de eixo (origem, eixo, u, v), pontos em coordenadas cilíndricas e trajetos de espiral/arco.

function frameAlong(o, a) {
  const axis = unit3(a);
  const helper = Math.abs(axis[2]) > 0.9 ? [1, 0, 0] : [0, 0, 1];
  const u = unit3(cross3(helper, axis));
  const v = cross3(axis, u);
  return { o, a: axis, u, v };
}

const frameOf = (o, a, u, v) => ({ o, a, u, v });

function pointOf(F, s, r, angle) {
  const c = Math.cos(angle);
  const n = Math.sin(angle);
  return [
    F.o[0] + F.a[0] * s + r * (c * F.u[0] + n * F.v[0]),
    F.o[1] + F.a[1] * s + r * (c * F.u[1] + n * F.v[1]),
    F.o[2] + F.a[2] * s + r * (c * F.u[2] + n * F.v[2]),
  ];
}

const radialOf = (F, angle) => {
  const c = Math.cos(angle);
  const n = Math.sin(angle);
  return [c * F.u[0] + n * F.v[0], c * F.u[1] + n * F.v[1], c * F.u[2] + n * F.v[2]];
};

const tangentOf = (F, angle) => {
  const c = Math.cos(angle);
  const n = Math.sin(angle);
  return [-n * F.u[0] + c * F.v[0], -n * F.u[1] + c * F.v[1], -n * F.u[2] + c * F.v[2]];
};

const angleToward = (F, direction) => Math.atan2(dot3(direction, F.v), dot3(direction, F.u));

function spiral(F, s, r0, r1, a0, a1, steps = 28) {
  const out = [];
  for (let index = 0; index <= steps; index++) {
    const t = index / steps;
    out.push(pointOf(F, s, r0 + (r1 - r0) * t, a0 + (a1 - a0) * t));
  }
  return out;
}

function arcPoints(F, s, r, a0, a1, steps = 24) {
  return Array.from({ length: steps + 1 }, (_, index) => pointOf(F, s, r, a0 + ((a1 - a0) * index) / steps));
}

return { frameAlong, frameOf, pointOf, radialOf, tangentOf, angleToward, spiral, arcPoints };
})();
const M16 = (() => { // lathe/planar.mjs
const { TAU, hullTagged } = M13;
// Utilitários planares: casco convexo, trechos contíguos e passos de amostragem.

const hullOf = (points) => hullTagged(points.map((p) => ({ p }))).map(({ p }) => p);

function runsOf(flags) {
  const count = flags.length;
  const start = flags.findIndex((flag, index) => flag !== flags[(index + count - 1) % count]);
  if (start < 0) return flags[0] ? [Array.from({ length: count + 1 }, (_, index) => index % count)] : [];
  const runs = [];
  let run = null;
  for (let step = 0; step < count; step++) {
    const index = (start + step) % count;
    if (flags[index]) {
      if (!run) run = [index];
      run.push((index + 1) % count);
    } else if (run) {
      runs.push(run);
      run = null;
    }
  }
  if (run) runs.push(run);
  return runs;
}

const stepsFor = (radius, least = 16, most = 112, scale = 0) => Math.max(least, Math.min(most, scale ? Math.ceil((TAU * radius * scale) / 3) : Math.round((radius * 1.3) / 4) * 4));

return { hullOf, runsOf, stepsFor };
})();
const M17 = (() => { // lathe/curva.mjs
// Curvas do perfil: inclinações de Hermite, trechos suaves e curva do perfil.

function hermiteSlopes(run) {
  const n = run.length - 1;
  const h = Array.from({ length: n }, (_, j) => run[j + 1][0] - run[j][0]);
  const delta = h.map((value, j) => (run[j + 1][1] - run[j][1]) / value);
  const d = new Array(n + 1).fill(0);
  const end = (h0, h1, d0, d1) => {
    let value = ((2 * h0 + h1) * d0 - h0 * d1) / (h0 + h1);
    if (Math.sign(value) !== Math.sign(d0)) value = 0;
    else if (Math.sign(d0) !== Math.sign(d1) && Math.abs(value) > Math.abs(3 * d0)) value = 3 * d0;
    return value;
  };
  if (n === 1) d[0] = d[1] = delta[0];
  else {
    d[0] = end(h[0], h[1], delta[0], delta[1]);
    d[n] = end(h[n - 1], h[n - 2], delta[n - 1], delta[n - 2]);
  }
  for (let k = 1; k < n; k++) {
    if (delta[k - 1] * delta[k] <= 0) continue;
    const w1 = 2 * h[k] + h[k - 1];
    const w2 = h[k] + 2 * h[k - 1];
    d[k] = (w1 + w2) / (w1 / delta[k - 1] + w2 / delta[k]);
  }
  return d;
}

function curveRuns(meridian) {
  const m = meridian.length;
  const segments = new Array(m - 1).fill(null);
  let start = 0;
  while (start < m - 1) {
    let end = start + 1;
    while (end < m - 1 && meridian[end][2] && Math.abs(meridian[end + 1][0] - meridian[end][0]) > 1e-9 && Math.abs(meridian[end][0] - meridian[end - 1][0]) > 1e-9) end++;
    const run = meridian.slice(start, end + 1);
    const n = run.length - 1;
    if (n >= 2 && run.every((point, j) => j === 0 || Math.abs(point[0] - run[j - 1][0]) > 1e-9)) {
      const d = hermiteSlopes(run);
      for (let j = 0; j < n; j++) {
        const [s0, r0] = run[j];
        const [s1, r1] = run[j + 1];
        const hj = s1 - s0;
        const d0 = d[j];
        const d1 = d[j + 1];
        const radius = (s) => {
          const t = (s - s0) / hj;
          return (2 * t ** 3 - 3 * t ** 2 + 1) * r0 + (t ** 3 - 2 * t ** 2 + t) * hj * d0 + (-2 * t ** 3 + 3 * t ** 2) * r1 + (t ** 3 - t ** 2) * hj * d1;
        };
        const slope = (s) => {
          const t = (s - s0) / hj;
          return ((6 * t ** 2 - 6 * t) * r0 + (6 * t - 6 * t ** 2) * r1) / hj + (3 * t ** 2 - 4 * t + 1) * d0 + (3 * t ** 2 - 2 * t) * d1;
        };
        segments[start + j] = { radius, slope };
      }
    }
    start = end;
  }
  return segments;
}

function profileCurve(profile, { slope = null, radius = null } = {}) {
  for (let index = 1; index < profile.length; index++) if (profile[index][0] - profile[index - 1][0] <= 1e-9) throw new Error(`profileCurve: s must rise along the profile (point ${index} at s = ${profile[index][0]})`);
  const segments = curveRuns(profile);
  const s0 = profile[0][0];
  const s1 = profile[profile.length - 1][0];
  const find = (s) => {
    let lo = 0;
    let hi = profile.length - 2;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (profile[mid][0] <= s) lo = mid;
      else hi = mid - 1;
    }
    return lo;
  };
  const linear = (index, s) => {
    const [sa, ra] = profile[index];
    const [sb, rb] = profile[index + 1];
    return [ra + ((rb - ra) * (s - sa)) / (sb - sa), (rb - ra) / (sb - sa)];
  };
  const radiusAt = radius ?? ((s) => {
    const at = Math.max(s0, Math.min(s1, s));
    const index = find(at);
    return segments[index] ? segments[index].radius(at) : linear(index, at)[0];
  });
  const slopeAt = slope ?? ((s) => {
    const at = Math.max(s0, Math.min(s1, s));
    const index = find(at);
    return segments[index] ? segments[index].slope(at) : linear(index, at)[1];
  });
  return { s0, s1, radius: radiusAt, slope: slopeAt, analytic: Boolean(slope && radius) };
}

return { curveRuns, profileCurve };
})();
const M18 = (() => { // lathe/silhueta.mjs
const { iso, pathOf } = M11;
const { dot3 } = M12;
const { viewOf } = M14;
const { pointOf } = M15;
// Silhuetas de perfis de revolução vistas pela câmera.




function silhouettePoints(profile, F, P, { samples = 72, slope: given = null, radius = null } = {}) {
  const V = viewOf(P);
  const aV = dot3(F.a, V);
  const uV = dot3(F.u, V);
  const vV = dot3(F.v, V);
  const facing = Math.atan2(vV, uV);
  const flat = Math.hypot(uV, vV) || 1e-9;
  const s0 = profile[0][0];
  const s1 = profile[profile.length - 1][0];
  const radiusAt = radius ?? ((s) => {
    let index = 1;
    while (index < profile.length - 1 && profile[index][0] < s) index++;
    const [sa, ra] = profile[index - 1];
    const [sb, rb] = profile[index];
    return ra + ((rb - ra) * (s - sa)) / (sb - sa || 1);
  });
  const side = (sign) => {
    const points = [];
    for (let step = 0; step <= samples; step++) {
      const s = s0 + ((s1 - s0) * step) / samples;
      const h = (s1 - s0) / samples / 2 || 0.01;
      const slope = given ? given(s) : (radiusAt(Math.min(s1, s + h)) - radiusAt(Math.max(s0, s - h))) / (Math.min(s1, s + h) - Math.max(s0, s - h) || 1);
      const c = (slope * aV) / flat;
      if (Math.abs(c) > 1) continue;
      points.push(pointOf(F, s, radiusAt(s), facing + sign * Math.acos(c)));
    }
    return points;
  };
  return [side(1), side(-1)];
}

const silhouetteOf = (profile, F, P, options = {}) =>
  silhouettePoints(profile, F, P, options)
    .map((points) => pathOf(points.map((point) => iso(point, P))))
    .join("");

return { silhouettePoints, silhouetteOf };
})();
const M19 = (() => { // lathe/suave.mjs
const { iso, pathOf } = M11;
const { TAU, BANDS, oriented } = M13;
const { dot3 } = M12;
const { viewOf, lightOf, scaleOf, bandOf } = M14;
const { pointOf, arcPoints } = M15;
const { curveRuns } = M17;
// Superfícies suaves de revolução: faixas de tom e linhas por fileira (interno, usado por lathe).






function labelledArcs(nr, na, view, light, gr = nr, ga = na) {
  const cuts = [];
  if (Math.abs(gr) > 1e-9) {
    const visibleCut = -(ga * view.a) / (gr * view.flat);
    if (visibleCut > -1 && visibleCut < 1) {
      const d = Math.acos(visibleCut);
      cuts.push(view.phi - d, view.phi + d);
    }
  }
  if (Math.abs(nr) > 1e-9) {
    for (const threshold of BANDS) {
      const c = (threshold - na * light.a) / (nr * light.flat);
      if (c > -1 && c < 1) {
        const d = Math.acos(c);
        cuts.push(light.phi - d, light.phi + d);
      }
    }
  }
  const wrap = (angle) => ((angle % TAU) + TAU) % TAU;
  const sorted = [...new Set(cuts.map((angle) => Math.round(wrap(angle) * 1e9) / 1e9))].sort((a, b) => a - b);
  const label = (angle) => {
    const facing = gr * view.flat * Math.cos(angle - view.phi) + ga * view.a;
    if (facing <= 1e-9) return -1;
    return bandOf(nr * light.flat * Math.cos(angle - light.phi) + na * light.a);
  };
  if (!sorted.length) {
    const tone = label(0);
    return tone < 0 ? [] : [{ tone, from: 0, to: TAU, full: true }];
  }
  const arcs = sorted.map((from, index) => {
    const to = index + 1 < sorted.length ? sorted[index + 1] : sorted[0] + TAU;
    return { tone: label((from + to) / 2), from, to };
  });
  const merged = [];
  for (const arc of arcs) {
    const last = merged[merged.length - 1];
    if (last && last.tone === arc.tone && Math.abs(last.to - arc.from) < 1e-9) last.to = arc.to;
    else merged.push({ ...arc });
  }
  if (merged.length > 1 && merged[0].tone === merged[merged.length - 1].tone && Math.abs(merged[merged.length - 1].to - TAU - merged[0].from) < 1e-9) {
    const last = merged.pop();
    merged[0].from = last.from - TAU;
  }
  return merged.filter((arc) => arc.tone >= 0);
}

function smoothRows(meridian, F, P, { rows = 6, slope = null, radius = null, steps = 48, error = 0.35, depth: deepest = 8, fineness = 0.1 } = {}) {
  const V = viewOf(P);
  const L = lightOf(P);
  const view = { a: dot3(F.a, V), flat: Math.hypot(dot3(F.u, V), dot3(F.v, V)), phi: Math.atan2(dot3(F.v, V), dot3(F.u, V)) };
  const light = { a: dot3(F.a, L), flat: Math.hypot(dot3(F.u, L), dot3(F.v, L)), phi: Math.atan2(dot3(F.v, L), dot3(F.u, L)) };
  const m = meridian.length;
  const faces = [];
  for (let i = 0; i < m - 1; i++) {
    const ds = meridian[i + 1][0] - meridian[i][0];
    const dr = meridian[i + 1][1] - meridian[i][1];
    const length = Math.hypot(ds, dr) || 1;
    faces.push([ds / length, -dr / length]);
  }
  const splines = slope ? null : curveRuns(meridian);
  const vertexNormal = (k, own) => {
    if (k <= 0 || k >= m - 1 || !meridian[k][2]) return own;
    const a = faces[k - 1];
    const b = faces[k];
    const x = a[0] + b[0];
    const y = a[1] + b[1];
    const length = Math.hypot(x, y) || 1;
    return [x / length, y / length];
  };
  const normalAt = (i, u) => {
    const [s0] = meridian[i];
    const [s1] = meridian[i + 1];
    const s = s0 + (s1 - s0) * u;
    const sign = Math.sign(s1 - s0) || 1;
    const g = slope ? slope(s) : splines[i] ? splines[i].slope(s) : null;
    if (g !== null) {
      const length = Math.hypot(1, g);
      return [sign / length, (-sign * g) / length];
    }
    const a = vertexNormal(i, faces[i]);
    const b = vertexNormal(i + 1, faces[i]);
    const x = a[0] + (b[0] - a[0]) * u;
    const y = a[1] + (b[1] - a[1]) * u;
    const length = Math.hypot(x, y) || 1;
    return [x / length, y / length];
  };
  const pointAt = (i, u, angle) => {
    const [s0, r0] = meridian[i];
    const [s1, r1] = meridian[i + 1];
    const s = s0 + (s1 - s0) * u;
    return iso(pointOf(F, s, radius ? radius(s) : splines && splines[i] ? splines[i].radius(s) : r0 + (r1 - r0) * u, angle), P);
  };
  const buckets = [[], [], [], [], []];
  const fine = TAU / Math.max(24, Math.min(192, steps * 1.5));
  const arcPoints = (i, u, from, to) => {
    const count = Math.max(1, Math.ceil((to - from) / fine));
    return Array.from({ length: count + 1 }, (_, index) => pointAt(i, u, from + ((to - from) * index) / count));
  };
  const centreOf = (arc, reference) => {
    let mid = (arc.from + arc.to) / 2 - reference;
    mid = ((mid % TAU) + TAU) % TAU;
    return mid;
  };
  const levelOf = (i, u) => {
    const [nr, na] = normalAt(i, u);
    const groups = new Map();
    for (const arc of labelledArcs(nr, na, view, light, faces[i][0], faces[i][1])) {
      if (!groups.has(arc.tone)) groups.set(arc.tone, []);
      groups.get(arc.tone).push(arc);
    }
    for (const list of groups.values()) list.sort((x, y) => centreOf(x, light.phi) - centreOf(y, light.phi));
    return { i, u, groups };
  };
  const same = (x, y) => {
    if (x.groups.size !== y.groups.size) return false;
    for (const [tone, list] of x.groups) if ((y.groups.get(tone) ?? []).length !== list.length) return false;
    return true;
  };
  const align = (reference, arc) => {
    const out = { ...arc };
    const centre = (reference.from + reference.to) / 2;
    while ((out.from + out.to) / 2 - centre > Math.PI) {
      out.from -= TAU;
      out.to -= TAU;
    }
    while (centre - (out.from + out.to) / 2 > Math.PI) {
      out.from += TAU;
      out.to += TAU;
    }
    return out;
  };
  const strayOf = (i, a, mid, b) => {
    let worst = 0;
    for (const [tone, list] of mid.groups) {
      list.forEach((arc, index) => {
        const lo = align(arc, a.groups.get(tone)[index]);
        const hi = align(arc, b.groups.get(tone)[index]);
        for (const key of ["from", "to"]) {
          const p = pointAt(i, mid.u, arc[key]);
          const p0 = pointAt(i, a.u, lo[key]);
          const p1 = pointAt(i, b.u, hi[key]);
          worst = Math.max(worst, Math.hypot(p[0] - (p0[0] + p1[0]) / 2, p[1] - (p0[1] + p1[1]) / 2));
        }
      });
    }
    return worst;
  };
  const sheet = (tone, levels) => {
    levels[0].groups.get(tone).forEach((arc, index) => {
      const chain = [arc];
      for (let k = 1; k < levels.length; k++) chain.push(align(chain[k - 1], levels[k].groups.get(tone)[index]));
      const last = levels.length - 1;
      const points = [...arcPoints(levels[0].i, levels[0].u, chain[0].from, chain[0].to)];
      for (let k = 1; k < last; k++) points.push(pointAt(levels[k].i, levels[k].u, chain[k].to));
      points.push(...arcPoints(levels[last].i, levels[last].u, chain[last].from, chain[last].to).reverse());
      for (let k = last - 1; k >= 1; k--) points.push(pointAt(levels[k].i, levels[k].u, chain[k].from));
      buckets[tone].push(pathOf(oriented(points), true));
    });
  };
  const overlap = (x, y) => Math.min(x.to, y.to) - Math.max(x.from, y.from);
  const strip = (tone, a, b) => {
    const below = a.groups.get(tone) ?? [];
    const above = (b.groups.get(tone) ?? []).map((arc) => ({ arc, used: false }));
    const quad = (lo, hi) => {
      const points = [...arcPoints(a.i, a.u, lo.from, lo.to), ...arcPoints(b.i, b.u, hi.from, hi.to).reverse()];
      buckets[tone].push(pathOf(oriented(points), true));
    };
    for (const arc of below) {
      let best = null;
      let score = -Infinity;
      for (const entry of above) {
        if (entry.used) continue;
        const aligned = align(arc, entry.arc);
        const value = overlap(arc, aligned);
        if (value > score) (score = value), (best = entry);
      }
      if (best && score > -0.05) {
        best.used = true;
        quad(arc, align(arc, best.arc));
      } else {
        const centre = (arc.from + arc.to) / 2;
        quad(arc, { from: centre, to: centre });
      }
    }
    for (const entry of above) {
      if (entry.used) continue;
      const centre = (entry.arc.from + entry.arc.to) / 2;
      quad({ from: centre, to: centre }, entry.arc);
    }
  };
  const k = scaleOf(P);
  let run = [];
  const flush = () => {
    const tones = [...new Set(run.flatMap((level) => [...level.groups.keys()]))].sort((x, y) => x - y);
    for (const tone of tones) {
      const countOf = (level) => (level.groups.get(tone) ?? []).length;
      let start = 0;
      for (let at = 1; at <= run.length; at++) {
        if (at < run.length && countOf(run[at - 1]) === countOf(run[at])) continue;
        if (at - 1 > start && countOf(run[start])) sheet(tone, run.slice(start, at));
        if (at < run.length) strip(tone, run[at - 1], run[at]);
        start = at;
      }
    }
    run = [];
  };
  for (let i = 0; i < m - 1; i++) {
    const ds = meridian[i + 1][0] - meridian[i][0];
    if (Math.abs(ds) < 1e-9) {
      flush();
      continue;
    }
    const count = Math.max(1, rows);
    const reach = (Math.hypot(ds, meridian[i + 1][1] - meridian[i][1]) * k) / count;
    const limit = Math.max(1, Math.min(deepest, Math.ceil(Math.log2(Math.max(2, reach / fineness)))));
    const coarse = Array.from({ length: count + 1 }, (_, index) => levelOf(i, index / count));
    run.push(coarse[0]);
    const refine = (a, b, depth) => {
      const mid = levelOf(i, (a.u + b.u) / 2);
      const split = depth < limit && (!same(a, b) || !same(a, mid) || (depth < 6 && strayOf(i, a, mid, b) > error));
      if (!split) {
        run.push(b);
        return;
      }
      refine(a, mid, depth + 1);
      refine(mid, b, depth + 1);
    };
    for (let index = 1; index <= count; index++) refine(coarse[index - 1], coarse[index], 0);
  }
  flush();
  return buckets;
}

return { smoothRows };
})();
const M20 = (() => { // lathe/torno.mjs
const { iso, pathOf } = M11;
const { TAU, hullTagged, groupsOf, oriented } = M13;
const { dot3, add3, sub3, mul3 } = M12;
const { viewOf, lightOf, scaleOf, bandOf } = M14;
const { pointOf, angleToward } = M15;
const { hullOf, stepsFor } = M16;
const { curveRuns } = M17;
const { smoothRows } = M19;
const { silhouettePoints } = M18;
// Torno: lathe e outlineOf, com a superfície e o contorno de um perfil de revolução.









function chains(edges) {
  const out = [];
  let current = null;
  for (const [a, b, keep] of edges) {
    if (!keep) {
      if (current) out.push(current);
      current = null;
      continue;
    }
    if (!current) current = [a];
    current.push(b);
  }
  if (current) out.push(current);
  if (out.length > 1 && edges[0][2] && edges[edges.length - 1][2]) {
    const last = out.pop();
    out[0] = [...last, ...out[0].slice(1)];
  }
  return out;
}

function curved(meridian, P, segments, { error = 0.2, depth = 6 } = {}) {
  const k = scaleOf(P);
  const out = [meridian[0]];
  for (let i = 0; i < meridian.length - 1; i++) {
    const curve = segments[i];
    const [s0] = meridian[i];
    const [s1] = meridian[i + 1];
    if (curve) {
      const split = (a, b, level) => {
        const mid = (a + b) / 2;
        const chord = (curve.radius(a) + curve.radius(b)) / 2;
        if (level < depth && Math.abs(curve.radius(mid) - chord) * k > error) {
          split(a, mid, level + 1);
          out.push([mid, curve.radius(mid), 1]);
          split(mid, b, level + 1);
        }
      };
      split(s0, s1, 0);
    }
    out.push(meridian[i + 1]);
  }
  return out;
}

function sideOf(meridian) {
  const m = meridian.length;
  let first = 0;
  let last = m - 1;
  if (meridian[0][1] <= 1e-9) first = 1;
  if (meridian[m - 1][1] <= 1e-9) last = m - 2;
  const side = meridian.slice(first, last + 1);
  for (let index = 1; index < side.length; index++) if (side[index][0] - side[index - 1][0] <= 1e-9) return null;
  return side.length >= 3 ? side : null;
}

function surfaceOf(side, F, P, rims, { samples = 96, slope = null } = {}) {
  const V = viewOf(P);
  const aV = dot3(F.a, V);
  const facing = Math.atan2(dot3(F.v, V), dot3(F.u, V));
  const [left, right] = silhouettePoints(side, F, P, { samples, slope });
  if (left.length <= samples || right.length <= samples) return null;
  const s0 = side[0][0];
  const s1 = side[side.length - 1][0];
  const angleOf = (point) => angleToward(F, sub3(point, add3(F.o, mul3(F.a, dot3(sub3(point, F.o), F.a)))));
  const wrap = (angle) => ((angle % TAU) + TAU) % TAU;
  const arc = (s, r, from, to, through) => {
    let sweep = wrap(to - from);
    if (wrap(through - from) > sweep) sweep -= TAU;
    const count = Math.max(4, Math.ceil(Math.abs(sweep) / (TAU / 96)), Math.ceil((Math.abs(sweep) * r * scaleOf(P)) / 1.5));
    return Array.from({ length: count + 1 }, (_, index) => iso(pointOf(F, s, r, from + (sweep * index) / count), P));
  };
  const r0 = side[0][1];
  const r1 = side[side.length - 1][1];
  const bottom = -aV > 0 ? facing + Math.PI : facing;
  const top = aV > 0 ? facing + Math.PI : facing;
  const L = left.map((point) => iso(point, P));
  const R = right.map((point) => iso(point, P));
  const topArc = arc(s1, r1, angleOf(left[left.length - 1]), angleOf(right[right.length - 1]), top);
  const bottomArc = arc(s0, r0, angleOf(right[0]), angleOf(left[0]), bottom);
  const keepFirst = rims === "all" || rims === "first" || rims === "ends";
  const keepLast = rims === "all" || rims === "last" || rims === "ends";
  const fill = pathOf([...L, ...topArc, ...[...R].reverse(), ...bottomArc], true);
  const outline = [pathOf(L), pathOf(R), keepLast ? pathOf(topArc) : "", keepFirst ? pathOf(bottomArc) : ""].join("");
  const hull = hullOf([...arc(s0, r0, 0, TAU - 1e-6, Math.PI), ...arc(s1, r1, 0, TAU - 1e-6, Math.PI), ...L, ...R]);
  let gap = 0;
  for (const p of [...L, ...R]) {
    let best = Infinity;
    for (let index = 0; index < hull.length; index++) {
      const a = hull[index];
      const b = hull[(index + 1) % hull.length];
      const dx = b[0] - a[0];
      const dy = b[1] - a[1];
      const t = Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy || 1)));
      best = Math.min(best, Math.hypot(p[0] - a[0] - dx * t, p[1] - a[1] - dy * t));
    }
    gap = Math.max(gap, best);
  }
  return { fill, outline, gap };
}

const softOf = (meridian) => meridian.some((point) => Boolean(point[2]));

function lathe(meridian, F, P, { steps = stepsFor(Math.max(...meridian.map(([, r]) => r)), 32, 160, scaleOf(P)), fill = "auto", hole = null, bevel = 0, rims = "all", smooth: asked = "auto", waist = 0.5 } = {}) {
  const smooth = asked === "auto" ? softOf(meridian) : asked;
  const V = viewOf(P);
  const L = lightOf(P);
  const aV = dot3(F.a, V);
  const uV = dot3(F.u, V);
  const vV = dot3(F.v, V);
  const aL = dot3(F.a, L);
  const uL = dot3(F.u, L);
  const vL = dot3(F.v, L);
  const n = steps;
  const m = meridian.length;
  const rings = meridian.map(([s, r]) => Array.from({ length: n }, (_, j) => iso(pointOf(F, s, r, (j / n) * TAU), P)));
  const midC = [];
  const midS = [];
  for (let j = 0; j < n; j++) {
    const t = ((j + 0.5) / n) * TAU;
    midC.push(Math.cos(t));
    midS.push(Math.sin(t));
  }
  const seen = [];
  const tones = [];
  for (let i = 0; i < m - 1; i++) {
    const ds = meridian[i + 1][0] - meridian[i][0];
    const dr = meridian[i + 1][1] - meridian[i][1];
    const length = Math.hypot(ds, dr);
    const row = [];
    const toneRow = [];
    for (let j = 0; j < n; j++) {
      if (length < 1e-9) {
        row.push(false);
        toneRow.push(0);
        continue;
      }
      const facing = ds * (midC[j] * uV + midS[j] * vV) - dr * aV;
      const lit = (ds * (midC[j] * uL + midS[j] * vL) - dr * aL) / length;
      row.push(facing > 1e-6 * length);
      toneRow.push(bandOf(lit));
    }
    seen.push(row);
    tones.push(toneRow);
  }
  const buckets = [[], [], [], [], []];
  const all = [];
  const smoothed = smooth ? smoothRows(meridian, F, P, { steps, ...(smooth === true ? {} : smooth) }) : null;
  if (smoothed) smoothed.forEach((list, tone) => list.forEach((d) => (buckets[tone].push(d), all.push(d))));
  for (let i = 0; i < m - 1; i++) {
    if (smoothed && Math.abs(meridian[i + 1][0] - meridian[i][0]) > 1e-9) continue;
    const keys = seen[i].map((visible, j) => (visible ? tones[i][j] : null));
    for (const { key, run } of groupsOf(keys)) {
      const a = run.map((j) => rings[i][j]);
      const b = run.map((j) => rings[i + 1][j]).reverse();
      const d = pathOf(oriented([...a, ...b]), true);
      buckets[key].push(d);
      all.push(d);
    }
  }
  const phi = Math.atan2(vV, uV);
  const flatV = Math.hypot(uV, vV);
  const rangeOf = (i) => {
    const ds = meridian[i + 1][0] - meridian[i][0];
    const dr = meridian[i + 1][1] - meridian[i][1];
    if (Math.hypot(ds, dr) < 1e-9) return null;
    if (Math.abs(ds) < 1e-9 || flatV < 1e-9) return -dr * aV > 1e-9 ? [-1, 1] : null;
    const c = (dr * aV) / (ds * flatV);
    return ds > 0 ? [Math.max(-1, c), 1] : [-1, Math.min(1, c)];
  };
  const arcsOf = (ranges) => {
    if (ranges.some((range) => !range)) return [];
    const lo = Math.max(...ranges.map(([x]) => x));
    const hi = Math.min(...ranges.map(([, x]) => x));
    if (lo >= hi - 1e-9) return [];
    const near = Math.acos(Math.min(1, hi));
    const far = Math.acos(Math.max(-1, lo));
    if (hi >= 1 && lo <= -1) return [[phi, phi + TAU]];
    if (hi >= 1) return [[phi - far, phi + far]];
    if (lo <= -1) return [[phi + near, phi + TAU - near]];
    return [[phi + near, phi + far], [phi - far, phi - near]];
  };
  const along = (s, r, [from, to]) => {
    const count = Math.max(2, Math.ceil(((to - from) * Math.max(r, 1e-6) * scaleOf(P)) / 1.5));
    return Array.from({ length: count + 1 }, (_, j) => iso(pointOf(F, s, r, from + ((to - from) * j) / count), P));
  };
  const creases = [];
  for (let i = 1; i < m - 1; i++) {
    if (meridian[i][2] || meridian[i][1] <= 1e-9) continue;
    for (const arc of arcsOf([rangeOf(i - 1), rangeOf(i)])) creases.push(pathOf(along(meridian[i][0], meridian[i][1], arc)));
  }
  let bevelPath = "";
  if (bevel > 0 && m > 2) {
    const ends = [];
    if (meridian[0][1] <= 1e-9) ends.push([0, 1, 1]);
    if (meridian[m - 1][1] <= 1e-9) ends.push([m - 2, m - 2, m - 3]);
    for (const [cap, rim, side] of ends) {
      const [s, r] = meridian[rim];
      for (const arc of arcsOf([rangeOf(cap), rangeOf(side)])) {
        const inset = along(s, Math.max(0, r - bevel), arc);
        const edge = along(s, r, arc);
        bevelPath += pathOf([edge[0], ...inset.slice(1, -1), edge[edge.length - 1]]);
      }
    }
  }
  const { fill: fillPath, outline } = outlineOf(meridian, F, P, { steps: n, fill, hole, rims, smooth, waist, all });
  return {
    fill: fillPath,
    outline,
    crease: creases.join(""),
    top: buckets[4].join(""),
    shades: buckets.slice(0, 4).map((list) => list.join("")),
    bevel: bevelPath || undefined,
    seamless: Boolean(smoothed),
  };
}

function shapeOf(meridian, P, smooth) {
  if (smooth && smooth.radius) {
    const segments = meridian.slice(0, -1).map(([s0], index) => (Math.abs(meridian[index + 1][0] - s0) > 1e-9 ? { radius: smooth.radius, slope: smooth.slope } : null));
    return curved(meridian, P, segments);
  }
  const curves = smooth && !smooth.slope ? curveRuns(meridian) : null;
  return curves && curves.some(Boolean) ? curved(meridian, P, curves) : meridian;
}

function outlineOf(meridian, F, P, { steps = stepsFor(Math.max(...meridian.map(([, r]) => r)), 32, 160, scaleOf(P)), fill = "auto", hole = null, rims = "all", smooth: asked = "auto", waist = 0.5, all = null } = {}) {
  const smooth = asked === "auto" ? softOf(meridian) : asked;
  const n = steps;
  let fillPath;
  let outline;
  const shape = shapeOf(meridian, P, smooth);
  const side = fill === "auto" || fill === "surface" ? sideOf(shape) : null;
  const surface = side && !hole ? surfaceOf(side, F, P, rims, { slope: smooth && smooth.slope ? smooth.slope : null }) : null;
  if (surface && (fill === "surface" || surface.gap > waist)) {
    fillPath = surface.fill;
    outline = surface.outline;
  } else if (fill === "hull" || fill === "auto" || fill === "surface") {
    const most = Math.max(...shape.map(([, r]) => r));
    const around = Math.min(320, Math.max(n, Math.ceil((TAU * most * scaleOf(P)) / 1.5)));
    const tagged = [];
    shape.forEach(([s, r], index) => {
      for (let j = 0; j < around; j++) tagged.push({ p: iso(pointOf(F, s, r, (j / around) * TAU), P), ring: index });
    });
    const hull = hullTagged(tagged);
    const holePath = hole ? pathOf(Array.from({ length: around }, (_, j) => iso(pointOf(F, hole[0], hole[1], (j / around) * TAU), P)), true) : "";
    fillPath = pathOf(hull.map(({ p }) => p), true) + holePath;
    if (rims === "all") {
      outline = pathOf(hull.map(({ p }) => p), true) + holePath;
    } else {
      const keepFirst = rims === "first" || rims === "ends";
      const keepLast = rims === "last" || rims === "ends";
      const edges = hull.map((a, index) => {
        const b = hull[(index + 1) % hull.length];
        const same = a.ring === b.ring;
        const keep = !same || (a.ring !== 0 && a.ring !== shape.length - 1) || (keepFirst && a.ring === 0) || (keepLast && a.ring === shape.length - 1);
        return [a.p, b.p, keep];
      });
      outline = chains(edges).map((points) => pathOf(points)).join("") + holePath;
    }
  } else {
    fillPath = all ? all.join("") : (({ shades, top }) => [...shades, top].join(""))(lathe(meridian, F, P, { steps: n, fill: "auto", hole, rims, smooth, waist }));
    outline = "";
  }
  return { fill: fillPath, outline };
}

return { softOf, lathe, outlineOf };
})();
const M21 = (() => { // lathe/solidos.mjs
const { iso, pathOf } = M11;
const { TAU, BANDS, groupsOf, oriented } = M13;
const { dot3, add3, sub3, mul3, unit3 } = M12;
const { viewOf, lightOf, bandOf } = M14;
const { frameAlong, pointOf, radialOf } = M15;
const { hullOf, runsOf, stepsFor } = M16;
// Sólidos prontos: sela, prisma, estádio e esfera.






const saddleStart = (R, a, angle) => Math.sqrt(Math.max(0, R * R - a * a * Math.cos(angle) ** 2));

function saddleOf(F, a, start, s1, P, { steps = stepsFor(a, 24) } = {}) {
  const V = viewOf(P);
  const L = lightOf(P);
  const n = steps;
  const angle = (j) => (j / n) * TAU;
  const from = typeof start === "function" ? start : (t) => saddleStart(start, a, t);
  const low = Array.from({ length: n }, (_, j) => iso(pointOf(F, from(angle(j)), a, angle(j)), P));
  const high = Array.from({ length: n }, (_, j) => iso(pointOf(F, s1, a, angle(j)), P));
  const seen = [];
  const tones = [];
  for (let j = 0; j < n; j++) {
    const normal = radialOf(F, angle(j + 0.5));
    seen.push(dot3(normal, V) > 1e-6);
    tones.push(bandOf(dot3(normal, L)));
  }
  const buckets = [[], [], [], [], []];
  const fills = [];
  const keys = seen.map((visible, j) => (visible ? tones[j] : null));
  for (const { key, run } of groupsOf(keys)) {
    const d = pathOf(oriented([...run.map((j) => low[j]), ...run.map((j) => high[j]).reverse()]), true);
    buckets[key].push(d);
    fills.push(d);
  }
  const capSeen = dot3(F.a, V) > 1e-6;
  if (capSeen) {
    const d = pathOf(oriented(high), true);
    buckets[bandOf(dot3(F.a, L))].push(d);
    fills.push(d);
  }
  const front = runsOf(seen);
  const back = runsOf(seen.map((visible) => !visible));
  const edges = [];
  for (const run of front) {
    edges.push(pathOf(run.map((j) => low[j])));
    edges.push(pathOf([low[run[0]], high[run[0]]]), pathOf([low[run[run.length - 1]], high[run[run.length - 1]]]));
  }
  for (const run of capSeen ? back : front) edges.push(pathOf(run.map((j) => high[j])));
  return {
    fill: fills.join(""),
    outline: edges.join(""),
    crease: capSeen ? front.map((run) => pathOf(run.map((j) => high[j]))).join("") : "",
    top: buckets[4].join(""),
    shades: buckets.slice(0, 4).map((list) => list.join("")),
    seamless: true,
  };
}

function prismOf(F, polygon, s0, s1, P) {
  const V = viewOf(P);
  const L = lightOf(P);
  const ring = oriented(polygon.map(([x, y]) => [x, y]));
  const n = ring.length;
  const at = (s, [x, y]) => [F.o[0] + F.a[0] * s + F.u[0] * x + F.v[0] * y, F.o[1] + F.a[1] * s + F.u[1] * x + F.v[1] * y, F.o[2] + F.a[2] * s + F.u[2] * x + F.v[2] * y];
  const low = ring.map((point) => iso(at(s0, point), P));
  const high = ring.map((point) => iso(at(s1, point), P));
  const buckets = [[], [], [], [], []];
  const seen = [];
  for (let j = 0; j < n; j++) {
    const [x0, y0] = ring[j];
    const [x1, y1] = ring[(j + 1) % n];
    const normal = unit3(add3(mul3(F.u, y1 - y0), mul3(F.v, -(x1 - x0))));
    const visible = dot3(normal, V) > 1e-6;
    seen.push(visible);
    if (visible) buckets[bandOf(dot3(normal, L))].push(pathOf([low[j], low[(j + 1) % n], high[(j + 1) % n], high[j]], true));
  }
  const ends = [[s0, low, mul3(F.a, -1)], [s1, high, F.a]];
  const capSeen = ends.map(([, , normal]) => dot3(normal, V) > 1e-6);
  ends.forEach(([, points, normal], index) => capSeen[index] && buckets[bandOf(dot3(normal, L))].push(pathOf(points, true)));
  const hull = hullOf([...low, ...high]);
  const creases = [];
  for (let j = 0; j < n; j++) {
    const prev = (j - 1 + n) % n;
    const a = [ring[j][0] - ring[prev][0], ring[j][1] - ring[prev][1]];
    const b = [ring[(j + 1) % n][0] - ring[j][0], ring[(j + 1) % n][1] - ring[j][1]];
    const sharp = (a[0] * b[0] + a[1] * b[1]) / (Math.hypot(a[0], a[1]) * Math.hypot(b[0], b[1]) || 1) < Math.cos(0.35);
    if (sharp && seen[j] && seen[prev]) creases.push(pathOf([low[j], high[j]]));
    ends.forEach(([, points], index) => capSeen[index] && seen[j] && creases.push(pathOf([points[j], points[(j + 1) % n]])));
  }
  return { fill: pathOf(hull, true), outline: pathOf(hull, true), crease: creases.join(""), top: buckets[4].join(""), shades: buckets.slice(0, 4).map((list) => list.join("")) };
}

function stadium(r0, r1, offset, steps = 32) {
  const circle = (cx, r) => Array.from({ length: steps }, (_, index) => [cx + r * Math.cos((index / steps) * TAU), r * Math.sin((index / steps) * TAU)]);
  return hullOf([...circle(0, r0), ...circle(offset, r1)]);
}

function sphereOf(centre, r, P, { steps = 48 } = {}) {
  const V = viewOf(P);
  const L = lightOf(P);
  const F = frameAlong(centre, V);
  const flat = (p) => iso(p, P);
  const rim = Array.from({ length: steps }, (_, index) => pointOf(F, 0, r, (index / steps) * TAU));
  const outline = pathOf(rim.map(flat), true);
  const capOf = (threshold) => {
    const points = rim.filter((p) => dot3(sub3(p, centre), L) / r >= threshold).map(flat);
    const c = dot3(L, V);
    const LF = frameAlong(add3(centre, mul3(L, r * threshold)), L);
    const radius = r * Math.sqrt(Math.max(0, 1 - threshold * threshold));
    for (let index = 0; index < steps; index++) {
      const p = pointOf(LF, 0, radius, (index / steps) * TAU);
      if (dot3(sub3(p, centre), V) >= -1e-9) points.push(flat(p));
    }
    if (c > threshold) points.push(flat(add3(centre, mul3(V, r))));
    return points.length > 2 ? pathOf(hullOf(points), true) : "";
  };
  const shades = [outline, capOf(BANDS[1]), capOf(BANDS[2]), capOf(BANDS[3])];
  return { fill: outline, outline, crease: "", top: "", shades, seamless: true };
}

return { saddleStart, saddleOf, prismOf, stadium, sphereOf };
})();
const M22 = (() => { // lathe/arcos.mjs
const { iso, pathOf } = M11;
const { TAU } = M13;
const { dot3, unit3 } = M12;
const { viewOf, scaleOf } = M14;
const { pointOf, radialOf } = M15;
const { stepsFor } = M16;
// Arcos, círculos, raios, nervuras, pontos e orientação de face.






function arcOf(F, s, r, P, { slope = 0, steps = 0, least = 0.02, inward = false, from = 0, to = TAU, trim = 0.3 } = {}) {
  const V = viewOf(P);
  const aV = dot3(F.a, V);
  const uV = dot3(F.u, V);
  const vV = dot3(F.v, V);
  const flat = Math.hypot(uV, vV);
  const phi = Math.atan2(vV, uV);
  const norm = Math.hypot(1, slope);
  const full = Math.abs(to - from - TAU) < 1e-9;
  let visible;
  if (flat < 1e-9) visible = (inward ? slope * aV / norm > least : -slope * aV / norm > least) ? [[phi, phi + TAU]] : [];
  else if (inward) {
    const c = (slope * aV - least * norm) / flat;
    visible = c <= -1 ? [] : c >= 1 ? [[phi, phi + TAU]] : [[phi + Math.acos(c), phi + TAU - Math.acos(c)]];
  } else {
    const c = (least * norm + slope * aV) / flat;
    visible = c >= 1 ? [] : c <= -1 ? [[phi, phi + TAU]] : [[phi - Math.acos(c), phi + Math.acos(c)]];
  }
  const at = (t) => iso(pointOf(F, s, r, t), P);
  const speed = (t) => {
    const a = at(t - 1e-3);
    const b = at(t + 1e-3);
    return Math.hypot(b[0] - a[0], b[1] - a[1]) / 2e-3;
  };
  const count = (span) => Math.max(2, Math.ceil(((steps || stepsFor(r, 24, 112)) * span) / TAU), Math.ceil((span * r * scaleOf(P)) / 1.5));
  const out = [];
  for (const [a, b] of visible) {
    if (b - a >= TAU - 1e-9 && full) {
      const n = count(TAU);
      out.push(pathOf(Array.from({ length: n }, (_, j) => at(from + (j / n) * TAU)), true));
      continue;
    }
    const pieces = full ? [[a, b, true, true]] : [];
    if (!full)
      for (let turn = -2; turn <= 2; turn++) {
        const lo = Math.max(a + turn * TAU, from);
        const hi = Math.min(b + turn * TAU, to);
        if (hi - lo > 1e-6) pieces.push([lo, hi, lo > from + 1e-9, hi < to - 1e-9]);
      }
    for (const [lo, hi, limb0, limb1] of pieces) {
      const span = hi - lo;
      const cut0 = limb0 && trim > 0 ? Math.min(span * 0.25, trim / (speed(lo) || 1)) : 0;
      const cut1 = limb1 && trim > 0 ? Math.min(span * 0.25, trim / (speed(hi) || 1)) : 0;
      const t0 = lo + cut0;
      const t1 = hi - cut1;
      const n = count(t1 - t0);
      out.push(pathOf(Array.from({ length: n + 1 }, (_, j) => at(t0 + ((t1 - t0) * j) / n))));
    }
  }
  return out.join("");
}

function circleOf(F, s, r, P, steps = 0) {
  const count = steps || stepsFor(r, 24, 112);
  return pathOf(
    Array.from({ length: count }, (_, j) => iso(pointOf(F, s, r, (j / count) * TAU), P)),
    true,
  );
}

function spokesOf(F, s, r0, r1, count, P, { phase = 0, sweep = 0, from = 0, to = TAU } = {}) {
  const lines = [];
  for (let index = 0; index < count; index++) {
    const t = phase + from + (index / count) * (to - from);
    lines.push(pathOf([iso(pointOf(F, s, r0, t), P), iso(pointOf(F, s, r1, t + sweep), P)]));
  }
  return lines.join("");
}

const fadeOf = (value, [lo, hi]) => {
  const t = Math.max(0, Math.min(1, (value - lo) / (hi - lo || 1e-9)));
  return t * t * (3 - 2 * t);
};

function ribsOf(F, s0, s1, r, count, P, { phase = 0, twist = 0, least = 0.12, r1 = r, fade = null, seams = false } = {}) {
  const V = viewOf(P);
  const lines = [];
  const slope = (r1 - r) / (s1 - s0 || 1);
  for (let index = 0; index < count; index++) {
    const t = phase + (index / count) * TAU;
    const radial = radialOf(F, t + twist / 2);
    const normal = unit3([radial[0] - slope * F.a[0], radial[1] - slope * F.a[1], radial[2] - slope * F.a[2]]);
    const facing = dot3(normal, V);
    const d = pathOf([iso(pointOf(F, s0, r, t), P), iso(pointOf(F, s1, r1, t + twist), P)]);
    if (fade) {
      const alpha = fadeOf(facing, fade);
      if (alpha > 0.02) lines.push({ d, alpha });
      continue;
    }
    if (facing < least) continue;
    lines.push(d);
  }
  const ends = seams ? [seams === "last" ? "" : arcOf(F, s0, r, P, { slope }), seams === "first" ? "" : arcOf(F, s1, r1, P, { slope })].filter(Boolean) : [];
  if (fade) return [...lines, ...ends.map((d) => ({ d, alpha: 1 }))];
  return [...lines, ...ends].join("");
}

function dotsOf(F, s, r, count, P, { phase = 0, slope = 0, least = 0.08, all = false, fade = null } = {}) {
  const V = viewOf(P);
  const out = [];
  for (let index = 0; index < count; index++) {
    const t = phase + (index / count) * TAU;
    let alpha = 1;
    if (!all) {
      const radial = radialOf(F, t);
      const normal = unit3([radial[0] - slope * F.a[0], radial[1] - slope * F.a[1], radial[2] - slope * F.a[2]]);
      const facing = dot3(normal, V);
      if (fade) alpha = fadeOf(facing, fade);
      else if (facing < least) continue;
      if (alpha <= 0.02) continue;
    }
    const [x, y] = iso(pointOf(F, s, r, t), P);
    out.push(alpha < 1 ? [x, y, alpha] : [x, y]);
  }
  return out;
}

function facingOf(F, angle, P, slope = 0) {
  const radial = radialOf(F, angle);
  return dot3(unit3([radial[0] - slope * F.a[0], radial[1] - slope * F.a[1], radial[2] - slope * F.a[2]]), viewOf(P));
}

return { arcOf, circleOf, spokesOf, fadeOf, ribsOf, dotsOf, facingOf };
})();
const M23 = (() => { // lathe/perfis.mjs
const { iso, pathOf } = M11;
const { TAU } = M13;
const { scaleOf } = M14;
const { pointOf } = M15;
const { hullOf, stepsFor } = M16;
const { lathe } = M20;
const { silhouettePoints } = M18;
const { profileCurve } = M17;
const { arcOf } = M22;
// Sólidos e faixas gerados a partir de um perfil: disco, anel, haste e cortes por faixa.









const disc = (s0, s1, r, F, P, options = {}) =>
  lathe(
    [
      [s0, 0],
      [s0, r],
      [s1, r],
      [s1, 0],
    ],
    F,
    P,
    { steps: stepsFor(r, 32, 160, scaleOf(P)), ...options },
  );

const ringBand = (s0, s1, rIn, rOut, F, P, options = {}) =>
  lathe(
    [
      [s0, rIn],
      [s0, rOut],
      [s1, rOut],
      [s1, rIn],
    ],
    F,
    P,
    { steps: stepsFor(rOut, 32, 160, scaleOf(P)), hole: null, ...options },
  );

function solidOf(profile, F, P, options = {}) {
  const first = profile[0];
  const last = profile[profile.length - 1];
  const meridian = [[first[0], 0], ...profile, [last[0], 0]];
  const most = Math.max(...profile.map(([, r]) => r));
  return lathe(meridian, F, P, { steps: stepsFor(most, 32, 160, scaleOf(P)), ...options });
}

function bandOfProfile(profile, F, P, options = {}) {
  const most = Math.max(...profile.map(([, r]) => r));
  return lathe(profile, F, P, { steps: stepsFor(most, 32, 160, scaleOf(P)), ...options });
}

function bandsOf(profile, cuts, F, P, { slope = null, radius = null, steps, rows = 6, rims = "none", fill = "auto", samples = 72 } = {}) {
  const curve = profileCurve(profile, { slope, radius });
  const { s0, s1 } = curve;
  const stops = [s0, ...[...new Set(cuts)].filter((s) => s > s0 + 1e-6 && s < s1 - 1e-6).sort((a, b) => a - b), s1];
  const most = Math.max(...profile.map(([, r]) => r), ...stops.map((s) => curve.radius(s)));
  const n = steps ?? stepsFor(most, 32, 160, scaleOf(P));
  const knot = (s, soft) => (soft ? [s, curve.radius(s), 1] : [s, curve.radius(s)]);
  const meridianOf = (a, b) => [knot(a), ...profile.filter(([s]) => s > a + 1e-9 && s < b - 1e-9).map(([s, , soft]) => knot(s, soft)), knot(b)];
  const shading = { slope: curve.slope, radius: curve.radius, rows };
  const bands = stops.slice(0, -1).map((a, index) => {
    const b = stops[index + 1];
    const meridian = meridianOf(a, b);
    const paths = lathe(meridian, F, P, { steps: n, rims, fill, smooth: shading });
    const seam = index < stops.length - 2 ? arcOf(F, b, curve.radius(b), P, { slope: curve.slope(b) }) : "";
    return { s0: a, s1: b, profile: meridian, paths: { ...paths, outline: "" }, outline: paths.outline, seam };
  });
  const sidesOf = (from = s0, to = s1, { ends = false, count = samples } = {}) => {
    const span = [Math.max(s0, from), Math.min(s1, to)];
    const dense = Array.from({ length: count + 1 }, (_, index) => knot(span[0] + ((span[1] - span[0]) * index) / count, true));
    const points = silhouettePoints(dense, F, P, { samples: count, slope: curve.slope, radius: curve.radius });
    const rim = (s) => arcOf(F, s, curve.radius(s), P, { slope: curve.slope(s) });
    const rimsAt = ends === true ? span : ends === "first" ? [span[0]] : ends === "last" ? [span[1]] : [];
    return { d: points.map((list) => pathOf(list.map((point) => iso(point, P)))).join("") + rimsAt.map(rim).join(""), points };
  };
  return { bands, sidesOf, radius: curve.radius, slope: curve.slope, steps: n, s0, s1 };
}

function rodOut(F, from, to, r, P, { stroke = 0, ...options } = {}) {
  const pad = 0.35 / scaleOf(P);
  const ring = (s) => Array.from({ length: 48 }, (_, index) => iso(pointOf(F, s, r + pad, (index / 48) * TAU), P));
  return {
    paths: disc(from - stroke, to, r, F, P, options),
    clip: pathOf(hullOf([...ring(from), ...ring(to + stroke)]), true),
    from,
    to,
    stroke,
    reach: to + stroke,
  };
}

return { disc, ringBand, solidOf, bandOfProfile, bandsOf, rodOut };
})();
const M24 = (() => { // lathe/caminho.mjs
const { dot3, add3, sub3, mul3, len3, unit3, cross3, lerp3 } = M12;
const { viewOf } = M14;
// Caminhos 3D: filete, reamostragem, comprimento, corte e leitura ao longo do caminho.


function fillet(points, radius, perArc = 7, { P = null, tube = 0, margin = 1.3 } = {}) {
  if (points.length < 3) return points.map((p) => [...p]);
  const V = P && tube ? viewOf(P) : null;
  const out = [[...points[0]]];
  for (let index = 1; index < points.length - 1; index++) {
    const prev = points[index - 1];
    const here = points[index];
    const next = points[index + 1];
    const into = unit3(sub3(here, prev));
    const outOf = unit3(sub3(next, here));
    const cosTurn = Math.max(-1, Math.min(1, dot3(into, outOf)));
    const turn = Math.acos(cosTurn);
    if (turn < 1e-3) {
      out.push([...here]);
      continue;
    }
    const room = Math.min(len3(sub3(here, prev)), len3(sub3(next, here))) * 0.48;
    const facing = V ? Math.abs(dot3(unit3(cross3(into, outOf)), V)) : 1;
    const bend = V ? Math.max(radius, (margin * tube) / (0.7 * Math.max(0.05, facing) ** 2)) : radius;
    const cut = Math.min(bend * Math.tan(turn / 2), room);
    const a = sub3(here, mul3(into, cut));
    const b = add3(here, mul3(outOf, cut));
    const count = Math.max(2, Math.ceil((perArc * turn) / (Math.PI / 2)));
    for (let step = 0; step <= count; step++) {
      const t = step / count;
      const p0 = lerp3(a, here, t);
      const p1 = lerp3(here, b, t);
      out.push(lerp3(p0, p1, t));
    }
  }
  out.push([...points[points.length - 1]]);
  return out;
}

function resample(points, spacing) {
  const lengths = [0];
  for (let index = 1; index < points.length; index++) lengths.push(lengths[index - 1] + len3(sub3(points[index], points[index - 1])));
  const total = lengths[lengths.length - 1];
  const count = Math.max(1, Math.round(total / spacing));
  const out = [];
  let segment = 1;
  for (let step = 0; step <= count; step++) {
    const at = (total * step) / count;
    while (segment < points.length - 1 && lengths[segment] < at) segment++;
    const a = points[segment - 1];
    const b = points[segment];
    const span = lengths[segment] - lengths[segment - 1] || 1;
    out.push(lerp3(a, b, Math.min(1, Math.max(0, (at - lengths[segment - 1]) / span))));
  }
  return out;
}

function pathLength(points) {
  let total = 0;
  for (let index = 1; index < points.length; index++) total += len3(sub3(points[index], points[index - 1]));
  return total;
}

function cut(points, from, to) {
  const out = [];
  let walked = 0;
  for (let index = 1; index < points.length; index++) {
    const a = points[index - 1];
    const b = points[index];
    const length = len3(sub3(b, a));
    const start = walked;
    const end = walked + length;
    if (end >= from && start <= to) {
      const t0 = Math.max(0, (from - start) / (length || 1));
      const t1 = Math.min(1, (to - start) / (length || 1));
      if (!out.length) out.push(lerp3(a, b, t0));
      out.push(lerp3(a, b, t1));
    }
    walked = end;
  }
  return out;
}

const pointAlong = (points, at) => {
  const piece = cut(points, at, at + 0.01);
  return piece[0] ?? points[points.length - 1];
};

function tangentAlong(points, at) {
  const total = pathLength(points);
  const a = pointAlong(points, Math.max(0, at - 0.6));
  const b = pointAlong(points, Math.min(total, at + 0.6));
  return unit3(sub3(b, a));
}

return { fillet, resample, pathLength, cut, pointAlong, tangentAlong };
})();
const M25 = Object.assign({}, M12, M14, M15, M16, M17, M18, M20, M21, M22, M23, M24); // lathe.mjs
const L = Object.assign({}, M11, M25);
L.TERMOS = ["rust","sqlite","postgres","postgresql","mysql","sql","api","apis","webhook","webhooks","endpoint","endpoints","rest","graphql","json","yaml","xml","token","tokens","hmac","sha","ssl","tls","dns","http","https","sse","websocket","localhost","loopback","proxy","servidor","servidores","backend","frontend","back-end","front-end","crate","npm","node","docker","kubernetes","deploy","commit","git","github","repositorio","repositório","framework","stack","sdk","cli","mcp","n8n","workflow","workflows","prompt","prompts","llm","claude","anthropic","openai","banco de dados","migracao","migração","migracoes","migrações","cache","hash","query","script","scripts","sandbox","i18n","cloud api","ci/cd","refactor","refatorar","refatoração","bug","bugs","debug","log","logs","idempotente","idempotência","assinatura do webhook"];
// ---- biblioteca/textos.js ----
// Textos fixos da página de apresentação, em pt-BR e en. A página nunca escreve texto que não esteja aqui ou em `conteudo`.
// `conteudo.idioma` escolhe o dicionário; idioma desconhecido cai no inglês. Chave sem tradução devolve a própria chave (nunca quebra).
(function (L) {
  const TEXTOS = {
    "pt-BR": {
      plano: "Plano de projeto", plano_interno: "Plano de projeto, visão interna", sim: "Simulação com dados de exemplo. O sistema final pode diferir.",
      menu: "Seções", pular: "Ir para o conteúdo", imprimir: "Imprimir",
      visao: "Visão geral", entregas: "O que você recebe", etapas: "Etapas", linha: "Linha do tempo",
      voce: "O que precisamos de você", decisoes: "Decisões e riscos", acoes: "Ações",
      riscos: "Riscos", decisoes_t: "Decisões", aberta: "a definir", decidida: "decidida", ate: "até",
      periodo: "Período", etapa: "Etapa", etapas_n: "etapas", dias: "dias", dia_de: "Dia {n} de {total}",
      marcos: "Marcos", fig: "Fig", figura_erro: "Esta figura não pôde ser desenhada.", codigo_nao_carregou: "O código desta figura não carregou.", figura_erro_det: "Detalhe",
      st_prevista: "prevista", st_andamento: "em andamento", st_concluida: "concluída",
      comeca: "começa em {d}", hoje: "hoje", nada: "Nada a informar nesta seção.",
      nav_dica: "Teclas: [ e ] trocam de seção, Home e End vão ao começo e ao fim.",
      gerado: "Gerado em {d}", nivel: "Nível", publico: "Público", cliente: "Cliente", interno: "Interno",
      nivel_rapido: "Rápido", nivel_padrao: "Padrão", nivel_completo: "Completo", nivel_epico: "Épico",
      inicio: "Início", fim: "Fim", progresso: "Dia do plano", sem_figura: "Sem figura nesta etapa.",
    },
    en: {
      plano: "Project plan", plano_interno: "Project plan, internal view", sim: "Simulation with sample data. The final system may differ.",
      menu: "Sections", pular: "Skip to content", imprimir: "Print",
      visao: "Overview", entregas: "What you receive", etapas: "Stages", linha: "Timeline",
      voce: "What we need from you", decisoes: "Decisions and risks", acoes: "Actions",
      riscos: "Risks", decisoes_t: "Decisions", aberta: "to be decided", decidida: "decided", ate: "by",
      periodo: "Period", etapa: "Stage", etapas_n: "stages", dias: "days", dia_de: "Day {n} of {total}",
      marcos: "Milestones", fig: "Fig", figura_erro: "This figure could not be drawn.", codigo_nao_carregou: "This figure's code did not load.", figura_erro_det: "Detail",
      st_prevista: "planned", st_andamento: "in progress", st_concluida: "done",
      comeca: "starts {d}", hoje: "today", nada: "Nothing to report in this section.",
      nav_dica: "Keys: [ and ] switch section, Home and End jump to the start and the end.",
      gerado: "Generated {d}", nivel: "Level", publico: "Audience", cliente: "Client", interno: "Internal",
      nivel_rapido: "Quick", nivel_padrao: "Standard", nivel_completo: "Complete", nivel_epico: "Epic",
      inicio: "Start", fim: "End", progresso: "Plan day", sem_figura: "No figure for this stage.",
    },
  };
  const dicionario = (idioma) => TEXTOS[idioma] || TEXTOS[String(idioma || "").startsWith("pt") ? "pt-BR" : "en"];
  // t("dia_de", "pt-BR", { n: 3, total: 20 }) -> "Dia 3 de 20"
  L.t = (chave, idioma, parametros) => {
    const texto = dicionario(idioma)[chave] ?? chave;
    return parametros ? texto.replace(/\{(\w+)\}/g, (_, nome) => (parametros[nome] ?? "")) : texto;
  };
  L.idiomaDe = (idioma) => (dicionario(idioma) === TEXTOS["pt-BR"] ? "pt-BR" : "en");
  // yyyy-mm-dd -> "26 out" / "Oct 26", em UTC para a data não andar com o fuso
  L.dataCurta = (iso, idioma) => {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(String(iso || ""))) return "";
    const d = new Date(iso + "T00:00:00Z");
    return new Intl.DateTimeFormat(L.idiomaDe(idioma), { day: "numeric", month: "short", timeZone: "UTC" }).format(d).replace(/\.$/, "");
  };
  L.TEXTOS = TEXTOS;
})(L);

// ---- biblioteca/laco.js ----
// Movimento calmo das figuras: um laço de quadros por figura, só enquanto ela está à vista, mais as duas curvas de seguir um alvo.
(function (L) {
  const mediaCalmo = () => (typeof matchMedia === "function" ? matchMedia("(prefers-reduced-motion: reduce)") : { matches: false });
  L.calmo = () => mediaCalmo().matches;

  // seguir exponencial: tau em segundos, calmo salta direto ao alvo
  L.segue = (valor, alvo, dt, tau, calmo) => (calmo ? alvo : valor + (alvo - valor) * (1 - Math.exp(-dt / tau)));

  // mola com passos pequenos (estado = { x, v }); devolve true enquanto ainda se move
  L.mola = (estado, alvo, dt, rigidez, amortecimento, calmo) => {
    if (calmo) { estado.x = alvo; estado.v = 0; return false; }
    for (let i = 0; i < 4; i++) {
      estado.v += ((alvo - estado.x) * rigidez - estado.v * amortecimento) * dt / 4;
      estado.x += estado.v * dt / 4;
    }
    if (Math.abs(alvo - estado.x) < 0.005 && Math.abs(estado.v) < 0.005) { estado.x = alvo; estado.v = 0; return false; }
    return true;
  };

  // passo(dt, calmo) devolve true para continuar. O laço para fora da tela (lê a ÚLTIMA entrada do observador) e quando o passo diz que acabou;
  // acordar() o retoma depois de uma mudança de alvo. Sem IntersectionObserver (testes) considera a figura à vista.
  L.laco = (palco, passo) => {
    let quadro = 0, ultimo = 0, visivel = typeof IntersectionObserver !== "function";
    const volta = (agora) => {
      const dt = ultimo ? Math.min((agora - ultimo) / 1000, 1 / 30) : 1 / 60;
      ultimo = agora;
      quadro = visivel && passo(dt, L.calmo()) ? requestAnimationFrame(volta) : 0;
    };
    const acordar = () => { if (!quadro && visivel) { ultimo = 0; quadro = requestAnimationFrame(volta); } };
    if (!visivel) {
      new IntersectionObserver((entradas) => { visivel = entradas[entradas.length - 1].isIntersecting; if (visivel) acordar(); }, { rootMargin: "120px 0px" }).observe(palco);
    } else acordar();
    return { acordar };
  };
})(L);

// ---- biblioteca/auditoria.js ----
// Auditoria do público cliente e a impressão (sha256) do conteúdo que cada figura explica. L.TERMOS vem de termos-tecnicos.json (o empacotador o injeta).
(function (L) {
  // ---- sha256 síncrono em JS puro (SubtleCrypto é assíncrono e não existe fora de contexto seguro) ----
  const K = new Uint32Array([
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
  ]);
  const utf8 = (texto) => {
    const bytes = [];
    for (const c of unescape(encodeURIComponent(texto))) bytes.push(c.charCodeAt(0));
    return bytes;
  };
  const sha256 = (texto) => {
    const bytes = utf8(texto), bits = bytes.length * 8;
    bytes.push(0x80);
    while (bytes.length % 64 !== 56) bytes.push(0);
    for (let i = 7; i >= 0; i--) bytes.push(i >= 4 ? 0 : (bits >>> (i * 8)) & 255);
    const h = new Uint32Array([0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]);
    const w = new Uint32Array(64), giro = (x, n) => (x >>> n) | (x << (32 - n));
    for (let o = 0; o < bytes.length; o += 64) {
      for (let i = 0; i < 16; i++) w[i] = (bytes[o + i * 4] << 24) | (bytes[o + i * 4 + 1] << 16) | (bytes[o + i * 4 + 2] << 8) | bytes[o + i * 4 + 3];
      for (let i = 16; i < 64; i++) {
        const s0 = giro(w[i - 15], 7) ^ giro(w[i - 15], 18) ^ (w[i - 15] >>> 3), s1 = giro(w[i - 2], 17) ^ giro(w[i - 2], 19) ^ (w[i - 2] >>> 10);
        w[i] = (w[i - 16] + s0 + w[i - 7] + s1) | 0;
      }
      let [a, b, c, d, e, f, g, hh] = h;
      for (let i = 0; i < 64; i++) {
        const t1 = (hh + (giro(e, 6) ^ giro(e, 11) ^ giro(e, 25)) + ((e & f) ^ (~e & g)) + K[i] + w[i]) | 0;
        const t2 = ((giro(a, 2) ^ giro(a, 13) ^ giro(a, 22)) + ((a & b) ^ (a & c) ^ (b & c))) | 0;
        hh = g; g = f; f = e; e = (d + t1) | 0; d = c; c = b; b = a; a = (t1 + t2) | 0;
      }
      h[0] += a; h[1] += b; h[2] += c; h[3] += d; h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
    }
    return Array.from(h, (x) => x.toString(16).padStart(8, "0")).join("");
  };
  // JSON com as chaves em ordem: o mesmo conteúdo dá sempre a mesma impressão
  const estavel = (v) => (v && typeof v === "object" ? (Array.isArray(v) ? "[" + v.map(estavel).join(",") + "]" : "{" + Object.keys(v).sort().map((k) => JSON.stringify(k) + ":" + estavel(v[k])).join(",") + "}") : JSON.stringify(v));
  L.sha256 = sha256;
  L.impressao = (valor) => sha256(estavel(valor));

  // ---- auditoria: nenhuma palavra técnica nem horas por plano no texto que o cliente lê ----
  const textos = (v, caminho, saida) => {
    if (typeof v === "string") saida.push([caminho, v]);
    else if (Array.isArray(v)) v.forEach((x, i) => textos(x, `${caminho}[${i}]`, saida));
    else if (v && typeof v === "object") for (const k of Object.keys(v)) if (k !== "interno") textos(v[k], caminho ? `${caminho}.${k}` : k, saida);
    return saida;
  };
  const escapa = (s) => s.replace(/[.*+?^${}()|[\]\\\/]/g, "\\$&");
  let regras = null;
  const monta = () => (regras ||= (L.TERMOS || []).map((t) => [t, new RegExp(`(?<![\\p{L}\\p{N}_-])${escapa(t.toLowerCase())}(?![\\p{L}\\p{N}_-])`, "u")]));
  const HORAS = /(?<![\p{L}\p{N}_])\d+([.,]\d+)?\s*(h|hs|horas|hours|hrs)(?![\p{L}\p{N}_])/iu;
  // devolve [{ termo, campo, trecho }]; vazio = limpo. O campo `interno` não conta (só existe no público interno).
  L.auditarCliente = (conteudo) => {
    const achados = [];
    for (const [campo, texto] of textos(conteudo, "", [])) {
      const baixo = texto.toLowerCase();
      for (const [termo, re] of monta()) {
        const m = re.exec(baixo);
        if (m) achados.push({ termo, campo, trecho: texto.slice(Math.max(0, m.index - 20), m.index + termo.length + 20) });
      }
      const h = HORAS.exec(texto);
      if (h) achados.push({ termo: "horas por plano", campo, trecho: texto.slice(Math.max(0, h.index - 20), h.index + h[0].length + 20) });
    }
    return achados;
  };
})(L);

// ---- biblioteca/pecas.js ----
// Peças reutilizáveis do anatomy: cada uma devolve um trecho de SVG (ou { svg, ... }), desenhado uma vez no ponto em que a peça está parada.
// Todas recebem a projeção P (L.fitProjection) e medidas no mundo do kit: x e y no chão, z para cima, 1 unidade ~ 1 px.
// O que se move sai em um <g data-peca="nome"> e a figura o desloca com L.translateAlong; nada aqui anima.
(function (L) {
  const { slabOf, cylinder, solidSvg, lineSvg, dotsSvg, iso, planOutline, insetPlan, corners, ring, haloOf, topTicks, knurl, radialTicks, lineOnTop, segment, depthOf } = L;
  const sol = (paths, estilo) => solidSvg(paths, estilo);
  const porProfundidade = (P, pontos) => [...pontos].sort((a, b) => depthOf([a[0], a[1], 0], P) - depthOf([b[0], b[1], 0], P));

  // Base: placa com quatro pés, sulco, parafusos nos cantos e uma régua com marcas. O topo da placa fica em z = 0.
  const base = (P, plano, { h = 12, pe = 5, regua = true } = {}) => {
    const out = [`<path class="iso-halo" d="${haloOf(plano, -h - pe, h, P)}"/>`];
    for (const [x, y] of porProfundidade(P, corners(plano, 22))) {
      out.push(sol(cylinder(x, y, 7, -h - pe, 1.6, P, 28), { tone: "lo" }), sol(cylinder(x, y, 5.4, -h - pe + 1.6, pe - 1.6, P, 28), { tone: "mid" }));
    }
    out.push(sol(slabOf(plano, -h, h, P, 8, 1.6), { tone: "mid" }), lineSvg(planOutline(insetPlan(plano, 5), 0, P), { tone: "lo" }));
    const parafusos = corners(plano, 10);
    out.push(lineSvg(parafusos.map(([x, y]) => ring(x, y, 2.2, 0, P, 16)).join(""), { tone: "lo" }), dotsSvg(parafusos.map(([x, y]) => iso([x, y, 0], P)), { size: 0.5 }));
    if (regua) {
      const faixa = { x: plano.x + 18, y: plano.y + plano.d - 15, w: plano.w - 36, d: 7, r: 1 };
      out.push(sol(slabOf(faixa, 0, 0.6, P, 3), { tone: "lo", crease: "none" }));
      const marcas = topTicks(faixa.x + 4, faixa.x + faixa.w - 4, 4, 5, faixa.y + 0.8, 0.6, [2, 3.6], P);
      out.push(lineSvg(marcas.minor, { tone: "lo" }), lineSvg(marcas.major, { tone: "mid" }));
      out.push(dotsSvg([iso([faixa.x + 2, faixa.y + 3.5, 0.6], P), iso([faixa.x + faixa.w - 2, faixa.y + 3.5, 0.6], P)], { size: 0.45 }));
    }
    return out.join("");
  };

  // Dois trilhos ao longo de x (em cada y da lista) com batentes nas pontas, sobre z = 0.
  const trilhos = (P, { de, ate, ys, d = 5, h = 3, topes = true }) => {
    const out = [];
    for (const y of ys) {
      out.push(sol(slabOf({ x: de, y, w: ate - de, d, r: 1 }, 0, h, P, 3, 0.4), { tone: "mid" }));
      out.push(dotsSvg([de + 6, ate - 6].map((x) => iso([x, y + d / 2, h], P)), { size: 0.4 }));
    }
    if (topes) {
      const y0 = Math.min(...ys) - 3, y1 = Math.max(...ys) + d + 3;
      for (const x of [de - 5, ate]) {
        out.push(sol(slabOf({ x, y: y0, w: 5, d: y1 - y0, r: 1.2 }, 0, 7, P, 3, 0.5), { tone: "mid" }));
        out.push(dotsSvg([iso([x + 2.5, y0 + 5, 7], P), iso([x + 2.5, y1 - 5, 7], P)], { size: 0.45 }));
      }
    }
    return out.join("");
  };

  // Ficha em pé: lâmina fina em x (a face larga olha para +x), com linhas de texto e uma aba no topo. Devolve o <g> da ficha.
  // A peça "lit" é ligada de fora: el.querySelector(".iso-solid").toggleAttribute("data-lit").
  const ficha = (P, { x, y0, y1, z = 0, alto, esp = 3, aba = null, linhas = 5 }) => {
    const out = [sol(slabOf({ x, y: y0, w: esp, d: y1 - y0, r: 0.6 }, z, alto, P, 2, 0.3), { tone: "hi" })];
    const xf = x + esp, passo = (alto - 16) / Math.max(linhas, 1);
    for (let i = 0; i < linhas; i++) {
      const za = z + alto - 9 - i * passo, fim = i === 0 ? y1 - 6 : y1 - 6 - ((i * 7) % 9);
      out.push(lineSvg(segment([xf, y0 + 5, za], [xf, fim, za], P), { tone: i === 0 ? "mid" : "faint", free: true }));
    }
    if (aba) {
      out.push(sol(slabOf({ x, y: aba.y, w: esp, d: aba.w, r: 0.4 }, z + alto, aba.h, P, 2), { tone: "mid" }));
      out.push(dotsSvg([iso([xf, aba.y + aba.w / 2, z + alto + aba.h / 2], P)], { size: 0.45, tone: "hi" }));
    }
    return `<g data-peca="ficha">${out.join("")}</g>`;
  };

  // Botão serrilhado (cilindro com ranhuras e um risco no topo).
  const botao = (P, { x, y, z = 0, r = 6, h = 4 }) => [
    sol(cylinder(x, y, r, z, h, P, 32), { tone: "mid" }),
    lineSvg(knurl(x, y, r, z, z + h, 36, P), { tone: "lo" }),
    lineSvg(lineOnTop([x, y], [x + r, y], z + h, P), { tone: "hi" }),
    dotsSvg([iso([x, y, z + h], P)], { size: 0.5, tone: "hi" }),
  ].join("");

  // Mostrador: caixa redonda com anel, marcas radiais e uma agulha. agulha(volta) devolve o `d` da agulha para volta em [0, 1) e a figura o grava.
  const mostrador = (P, { x, y, z = 0, r = 20, h = 10 }) => {
    const topo = z + h, out = [sol(cylinder(x, y, r, z, h, P, 64, 0.8), { tone: "hi", crease: "lo" })];
    out.push(lineSvg(knurl(x, y, r, topo - 3.4, topo, 84, P, 0.3), { tone: "lo" }), lineSvg(ring(x, y, r - 2.2, topo, P, 64), { tone: "lo" }));
    const marcas = radialTicks(x, y, r - 3, 50, 5, topo, [1.3, 2.8], P);
    out.push(lineSvg(marcas.minor, { tone: "lo" }), lineSvg(marcas.major, { tone: "mid" }), lineSvg(ring(x, y, 1.6, topo, P, 16), { tone: "mid" }));
    const agulha = (volta) => {
      const a = volta * Math.PI * 2;
      return lineOnTop([x - Math.sin(a) * 3.4, y + Math.cos(a) * 3.4], [x + Math.sin(a) * (r - 4), y - Math.cos(a) * (r - 4)], topo + 0.2, P);
    };
    return { svg: out.join(""), agulha, topo };
  };

  // Plaquinha de identificação: lâmina baixa com três linhas de regra e dois pontos.
  const placa = (P, { x, y, w = 24, d = 10, z = 0 }) => [
    sol(slabOf({ x, y, w, d, r: 1 }, z, 1.2, P, 3), { tone: "lo", crease: "none" }),
    ...[0.28, 0.5, 0.72].map((t, i) => lineSvg(lineOnTop([x + 3, y + d * t], [x + w - 3 - i * 4, y + d * t], z + 1.2, P), { tone: "faint", free: true })),
    dotsSvg([iso([x + 1.6, y + 1.6, z + 1.2], P), iso([x + w - 1.6, y + d - 1.6, z + 1.2], P)], { size: 0.4 }),
  ].join("");

  L.pecas = { base, trilhos, ficha, botao, mostrador, placa };
})(L);

// ---- biblioteca/secoes.js ----
// Seções da apresentação como texto HTML puro (sem DOM): cada função recebe o `conteudo` e devolve uma string. Nada é inventado aqui:
// os números vêm das datas de `conteudo`, o resto é o que o agente escreveu. O que vem de fora passa sempre por esc().
(function (L) {
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const dia = (iso) => (/^\d{4}-\d{2}-\d{2}$/.test(String(iso || "")) ? Date.UTC(+iso.slice(0, 4), +iso.slice(5, 7) - 1, +iso.slice(8, 10)) / 86400000 : NaN);
  const num = (n) => String(n).padStart(2, "0");
  const lista = (v) => (Array.isArray(v) ? v : []);

  // período do plano e onde a data do documento cai nele
  const periodo = (c) => {
    const ini = Math.min(...lista(c.etapas).map((e) => dia(e.inicio)).filter(Number.isFinite)), fim = Math.max(...lista(c.etapas).map((e) => dia(e.fim)).filter(Number.isFinite));
    if (!Number.isFinite(ini) || !Number.isFinite(fim)) return null;
    const hoje = dia(c.data), total = fim - ini + 1;
    const n = Number.isFinite(hoje) ? Math.min(total, Math.max(0, hoje - ini + 1)) : 0;
    return { ini, fim, total, n, pct: Math.round((n / total) * 100), hoje: Number.isFinite(hoje) ? hoje : null };
  };
  // `etapa.situacao` (prevista, andamento, concluida), quando vem no conteúdo, vale mais que as datas: o blueprint é quem sabe o que já começou
  const situacao = (e, c) => {
    if (["prevista", "andamento", "concluida"].includes(e.situacao)) return e.situacao;
    const hoje = dia(c.data);
    if (!Number.isFinite(hoje)) return "prevista";
    return hoje > dia(e.fim) ? "concluida" : hoje >= dia(e.inicio) ? "andamento" : "prevista";
  };
  const doDia = (n, idioma) => L.dataCurta(new Date(n * 86400000).toISOString().slice(0, 10), idioma);

  const secao = (id, n, titulo, corpo, extra = "") =>
    `<section class="ap-sec" id="s-${id}" aria-labelledby="h-${id}" data-sec="${id}"${extra}><header class="ap-sec-cab"><span class="ap-sec-n" aria-hidden="true">${num(n)}</span><h2 id="h-${id}">${esc(titulo)}</h2><span class="ap-sec-regua" aria-hidden="true"></span></header><div class="ap-sec-corpo">${corpo}</div></section>`;
  // a figura da seção ao lado do texto (abaixo em tela estreita); duas figuras na mesma seção ficam uma após a outra
  const lado = (corpo, fig) => (fig ? `<div class="ap-com-fig"><div class="ap-com-texto">${corpo}</div><div class="ap-com-fig-col">${fig}</div></div>` : corpo);
  const vazio = (idioma) => `<p class="ap-vazio">${esc(L.t("nada", idioma))}</p>`;
  const datas = (a, b, idioma) => (a && b ? `${esc(L.dataCurta(a, idioma))} &rarr; ${esc(L.dataCurta(b, idioma))}` : "");

  const visao = (c, cartoes, n) => {
    const p = periodo(c), idioma = c.idioma, t = (k, par) => L.t(k, idioma, par);
    const resumo = lista(c.resumo).map((f) => `<li>${esc(f)}</li>`).join("");
    const kpis = p ? [
      [t("periodo"), `${doDia(p.ini, idioma)} &rarr; ${doDia(p.fim, idioma)}`],
      [t("etapas"), String(lista(c.etapas).length)],
      [t("entregas"), String(lista(c.entregas).length)],
      [t("progresso"), p.hoje === null || p.hoje < p.ini ? t("comeca", { d: doDia(p.ini, idioma) }) : t("dia_de", { n: p.n, total: p.total })],
    ].map(([r, v]) => `<div class="ap-kpi"><span>${esc(r)}</span><b>${v}</b></div>`).join("") : "";
    const barra = p ? `<div class="ap-barra" role="img" aria-label="${esc(t("progresso"))}: ${p.pct}%"><i style="width:${p.pct}%"></i></div>` : "";
    return secao("visao", n, t("visao"), `<div class="ap-visao"><div class="ap-visao-texto"><h3 class="ap-titulo">${esc(c.titulo)}</h3>${c.subtitulo ? `<p class="ap-sub">${esc(c.subtitulo)}</p>` : ""}${c.cliente ? `<p class="ap-cliente">${esc(c.cliente)}</p>` : ""}<ul class="ap-resumo">${resumo}</ul></div><div class="ap-visao-fig">${cartoes.capa || ""}</div></div><div class="ap-kpis">${kpis}</div>${barra}`);
  };

  const entregas = (c, n, fig) => {
    const l = lista(c.entregas);
    return secao("entregas", n, L.t("entregas", c.idioma), lado(l.length ? `<div class="ap-grade">${l.map((e, i) => `<article class="ap-cartao"><span class="ap-num">${num(i + 1)}</span><h3>${esc(e.titulo)}</h3><p>${esc(e.texto)}</p></article>`).join("")}</div>` : vazio(c.idioma), fig));
  };

  const etapas = (c, cartoes, n, extras = "", fig = "") => {
    const idioma = c.idioma, l = lista(c.etapas);
    const corpo = l.map((e, i) => {
      const st = situacao(e, c), marcos = lista(e.marcos);
      const m = marcos.length ? `<ul class="ap-marcos" aria-label="${esc(L.t("marcos", idioma))}">${marcos.map((k) => `<li><i aria-hidden="true"></i><b>${esc(L.dataCurta(k.data, idioma))}</b><span>${esc(k.nome)}</span></li>`).join("")}</ul>` : "";
      return `<article class="ap-etapa" id="e-${esc(e.id || i + 1)}" data-st="${st}"><header><span class="ap-num">${num(i + 1)}</span><h3>${esc(e.titulo)}</h3><span class="ap-datas">${datas(e.inicio, e.fim, idioma)}</span><span class="ap-tag" data-st="${st}">${esc(L.t("st_" + st, idioma))}</span></header><p>${esc(e.texto)}</p>${m}${e.figura && cartoes[e.figura] ? `<div class="ap-etapa-fig">${cartoes[e.figura]}</div>` : ""}</article>`;
    }).join("");
    return secao("etapas", n, L.t("etapas", idioma), lado((l.length ? corpo : vazio(idioma)) + (extras ? `<div class="ap-etapa-fig ap-extras">${extras}</div>` : ""), fig));
  };

  const linha = (c, n, fig = "") => {
    const p = periodo(c), idioma = c.idioma;
    if (!p) return secao("linha", n, L.t("linha", idioma), lado(vazio(idioma), fig));
    const pos = (d) => (((d - p.ini) / p.total) * 100).toFixed(2);
    const eixo = [];
    const passo = Math.max(1, Math.ceil(p.total / 6));
    for (let d = p.ini; d <= p.fim; d += passo) eixo.push(`<span style="left:${pos(d)}%">${esc(doDia(d, idioma))}</span>`);
    const linhas = lista(c.etapas).map((e, i) => {
      const a = dia(e.inicio), b = dia(e.fim);
      if (!Number.isFinite(a) || !Number.isFinite(b)) return "";
      const marcos = lista(e.marcos).map((k) => (Number.isFinite(dia(k.data)) ? `<b class="ap-g-marco" style="left:${pos(dia(k.data) + 0.5)}%" title="${esc(k.nome)}"></b>` : "")).join("");
      return `<li class="ap-g-linha"><span class="ap-g-nome"><i>${num(i + 1)}</i>${esc(e.titulo)}</span><div class="ap-g-faixa"><i class="ap-g-barra" data-st="${situacao(e, c)}" style="left:${pos(a)}%;width:${(((b - a + 1) / p.total) * 100).toFixed(2)}%"></i>${marcos}</div></li>`;
    }).join("");
    const hoje = p.hoje !== null && p.hoje >= p.ini && p.hoje <= p.fim ? `<i class="ap-g-hoje" style="--p:${((p.hoje + 0.5 - p.ini) / p.total).toFixed(4)}" title="${esc(L.t("hoje", idioma))}"></i>` : "";
    return secao("linha", n, L.t("linha", idioma), lado(`<div class="ap-gantt" style="--passo:${((passo / p.total) * 100).toFixed(3)}%"><div class="ap-g-eixo" aria-hidden="true"><span></span><div>${eixo.join("")}</div></div><ol class="ap-g-lista">${linhas}</ol>${hoje}</div>`, fig));
  };

  const voce = (c, n, fig = "") => {
    const l = lista(c.precisamosDeVoce), idioma = c.idioma;
    return secao("voce", n, L.t("voce", idioma), lado(l.length ? `<div class="ap-grade ap-grade-voce">${l.map((x, i) => `<article class="ap-cartao ap-pedido"><span class="ap-num">${num(i + 1)}</span><h3>${esc(x.titulo)}</h3><p>${esc(x.texto)}</p>${x.ate ? `<span class="ap-ate">${esc(L.t("ate", idioma))} ${esc(L.dataCurta(x.ate, idioma))}</span>` : ""}</article>`).join("")}</div>` : vazio(idioma), fig));
  };

  const decisoes = (c, n, figD = "", figR = "") => {
    const idioma = c.idioma, d = lista(c.decisoes), r = lista(c.riscos);
    const colD = d.length ? d.map((x) => `<article class="ap-item" data-estado="${x.estado === "decidida" ? "decidida" : "aberta"}"><h4>${esc(x.titulo)}</h4><p>${esc(x.texto)}</p><span class="ap-tag" data-estado="${x.estado === "decidida" ? "decidida" : "aberta"}">${esc(L.t(x.estado === "decidida" ? "decidida" : "aberta", idioma))}</span></article>`).join("") : vazio(idioma);
    const colR = r.length ? r.map((x) => `<article class="ap-item"><h4>${esc(x.titulo)}</h4><p>${esc(x.texto)}</p></article>`).join("") : vazio(idioma);
    return secao("decisoes", n, L.t("decisoes", idioma), `<div class="ap-duas"><div><h3 class="ap-sub-t">${esc(L.t("decisoes_t", idioma))}</h3>${colD}${figD ? `<div class="ap-fig-col">${figD}</div>` : ""}</div><div><h3 class="ap-sub-t">${esc(L.t("riscos", idioma))}</h3>${colR}${figR ? `<div class="ap-fig-col">${figR}</div>` : ""}</div></div>`);
  };

  // só no público interno
  const acoes = (c, n, fig = "") => {
    const l = lista(c.interno && c.interno.acoes);
    return secao("acoes", n, L.t("acoes", c.idioma), lado(l.length ? l.map((a) => `<article class="ap-item"><h4><span class="ap-num">${esc(a.numero)}</span> ${esc(a.nome)}</h4><ul class="ap-planos">${lista(a.planos).map((p) => `<li>${esc(p)}</li>`).join("")}</ul></article>`).join("") : vazio(c.idioma), fig));
  };

  L.secoes = { esc, dia, periodo, situacao, visao, entregas, etapas, linha, voce, decisoes, acoes };
})(L);

// ---- biblioteca/fragmento.js ----
// A seção ativa e a URL: o editor do app recarrega o visor depois de cada alteração salva e quer voltar à mesma seção.
// Fragmentos aceitos (o primeiro é o que a página escreve): #visao  #entregas  #etapas  #etapas-3 (a 3ª etapa)  #linha  #voce  #decisoes  #acoes,
// também #s-<seção> (o id real das seções), #e-<id da etapa> e os nomes do conteúdo: #recebe (entregas), #precisamos (voce), #riscos (decisoes).
// Seção ou etapa que deixou de existir cai na mais próxima, sem erro; fragmento que não é nosso não faz nada.
// Do app para a página: window.postMessage({ tipo: "ap-ir", secao: "etapas" [, etapa: 3] }, "*") rola até lá. Da página para o app (quando há um pai):
// postMessage({ tipo: "ap-estado", secao, etapa (1 a n ou null), fragmento, via }) quando a seção visível muda; `via` diz como o fragmento foi gravado
// ("replaceState", "replace" ou "nenhum"; em iframe sandbox sem allow-same-origin o primeiro pode falhar e a página cai no segundo sem quebrar).
(function (L) {
  const ORDEM = ["visao", "entregas", "etapas", "linha", "voce", "decisoes", "acoes"];
  const ALIAS = { recebe: "entregas", precisamos: "voce", riscos: "decisoes" };
  const preso = (n, a, b) => Math.min(b, Math.max(a, n));
  const maisProxima = (nome, secoes) => {
    const i = ORDEM.indexOf(nome);
    return { secao: secoes.reduce((m, s) => (Math.abs(ORDEM.indexOf(s) - i) < Math.abs(ORDEM.indexOf(m) - i) ? s : m), secoes[0]) };
  };

  // fragmento -> { secao, etapa? } (etapa é índice a partir de 0) ou null
  const resolver = (fragmento, secoes, idsEtapas = []) => {
    const f = String(fragmento == null ? "" : fragmento).replace(/^#/, "").trim();
    if (!f || !secoes.length) return null;
    const porId = /^e-(.+)$/.exec(f);
    if (porId) {
      const i = idsEtapas.indexOf(porId[1]);
      return i >= 0 ? { secao: "etapas", etapa: i } : maisProxima("etapas", secoes);
    }
    const m = /^(?:s-)?([a-z]+)(?:-(\d+))?$/.exec(f.toLowerCase());
    const nome = m && (ALIAS[m[1]] || m[1]);
    if (!nome || !ORDEM.includes(nome)) return null;
    const alvo = secoes.includes(nome) ? { secao: nome } : maisProxima(nome, secoes);
    if (alvo.secao === "etapas" && m[2] && idsEtapas.length) alvo.etapa = preso(+m[2], 1, idsEtapas.length) - 1;
    return alvo;
  };
  const texto = (r) => (r.etapa != null ? `etapas-${r.etapa + 1}` : r.secao);

  const ligar = (raiz) => {
    const secoes = [...raiz.querySelectorAll(".ap-sec[data-sec]")], etapas = [...raiz.querySelectorAll(".ap-etapa")];
    const ids = secoes.map((s) => s.dataset.sec), idsEtapas = etapas.map((e) => String(e.id).replace(/^e-/, ""));
    const elDe = (r) => (r.etapa != null && etapas[r.etapa]) || secoes[ids.indexOf(r.secao)];
    let escrito = null, gravado = null, livre = false, mexeu = false, adiar = 0, atual = null, manual = null;
    // `manual`: o alvo pedido (fragmento, ap-ir, menu) vale até o usuário mexer; a última seção, por exemplo, nunca chega ao topo e o observador diria a anterior
    const ir = (r, instantaneo) => {
      const el = r && elDe(r);
      if (!el) return;
      manual = r;
      clearTimeout(adiar);
      adiar = setTimeout(() => publica(r), 200);
      el.scrollIntoView({ block: "start", behavior: instantaneo ? "instant" : "auto" });
    };

    const escreve = (f) => {
      escrito = f;
      try { history.replaceState(null, "", "#" + f); return "replaceState"; } catch (e) { /* sandbox: tenta o outro */ }
      try { location.replace("#" + f); return "replace"; } catch (e) { return "nenhum"; }
    };
    const publica = (r) => {
      const f = texto(r);
      if (!livre || f === gravado) return;
      gravado = f;
      const via = escreve(f);
      try { if (window.parent && window.parent !== window) window.parent.postMessage({ tipo: "ap-estado", secao: r.secao, etapa: r.etapa == null ? null : r.etapa + 1, fragmento: f, via }, "*"); } catch (e) { /* sem pai */ }
    };

    // a seção (e a etapa) à vista: a primeira de cada lista, no mesmo recorte do menu
    if (typeof IntersectionObserver === "function") {
      const sS = new Set(), sE = new Set();
      const io = new IntersectionObserver((es) => {
        for (const e of es) (e.target.classList.contains("ap-sec") ? sS : sE)[e.isIntersecting ? "add" : "delete"](e.target);
        const sec = secoes.find((s) => sS.has(s));
        if (!sec) return;
        atual = { secao: sec.dataset.sec };
        const i = etapas.findIndex((x) => sE.has(x));
        if (atual.secao === "etapas" && i >= 0) atual.etapa = i;
        if (manual) return;
        clearTimeout(adiar);
        adiar = setTimeout(() => publica(atual), 150); // a rolagem suave passa por várias seções: só a que ficou conta
      }, { rootMargin: "-20% 0px -65% 0px" });
      [...secoes, ...etapas].forEach((x) => io.observe(x));
    }

    // abrir já na seção do fragmento (sem rolagem suave) e conferir de novo quando as figuras acabarem de mudar o layout
    const inicial = resolver(location.hash, ids, idsEtapas);
    const solta = () => { livre = true; if (manual || atual) publica(manual || atual); };
    ["wheel", "touchstart", "keydown", "pointerdown"].forEach((ev) => addEventListener(ev, () => { mexeu = true; manual = null; }, { passive: true }));
    if (inicial) {
      ir(inicial, true);
      requestAnimationFrame(() => ir(inicial, true));
      setTimeout(() => { if (!mexeu) ir(inicial, true); solta(); }, 350);
    } else setTimeout(solta, 350);

    addEventListener("hashchange", () => { if (location.hash.slice(1) !== escrito) ir(resolver(location.hash, ids, idsEtapas), false); });
    addEventListener("message", (e) => {
      const d = e.data;
      if (!d || typeof d !== "object" || d.tipo !== "ap-ir" || typeof d.secao !== "string") return;
      if (e.source !== window.parent && e.source !== window) return;
      ir(resolver(d.secao + (Number.isInteger(d.etapa) && !/-\d+$/.test(d.secao) ? "-" + d.etapa : ""), ids, idsEtapas), false);
    });
    // os links do menu não empilham histórico no iframe do app: rolam e gravam o fragmento no lugar
    raiz.addEventListener("click", (e) => {
      const a = e.target.closest("[data-nav]");
      if (!a) return;
      e.preventDefault();
      ir({ secao: a.dataset.nav }, false);
    });
  };

  L.fragmento = { ORDEM, ALIAS, resolver, texto, ligar };
})(L);

// ---- biblioteca/casca.js ----
// A casca da apresentação: monta a interface (barra superior, navegação por seções, painéis) a partir de `conteudo` e roda as figuras.
// L.casca.html(conteudo, figuras) devolve { html, vivos } sem tocar no DOM (serve aos testes e à prévia); L.casca.montar(raiz, ...) a põe na página.
// Cada figura vai para o painel da sua seção (`figura.secao`: visao, recebe, etapas, linha, precisamos, decisoes, riscos, acoes), ao lado do texto; a que uma etapa
// cita (`etapas[].figura`) fica dentro da etapa; a sem seção e sem etapa vai ao fim de Etapas. Uma figura que lança erro vira um cartão de erro; o resto da página segue.
(function (L) {
  const { esc } = L.secoes, S = L.secoes;

  // `erro` nulo = o bloco do código da figura nem carregou (erro de sintaxe: `codigo` não chegou como função); senão, erro ao desenhar
  const cartaoDeErro = (f, n, idioma, erro) =>
    `<figure class="iso-figure ap-fig-erro" role="alert"><div class="iso-plate"><p class="ap-erro-t"><b>${esc(L.t("fig", idioma))} ${n}</b> ${esc(f.titulo || f.id)}</p>${erro ? `<p>${esc(L.t("figura_erro", idioma))}</p><p class="ap-erro-d">${esc(L.t("figura_erro_det", idioma))}: <code>${esc(erro.message ? erro.message : String(erro))}</code></p>` : `<p>${esc(L.t("codigo_nao_carregou", idioma))}</p>`}</div></figure>`;

  // cada figura vira um cartão do kit (Fig, título, dica, leitura, chaves, legenda); `vivos` guarda o que ganha movimento depois
  const rodar = (c, figuras) => {
    const cartoes = {}, vivos = [];
    figuras.forEach((f, i) => {
      const n = i + 1;
      if (typeof f.codigo !== "function") { cartoes[f.id] = cartaoDeErro(f, n, c.idioma, null); return; }
      try {
        const r = f.codigo(L, c);
        cartoes[f.id] = L.plateHtml({
          fig: `${L.t("fig", c.idioma)} ${n}`, title: f.titulo, hint: f.dica, readout: f.leitura || "",
          keys: (f.chaves || []).map((k) => ({ mark: k.marca, label: k.rotulo })), caption: f.legenda,
          body: `<div class="iso-stage" tabindex="0" role="group" aria-label="${esc(f.rotulo || f.titulo)}" data-figura="${esc(f.id)}">${r.svg}</div>`,
        });
        if (typeof r.vivo === "function") vivos.push({ id: f.id, vivo: r.vivo });
      } catch (e) { cartoes[f.id] = cartaoDeErro(f, n, c.idioma, e); }
    });
    return { cartoes, vivos };
  };

  // o nome da figura no conteúdo -> o id da seção da página (riscos mora na seção Decisões e riscos); desconhecido = sem seção
  const SECAO_DA_FIGURA = { visao: "visao", recebe: "entregas", entregas: "entregas", etapas: "etapas", linha: "linha", precisamos: "voce", voce: "voce", decisoes: "decisoes", riscos: "riscos", acoes: "acoes" };
  const secaoDaFigura = (f) => SECAO_DA_FIGURA[f && f.secao];
  const usadasPorEtapas = (c) => new Set((c.etapas || []).map((e) => e.figura).filter(Boolean));

  // a figura da capa (Visão geral): a de secao "visao", senão a de id "capa", senão a primeira sem seção; uma que uma etapa cita fica na etapa
  const capaDe = (c, figuras) => {
    const livres = figuras.filter((f) => !usadasPorEtapas(c).has(f.id));
    return (livres.find((f) => secaoDaFigura(f) === "visao") || livres.find((f) => f.id === "capa") || livres.find((f) => !f.secao) || {}).id;
  };

  const secoesDe = (c) => ["visao", c.entregas && c.entregas.length ? "entregas" : null, "etapas", "linha", c.precisamosDeVoce && c.precisamosDeVoce.length ? "voce" : null, "decisoes", c.publico === "interno" ? "acoes" : null].filter(Boolean);

  const html = (c, figuras = []) => {
    c = { ...c, idioma: L.idiomaDe(c.idioma) };
    const idioma = c.idioma, t = (k, p) => L.t(k, idioma, p);
    const { cartoes, vivos } = rodar(c, figuras);
    const ids = secoesDe(c), capa = capaDe(c, figuras), usadas = usadasPorEtapas(c);
    // onde cada figura cai: a da etapa, dentro dela; a da capa e as de seção, no painel da seção; o resto (e o que cairia numa seção que não existe), no fim de Etapas
    const porSecao = {}, soltas = [];
    for (const f of figuras) {
      if (usadas.has(f.id)) continue;
      const sec = f.id === capa ? "visao" : secaoDaFigura(f), existe = sec && ids.includes(sec === "riscos" ? "decisoes" : sec);
      if (!existe) soltas.push(cartoes[f.id]);
      else (porSecao[sec] = porSecao[sec] || [])[f.id === capa ? "unshift" : "push"](cartoes[f.id]);
    }
    const j = (k) => (porSecao[k] || []).join(""), extras = soltas.join("");
    const montar = { visao: (n) => S.visao(c, { capa: j("visao") }, n), entregas: (n) => S.entregas(c, n, j("entregas")), etapas: (n) => S.etapas(c, cartoes, n, extras, j("etapas")), linha: (n) => S.linha(c, n, j("linha")), voce: (n) => S.voce(c, n, j("voce")), decisoes: (n) => S.decisoes(c, n, j("decisoes"), j("riscos")), acoes: (n) => S.acoes(c, n, j("acoes")) };
    const corpo = ids.map((id, i) => montar[id](i + 1)).join("");
    const nav = ids.map((id, i) => `<li><a href="#s-${id}" data-nav="${id}"><i>${String(i + 1).padStart(2, "0")}</i>${esc(t(id))}</a></li>`).join("");
    const nivel = c.nivel ? `<span class="ap-etiqueta">${esc(t("nivel"))}: ${esc(t("nivel_" + c.nivel))}</span>` : "";
    const quem = c.publico === "interno" ? "interno" : "cliente";
    const pagina = `<div class="ap-ia iso" data-theme="dark" lang="${esc(idioma)}"><a class="ap-pular" href="#conteudo">${esc(t("pular"))}</a>
<header class="ap-topo"><div class="ap-migalhas"><span>${esc(t(quem))}</span><i aria-hidden="true">/</i><span>${esc(t(quem === "interno" ? "plano_interno" : "plano"))}</span></div><div class="ap-titulo-caixa"><b>${esc(c.titulo)}</b>${c.cliente ? `<span>${esc(c.cliente)}</span>` : ""}</div><div class="ap-topo-acoes">${nivel}<button class="ap-btn" type="button" data-acao="imprimir">${esc(t("imprimir"))}</button></div></header>
<div class="ap-corpo"><nav class="ap-nav" aria-label="${esc(t("menu"))}"><p class="ap-rot">${esc(t("menu"))}</p><ol>${nav}</ol><p class="ap-dica">${esc(t("nav_dica"))}</p></nav><main id="conteudo" class="ap-principal" tabindex="-1">${corpo}</main></div>
<footer class="ap-rodape"><span>${esc(t("sim"))}</span><span>${c.data ? esc(t("gerado", { d: L.dataCurta(c.data, idioma) })) : ""}</span></footer></div>`;
    return { html: pagina, vivos };
  };

  const montar = (raiz, conteudo, figuras) => {
    const { html: pagina, vivos } = html(conteudo, figuras);
    raiz.innerHTML = pagina;
    for (const { id, vivo } of vivos) {
      const palco = raiz.querySelector(`[data-figura="${CSS.escape(id)}"]`), leitura = palco && palco.closest(".iso-plate").querySelector("[data-readout]");
      if (!palco) continue;
      try { vivo(palco, (texto) => { if (leitura && leitura.textContent !== texto) leitura.textContent = texto; }); } catch (e) { console.error(e); }
    }
    ligarNavegacao(raiz);
  };

  // navegação: seção atual no menu, [ e ] trocam de seção, Home e End, e o botão de imprimir
  const ligarNavegacao = (raiz) => {
    const secoes = [...raiz.querySelectorAll(".ap-sec")], links = [...raiz.querySelectorAll("[data-nav]")];
    const marca = (id) => links.forEach((a) => (a.dataset.nav === id ? a.setAttribute("aria-current", "true") : a.removeAttribute("aria-current")));
    if (typeof IntersectionObserver === "function") {
      const visiveis = new Set();
      const io = new IntersectionObserver((es) => {
        for (const e of es) (e.isIntersecting ? visiveis.add(e.target) : visiveis.delete(e.target));
        const primeira = secoes.find((s) => visiveis.has(s));
        if (primeira) marca(primeira.dataset.sec);
      }, { rootMargin: "-20% 0px -65% 0px" });
      secoes.forEach((s) => io.observe(s));
    }
    marca(secoes[0] && secoes[0].dataset.sec);
    const ir = (i) => { const s = secoes[Math.min(secoes.length - 1, Math.max(0, i))]; if (s) { s.scrollIntoView({ block: "start" }); s.focus?.({ preventScroll: true }); } };
    const atual = () => Math.max(0, secoes.findIndex((s) => s.dataset.sec === (links.find((a) => a.getAttribute("aria-current")) || { dataset: {} }).dataset.nav));
    addEventListener("keydown", (e) => {
      if (e.ctrlKey || e.metaKey || e.altKey || /^(INPUT|TEXTAREA|SELECT)$/.test((e.target && e.target.tagName) || "")) return;
      if (e.key === "]") { e.preventDefault(); ir(atual() + 1); } else if (e.key === "[") { e.preventDefault(); ir(atual() - 1); }
      else if (e.key === "Home") { e.preventDefault(); ir(0); } else if (e.key === "End") { e.preventDefault(); ir(secoes.length - 1); }
    });
    raiz.addEventListener("click", (e) => { if (e.target.closest("[data-acao=imprimir]")) print(); });
    L.fragmento.ligar(raiz); // abre na seção do fragmento da URL e o mantém em dia (biblioteca/fragmento.js)
  };

  L.casca = { html, montar, capaDe, secoesDe, secaoDaFigura };
})(L);
