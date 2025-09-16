//#################################################################

//Task 1 a) i)

//layout(location = 0) in vec3 position;
//layout(location = 1) in vec4 color;

//out vec4 vertexColor;

//void main()
//{
//    vertexColor = color; // Pass the input color to the fragment shader
//    gl_Position = vec4(position, 1.0);
//}

//#################################################################

//Task 3 a)

#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

out vec4 vertexColor;

void main()
{

    mat4 myMatrix;

    // Identity matrix
    myMatrix[0] = vec4(1.0, 0.0, 0.0, 0.0);
    myMatrix[1] = vec4(0.0, 1.0, 0.0, 0.0);
    myMatrix[2] = vec4(0.0, 0.0, 1.0, 0.0);
    myMatrix[3] = vec4(0.0, 0.0, 0.0, 1.0);

    gl_Position = myMatrix * vec4(position, 1.0);

    // Pass color to fragment shader
    vertexColor = color;
}

//#################################################################