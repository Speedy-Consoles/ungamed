#version 400

uniform sampler2D tex;
uniform vec3 color;

in vec2 gf_texture_position;

out vec4 out_color;

void main() {
    out_color = vec4(color, 1.0) * texture(tex, gf_texture_position);
}
