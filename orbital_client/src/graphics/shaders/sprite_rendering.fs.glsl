#version 330 core
in vec2 TexCoords;
out vec4 color;

uniform sampler2D sprite_texture;
uniform vec4 sprite_color;

void main()
{
    vec4 sampled = texture(sprite_texture, TexCoords) * sprite_color;
    color = sampled;
}
