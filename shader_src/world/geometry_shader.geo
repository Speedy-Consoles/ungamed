#version 400

layout(triangles) in;
layout(triangle_strip, max_vertices = 3) out;

in vec4 vg_screen_position[];
in vec4 vg_world_position[];
in vec2 vg_texture_position[];

out vec3 gf_normal;
out vec2 gf_texture_position;

void main() {
    vec3 edge1 = vg_world_position[1].xyz - vg_world_position[0].xyz;
    vec3 edge2 = vg_world_position[2].xyz - vg_world_position[0].xyz;
    gf_normal = normalize(cross(edge1, edge2));

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
