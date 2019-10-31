#version 400

layout(triangles) in;
layout(triangle_strip, max_vertices = 3) out;

in vec4 vg_screen_position[];
in vec2 vg_texture_position[];

out vec2 gf_texture_position;

void main() {
    gl_Position = vg_screen_position[0];
    gf_texture_position = vg_texture_position[0];
    EmitVertex();

    gl_Position = vg_screen_position[1];
    gf_texture_position = vg_texture_position[1];
    EmitVertex();

    gl_Position = vg_screen_position[2];
    gf_texture_position = vg_texture_position[2];
    EmitVertex();
}
