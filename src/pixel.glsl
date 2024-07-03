precision mediump float;
uniform float size;

void main() {
    if (
        mod(gl_FragCoord.x, size) < 1.0 ||
            mod(gl_FragCoord.y, size) < 1.0
    ) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.8);
    } else {
        discard;
    }
}
