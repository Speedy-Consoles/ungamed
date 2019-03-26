#version 400

uniform mat4 world_to_screen_matrix;
uniform mat4 object_to_world_matrix;

in vec3 position;
in vec2 texture_position;

out vec4 vg_world_position;
out vec4 vg_screen_position;
out vec2 vg_texture_position;

void main() {
    vg_world_position = object_to_world_matrix * vec4(position, 1.0);
    vg_screen_position = world_to_screen_matrix * vg_world_position;
    vg_texture_position = texture_position;
}
