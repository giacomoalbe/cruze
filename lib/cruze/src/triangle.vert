#version 330 core

layout (location = 0) in vec3 Position;

out VS_OUPUT {
  vec3 Color;
  vec2 v_uv;
} OUT;

uniform mat4 projection;
uniform mat4 model;
uniform vec3 color;

void main() {
  gl_Position = projection * model * vec4(Position.xyz, 1.0);

  OUT.Color = color;

  OUT.v_uv = vec2(
      (-gl_Position.x + 0.4) / 0.8,
      (gl_Position.y + 0.4) / 0.8
  );
}
