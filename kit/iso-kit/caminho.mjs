// Caminhos SVG: pontos em texto de path.
import { tenth } from "./interno.mjs";
export function pathOf(points, closed = false) {
    if (!points.length)
        return "";
    return points.map(([x, y], at) => `${at ? "L" : "M"}${tenth(x)} ${tenth(y)}`).join("") + (closed ? "Z" : "");
}
export const joinPaths = (paths) => paths.filter(Boolean).join("");
