// GLSL for series rendering. Positions arrive relative to a reference origin (float32-safe);
// the data→pixel transform is a uniform, so pan/zoom never rebuilds geometry.

const transform = /* glsl */ `
uniform vec2 uPlotScale;      // pixels per data unit (y negative: pixel y grows downward)
uniform vec2 uPlotOffset;     // pixel position of the reference origin
uniform vec2 uPlotSize; // plot size in CSS pixels

vec2 toPixels(vec2 relative) {
  return relative * uPlotScale + uPlotOffset;
}

vec4 toClip(vec2 p) {
  return vec4(p.x / uPlotSize.x * 2.0 - 1.0, 1.0 - p.y / uPlotSize.y * 2.0, 0.0, 1.0);
}
`;

/** One instanced quad per line segment; constant pixel width. */
export const lineVertex = /* glsl */ `
in vec2 aPosition; // quad corner: x ∈ {0, 1} (start → end), y ∈ {-1, 1} (side)
in vec2 aStart;
in vec2 aEnd;
uniform float uWidth;
out float vAlong;
${transform}
void main() {
  vec2 a = toPixels(aStart);
  vec2 b = toPixels(aEnd);
  vec2 d = b - a;
  float len = length(d);
  vec2 t = len > 0.0001 ? d / len : vec2(1.0, 0.0);
  vec2 n = vec2(-t.y, t.x);
  float hw = uWidth * 0.5;
  vec2 p = mix(a, b, aPosition.x) + t * (aPosition.x * 2.0 - 1.0) * hw + n * aPosition.y * hw;
  vAlong = aPosition.x * len;
  gl_Position = toClip(p);
}
`;

export const lineFragment = /* glsl */ `
in float vAlong;
uniform vec4 uSeriesColor;
uniform float uDashed;
out vec4 finalColor;
void main() {
  if (uDashed > 0.5 && mod(vAlong, 10.0) > 6.0) discard;
  finalColor = vec4(uSeriesColor.rgb * uSeriesColor.a, uSeriesColor.a);
}
`;

/** One instanced quad per marker, shaded as an anti-aliased (optionally hollow) disc. */
export const markerVertex = /* glsl */ `
in vec2 aPosition; // quad corner in [-1, 1]²
in vec2 aCenter;
uniform highp float uRadius; // shared by both stages, so precision must match
out vec2 vLocal;
${transform}
void main() {
  float extent = uRadius + 1.0;
  vLocal = aPosition * extent;
  gl_Position = toClip(toPixels(aCenter) + vLocal);
}
`;

export const markerFragment = /* glsl */ `
in vec2 vLocal;
uniform vec4 uSeriesColor;
uniform highp float uRadius; // shared by both stages, so precision must match
uniform float uHollow;
out vec4 finalColor;
void main() {
  float d = length(vLocal);
  float coverage = clamp(uRadius + 0.5 - d, 0.0, 1.0);
  if (uHollow > 0.5) coverage *= clamp(d - (uRadius - 1.5) + 0.5, 0.0, 1.0);
  if (coverage <= 0.0) discard;
  float alpha = uSeriesColor.a * coverage;
  finalColor = vec4(uSeriesColor.rgb * alpha, alpha);
}
`;
