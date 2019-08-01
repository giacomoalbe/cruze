#version 330 core

layout (location = 0) in vec3 Position;

out VS_OUPUT {
  vec3 first_color;
  vec3 last_color;
  vec2 start_pos;
  vec2 end_pos;
  vec4 bbox;
} OUT;

uniform mat4 projection;
uniform mat4 model;

uniform vec3 first_color;
uniform vec3 last_color;
uniform vec2 start_pos;
uniform vec2 end_pos;
uniform vec4 bbox;

void main() {
  gl_Position = projection * model * vec4(Position.xyz, 1.0);

  OUT.first_color = first_color;
  OUT.last_color = last_color;

  OUT.start_pos = start_pos;
  OUT.end_pos = end_pos;

  OUT.bbox = bbox;
}
