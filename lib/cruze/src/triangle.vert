#version 330 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec3 Color;

out VS_OUPUT {
  vec3 Color;
  vec2 v_uv;
} OUT;

void main() {
  gl_Position = vec4(Position * 0.5, 1.0);

  OUT.Color = Color;

  OUT.v_uv = vec2(
      (-gl_Position.x + 0.4) / 0.8,
      (gl_Position.y + 0.4) / 0.8
  );
}
