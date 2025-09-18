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

//#version 430 core

//layout(location = 0) in vec3 position;
//layout(location = 1) in vec4 color;

//out vec4 vertexColor;

//void main()
//{

//    mat4 myMatrix;

    // Identity matrix
//    myMatrix[0] = vec4(1.0, 0.0, 0.0, 0.0);
//    myMatrix[1] = vec4(0.0, 1.0, 0.0, 0.0);
//    myMatrix[2] = vec4(0.0, 0.0, 1.0, 0.0);
//    myMatrix[3] = vec4(0.0, 0.0, 0.0, 1.0);

//    gl_Position = myMatrix * vec4(position, 1.0);

    // Pass color to fragment shader
//    vertexColor = color;
//}

//#################################################################

//Task 3 b)

//#version 430 core

//layout(location = 0) in vec3 position;
//layout(location = 1) in vec4 color;

//out vec4 vertexColor;

// Uniform variable passed in from Rust
//uniform float u_val;

//void main()
//{
//    mat4 myMatrix;

//    float a = 1.0; 
//    float b = 0.0; 
//    float c = 0.0; 
//    float d = 0.0; 
//    float e = 1.0; 
//    float f = 0.0; 

//    // a = u_val;  // x scaling
//    // b = u_val;  // shear x (relative to y)
//    // c = u_val;  // translate x
//    // d = u_val;  // shear y (relative to x)
//    // e = u_val;  // y scaling
//    f = u_val;  // translate y

//    myMatrix[0] = vec4(a, d, 0.0, 0.0);
//    myMatrix[1] = vec4(b, e, 0.0, 0.0);
//    myMatrix[2] = vec4(0.0, 0.0, 1.0, 0.0);
//    myMatrix[3] = vec4(c, f, 0.0, 1.0);

//    gl_Position = myMatrix * vec4(position, 1.0);
//    vertexColor = color;
//}


//#################################################################

// Task 4 a)

#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

out vec4 vertexColor;

// Transformation matrix passed from CPU
uniform mat4 u_transform;

void main()
{
    gl_Position = u_transform * vec4(position, 1.0);
    vertexColor = color;
}


//#################################################################