#version 330 core

in VS_OUPUT {
  vec3 Color;
  vec2 v_uv;
} IN;

out vec4 Color;

void main() {
  vec3 t_color = vec3(1, 0, 0);
  vec3 b_color = vec3(0, 0, 1);

  //Color = vec4(b_color * (1 - IN.v_uv.y) + t_color * IN.v_uv.y, 1);
  Color = vec4(IN.Color.rgb, 1.0);
}
