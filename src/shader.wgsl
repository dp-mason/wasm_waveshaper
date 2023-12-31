// this is the output of Vertex Shader
//      in the context of the fragment shader "@builtin(position)" refers to the framebuffer position of fragment
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color:vec3<f32>, // TODO: are we overwriting the vert buffer (position part that is at loc 0) ??
    @location(2) slope_intercept:vec2<f32>,
    @location(3) world_pos:vec3<f32>,
};

// !!! WGSL INTERPRETS MATRICES AS SETS OF COLUMN VECTORS !!!
    // example: mat2x3 data type in wgsl is a matrix with 2 columns and 3 rows
    // https://gpuweb.github.io/gpuweb/wgsl/#matrix-types
struct GraphicsInput {
    cursor_pixel_pos:vec4<f32>,
    world_to_clip_transfm:mat4x4<f32>,
    canvas_dimensions:vec4<u32>,
}

@group(0) @binding(0) var<uniform> graphics_input: GraphicsInput;

// Vertex Shader
// @location(0) is the position of the vert in clip space, written to the vertex buffer in rendering.rs
// @location(1) is the color that we assigned to this vert and wrote to the vertex buffer
@vertex
fn vert_main(
    @builtin(vertex_index) vert_index:u32, // we can use these because they are buffers defined and written to in the configuration of the vert buffers
    @builtin(instance_index) inst_index:u32,
    @location(0) world_position:vec3<f32>, 
    @location(1) color:vec3<f32>,
    @location(2) instance_pos:vec2<f32>,
    @location(3) right_nbr_pos:vec3<f32>,
    @location(4) instance_scale:f32,
) -> VertexOutput {
    var return_data:VertexOutput;
    // write some data to the vertex's position attribute, THIS VALUE WILL BE CHANGED INBETWEEN THE VERT AND FRAG SHADERS
    // TODO: 67 is a magic number repesenting the total verts that make up the background plane and the circle node
    if(vert_index < 4u){
        
        // the vert shader for the background and circle
        return_data.position = vec4<f32>(world_position, 1.0); // dont transform to clip space, the background coords are actually already in clip space
        // color of background plane corresponds with the UV coords of the plane
        return_data.color = vec3<f32>(color[0], color[1], 0.0);

        return return_data; // doesn't need anything else if it is the background 
    }
    
    // nbr_wrap is a value that enables the wave shape visualization to extend past the right border of the screen
    // it will onlt apply to circle instances that are on the furthest right point of the screen
    // TODO: here "20.0" is a magic number representing the unit width of the screen in world coords
    //  this will need to be brought in here so that the wrapping the wav viz works regardless of aspect ratio
    var card_scale:vec3<f32>;
    // max_ext can be aaplied to x or y scale to make the card extend past the edges of the view
    var max_ext:f32 = (1.0/graphics_input.world_to_clip_transfm[1][1]) * 4.0;
    if right_nbr_pos[0] < instance_pos[0] {
        card_scale = vec3(max_ext, max_ext, 0.0);
    }
    else {
        card_scale = vec3(right_nbr_pos[0] - instance_pos[0], max_ext, 1.0);
    }

    // pass the world position
    return_data.world_pos = world_position * vec3(instance_scale, instance_scale, 1.0) * card_scale + vec3(instance_pos, 0.0);

    // the vert shader for the circle instances
    return_data.position = graphics_input.world_to_clip_transfm * vec4(return_data.world_pos, 1.0);
    
    // todo: highlight this circle if the cursor is hovering over it
    return_data.color = vec3(0.0, world_position[1] * instance_scale + instance_pos[1], 0.0);

    var slope:f32;
    // rightmost slope takes into account that the position of the right neighbor of the
    // rightmost node's right neighbor should be treated as though it is off screen to the right in order to appear cyclical
    if (right_nbr_pos[0] < instance_pos[0]) {
        // world_to_clip_transfm[1][1] represents the current world scale
        slope = (right_nbr_pos[1] - instance_pos[1]) / ( (right_nbr_pos[0] + 2.0 * (1.0/graphics_input.world_to_clip_transfm[1][1])) - instance_pos[0] );
    } else {
        // normal slope calculation
        slope = (right_nbr_pos[1] - instance_pos[1]) / (right_nbr_pos[0] - instance_pos[0]);
    }
    //const slope = 2.0, // TODO: placeholder slope
    return_data.slope_intercept = vec2(
        slope,
        (instance_pos[1] + (instance_pos[0] * -1.0 * slope)) // y axis intercept
    );
    
    return return_data;
}



// Background: Puts a red circle around the cursor, rest of the plane is the color of the UV position of the fragment
// The Circle instances are shaded such that they show their relationship to their right neighbor
// TODO: I think there may be some advantages to having everything be in one shader as far as effects, but the professional
//  thing to do would be to have separate shaders and draw call for the background and circle instances. Oh well...
@fragment
fn frag_main(
    vert_data: VertexOutput, 
) -> @location(0) vec4<f32> {
    // We assigned a clip space postion to the @builtin position attribute before, but now it has been transformed
    // into the framebuffer coordinate position of this fragment. This happened INBETWEEN vert and frag stages
    // whereas the color will be a direct interpolation of the value we assigned in the vert shader
    
    var diff_vec = vert_data.position - graphics_input.cursor_pixel_pos;
    var cull = diff_vec[0] > 50.0 || diff_vec[1] > 50.0;
    if cull == false {
        var dist = length(diff_vec);
        // creates a 10px radius red circle around the cursor
        if dist < 10.0 {
            return vec4<f32>(1.0, 0.0, 0.0, 1.0); 
        }
    }

    // if this fragment falls between the waveshaping line and the zero baseline, shade it in
    if ( (vert_data.world_pos[1] < (vert_data.slope_intercept[0] * vert_data.world_pos[0]) + vert_data.slope_intercept[1] && 
         vert_data.world_pos[1] > 0.0) ||
         (vert_data.world_pos[1] > (vert_data.slope_intercept[0] * vert_data.world_pos[0]) + vert_data.slope_intercept[1] &&
         vert_data.world_pos[1] < 0.0 )
    ) {
        //this is where the shader for the wave visualization is defined
        return vec4<f32>(1.0, 1.0, 1.0, 1.0); // show the shape of the wave in white
    }
    
    // transparent, because not in area under the waveshape, shows the webpage background not background plane :(
    // there must need to be multiple draw calls if I want one plane rendered on top of the other?
    return vec4<f32>(.0, 0.0, 0.0, 0.0);
}