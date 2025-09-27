#version 430 core

//in vec4 vertexColor;
//in vec3 vertexNormal;

//out vec4 color;

//void main()
//{
//    //color = vec4(vertexNormal, 1.0);

//    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
//    vec3 litColor = vertexColor.rgb * (max(0.0, dot(vertexNormal, -lightDirection)));
//    color = vec4(litColor, 1.0);
//}

// Phong shading

in vec4 vertexColor;
in vec3 vertexNormal;
in vec3 fragPos;

out vec4 color;

uniform vec3 u_lightPos;
uniform vec3 u_viewPos;

void main()
{
    vec3 emissiveColor = vec3(0.0, 0.0, 0.0); // Only here because of formula
    vec3 ambientStrength  = vec3(0.2);
    vec3 diffuseStrength  = vec3(0.7);
    vec3 specularStrength = vec3(0.5);
    float shininess = 32.0;

    // Emissive
    vec3 emissive = emissiveColor;

    // Ambient
    vec3 ambient = ambientStrength * vertexColor.rgb;

    // Diffuse
    vec3 norm = normalize(vertexNormal);
    vec3 lightDir = normalize(u_lightPos - fragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diffuseStrength * diff * vertexColor.rgb;

    // Specular
    vec3 viewDir = normalize(u_viewPos - fragPos);
    vec3 reflectDir = reflect(-lightDir, norm);  
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
    vec3 specular = specularStrength * spec * vec3(1.0); // white highlight

    // Final color: I=Ie​+Ia​+Id​+Is​
    vec3 result = emissive + ambient + diffuse + specular;
    color = vec4(result, 1.0);
}