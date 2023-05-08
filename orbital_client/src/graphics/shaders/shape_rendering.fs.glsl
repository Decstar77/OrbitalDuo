#version 330 core
out vec4 FragColor;

uniform int  mode;
uniform vec4 color;
uniform vec4 shapePosAndSize;
uniform vec4 shapeRadius;

// from http://www.iquilezles.org/www/articles/distfunctions/distfunctions

float CircleSDF(vec2 r, vec2 p, float rad) {
    return 1 - max(length(p - r) - rad, 0);
}

float BoxSDF(vec2 r, vec2 p, vec2 s) {
    return 1 - length(max(abs(p - r) - s, 0));
}

float RoundedBoxSDF(vec2 r, vec2 p, vec2 s, float rad) {
    return 1 - (length(max(abs(p - r) - s + rad, 0)) - rad);
}

void main() {
    vec2 s = shapePosAndSize.zw;
    vec2 r = shapePosAndSize.xy;
    vec2 p = gl_FragCoord.xy;

    if (mode == 0) {
        FragColor = color;
    } else if (mode == 1) {
        float d = CircleSDF(r, p, shapeRadius.x);
        d = clamp(d, 0.0, 1.0);
        FragColor = vec4(color.xyz, color.w * d);
    } else if (mode == 2) {
        float d = RoundedBoxSDF(r, p, s / 2, shapeRadius.x);
        d = clamp(d, 0.0, 1.0);
        FragColor = vec4(color.xyz, color.w * d);
    } else {
        FragColor = vec4(1, 0, 1, 1);
    }
}