#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec3 normal;

out vec4 vertexColor;
out vec3 vertexNormal;
out vec3 fragPos;

uniform mat4 u_model;
uniform mat4 u_model_view_projection;

void main()
{
    gl_Position = u_model_view_projection * vec4(position, 1.0);

    vertexColor = color;

    mat3 croppedModel = mat3(u_model);
    vertexNormal = normalize(croppedModel * normal);

    fragPos = vec3(u_model * vec4(position, 1.0));

}