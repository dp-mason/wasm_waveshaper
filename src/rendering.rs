
// Received a ton of help from: https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#first-some-housekeeping-state
use wgpu::{Instance, util::DeviceExt, DynamicOffset};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position:[f32;3],
    color:[f32;3]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GraphicsInput {
    cursor_position:[f32;4],
    world_to_clip_transfm:[[f32;4];4],
    canvas_dimensions:[u32;4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CircleInstance {
    position:[f32;3],
    right_nbr_pos:[f32;3],
    scale:f32
}
impl CircleInstance {
    // returns a vertex buffer layout used for storing this data type in a Vertex Buffer
    // TODO: I think my issue has gotta be origniating here... Idk how tho, look up how vertexbufferlayout works again
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout{
            array_stride: std::mem::size_of::<CircleInstance>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute{
                    offset:0,
                    shader_location:2,
                    format:wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute{
                    offset:std::mem::size_of::<[f32; 3]>()       as wgpu::BufferAddress,
                    shader_location:3,
                    format:wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute{
                    offset:(std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location:4,
                    format:wgpu::VertexFormat::Float32,
                }
            ]
        }
    }
}

// TODO: move this to a "shapes" module
// define triangles that fill the screen
// to accomodate this setup in the render pipeline config settings, the topology is set to "strip"
const VERTICES:&[Vertex] = &[
    // background filling triangles
    // also used for rendering the areas between the points and the baseline
    Vertex{position: [ 1.0,  1.0, 1.0], color: [1.0, 0.0, 0.0]}, // top right
    Vertex{position: [-1.0,  1.0, 1.0], color: [0.0, 0.0, 0.0]}, // top left
    Vertex{position: [ 1.0, -1.0, 1.0], color: [1.0, 1.0, 0.0]}, // bottom right
    Vertex{position: [-1.0, -1.0, 1.0], color: [0.0, 1.0, 0.0]}, // bottom left

    // Circle
    // Vertex{ position: [ 0.000000, -1.000000, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [-0.382683, -0.923880, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [-0.707107, -0.707107, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [-0.923880, -0.382683, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [-1.000000,  0.000000, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [-0.923880,  0.382683, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [-0.707107,  0.707107, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [-0.382683,  0.923880, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [ 0.000000,  1.000000, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [ 0.382683,  0.923880, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [ 0.707107,  0.707107, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [ 0.923880,  0.382684, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [ 1.000000, -0.000000, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [ 0.923879, -0.382684, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [ 0.707107, -0.707107, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [ 0.382683, -0.923880, 0.5], color: [1.0, 0.0, 1.0] },
    // Vertex{ position: [-0.000000, -0.000000, 0.5], color: [1.0, 0.0, 1.0] },

    // WaveCard
    Vertex{position: [ 1.0,  0.5, 1.0], color: [1.0, 0.0, 0.0]}, // top right
    Vertex{position: [ 0.0,  0.5, 1.0], color: [0.0, 0.0, 0.0]}, // top left
    Vertex{position: [ 1.0, -0.5, 1.0], color: [1.0, 1.0, 0.0]}, // bottom right
    Vertex{position: [ 0.0, -0.5, 1.0], color: [0.0, 1.0, 0.0]}, // bottom left
];

// would be four, but wavefront obj format starts indexing from one for some reason
// ^^^ the numbers were taken from a wavefront obj file I exported
// TODO: I changed the offset to four to test creating the WaveCards, change it back if u use circles again
const CIRCLE_START_OFFSET:u16 = 4; 

const TRI_INDEX_BUFFER:&[u16] = &[
    // Background Filling Triangles
    2, 0, 1,
    2, 1, 3,
    
    // Circle
    //  1 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET,  2 + CIRCLE_START_OFFSET,
    //  2 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET,  3 + CIRCLE_START_OFFSET,
    //  3 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET,  4 + CIRCLE_START_OFFSET,
    //  4 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET,  5 + CIRCLE_START_OFFSET,
    //  5 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET,  6 + CIRCLE_START_OFFSET,
    //  6 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET,  7 + CIRCLE_START_OFFSET,
    //  7 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET,  8 + CIRCLE_START_OFFSET,
    //  8 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET,  9 + CIRCLE_START_OFFSET,
    //  9 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET, 10 + CIRCLE_START_OFFSET,
    // 10 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET, 11 + CIRCLE_START_OFFSET,
    // 11 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET, 12 + CIRCLE_START_OFFSET,
    // 12 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET, 13 + CIRCLE_START_OFFSET,
    // 13 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET, 14 + CIRCLE_START_OFFSET,
    // 14 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET, 15 + CIRCLE_START_OFFSET,
    // 15 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET, 16 + CIRCLE_START_OFFSET,
    // 16 + CIRCLE_START_OFFSET, 17 + CIRCLE_START_OFFSET,  1 + CIRCLE_START_OFFSET,

    // WaveCard
    2 + CIRCLE_START_OFFSET, 1 + CIRCLE_START_OFFSET, 3 + CIRCLE_START_OFFSET,
    2 + CIRCLE_START_OFFSET, 0 + CIRCLE_START_OFFSET, 1 + CIRCLE_START_OFFSET
];

fn dot_product(transform_matrix:[[f32;4];4], vector:[f32;4]) -> [f32;4]{
    [
        transform_matrix[0][0] * vector[0] + transform_matrix[1][0] * vector[1] + transform_matrix[2][0] * vector[2] + transform_matrix[3][0] * vector[3],
        transform_matrix[0][1] * vector[0] + transform_matrix[1][1] * vector[1] + transform_matrix[2][1] * vector[2] + transform_matrix[3][1] * vector[3],
        transform_matrix[0][2] * vector[0] + transform_matrix[1][2] * vector[1] + transform_matrix[2][2] * vector[2] + transform_matrix[3][2] * vector[3],
        transform_matrix[0][3] * vector[0] + transform_matrix[1][3] * vector[1] + transform_matrix[2][3] * vector[2] + transform_matrix[3][3] * vector[3],
    ]
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    load_color: wgpu::Color, // for the challenge section, delete later
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,

    render_pipeline: wgpu::RenderPipeline,

    cursor_pos:[f32;2],
    aspect_ratio:f32,
    world_scale:f32,
    clip_to_world_transform:[[f32;4];4],
    
    graphics_input_buffer: wgpu::Buffer,
    
    uniform_bind_group: wgpu::BindGroup,
    
    vertex_buffer: wgpu::Buffer,
    tri_index_buffer: wgpu::Buffer,
    num_tri_indices: u32,

    circle_instances: Vec<CircleInstance>,
    circle_instances_buffer: wgpu::Buffer,
}
impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // creates (handle?) to the GPU, Backends refers to Vulkan, DXD12, Metal and (BrowserWebGPU?)
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor{
            backends:wgpu::Backends::all(),
            dx12_shader_compiler:wgpu::Dx12Compiler::Dxc { dxil_path: None, dxc_path: None }
        });

        // Apparently this should be safe, because "State" owns the window
        // this ensures that the State will live as long as the window?
        // "surface" is the section of the window we draw to
        let surface = unsafe { instance.create_surface(&window).unwrap() };

        // "adapter" houses the info about our GPU
        // if you have multiple graphics cards you will be able to interate over returned adapters
        // used to create Device and Queue
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false, // do not default to software rendering
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            view_formats: vec![surface.get_capabilities(&adapter).formats[0]],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let load_color = wgpu::Color{r:0.0, g:0.0, b:0.0, a:1.0};

        // "include_str!" imports the contents of a file as a static string, which can be useful
        // I instead use include_wgsl! which creates a ShaderModuleDescriptor from the file that you supply
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        
        // !! BUFFER STUFF !!

        // Vertex Buffer
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        // Triangle Index Buffer
        let tri_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(TRI_INDEX_BUFFER),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_tri_indices = TRI_INDEX_BUFFER.len() as u32;

        let circle_instances: [CircleInstance; 0] = [
            // CircleInstance {
            //     position:[0.0, 0.0, 0.0],
            //     scale:1.0, //TODO: bruh this is a hack, figure this shit out
            //     right_nbr_pos:[10.0, 0.0, 0.0], // TODO: 10 is a migic num and this is a placeholder nyways
            // },
        ];
        let circle_instances_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Circle Instance Buffer"),
                contents: bytemuck::cast_slice(&circle_instances),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        // Create Vertex Buffer Layout
        // From: https://sotrh.github.io/learn-wgpu/beginner/tutorial4-buffer/#so-what-do-i-do-with-it
        //      We need to tell the render_pipeline to use this buffer when we are drawing, but first, we need to 
        //  tell the render_pipeline how to read the buffer. We do this using VertexBufferLayouts and the 
        //  vertex_buffers field that I promised we'd talk about when we created the render_pipeline.
        //      A VertexBufferLayout defines how a buffer is represented in memory. Without this, the 
        //  render_pipeline has no idea how to map the buffer in the shader. Here's what the descriptor for a 
        //  buffer full of Vertex would look like.
        // If we wanna get sophistacated with it, we can returnb this layout description in a implementation
        //  of the Vertex struct. Not too worried about it right now though
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            // byte length of vertex data, so array can be stepped through in linear memory
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // put the position data in the first location of the vert buffer
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3, // matches format defined in Vertex struct
                },
                // put the color data in the second location in the vert buffer
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, // step past postion data to get color
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        };

        let cursor_pos:[f32;4] = [-1.0, 1.0, 0.0, 1.0];
        let aspect_ratio: f32 = size.height as f32 / size.width as f32;

        // !!! WGSL INTERPRETS MATRICES AS SETS OF COLUMN VECTORS !!!
        // example: mat2x3 data type in wgsl is a matrix with 2 columns and 3 rows
        // https://gpuweb.github.io/gpuweb/wgsl/#matrix-types
        let world_scale:f32 = 0.1;
        let world_to_clip_transfm:[[f32;4];4] = [
            [aspect_ratio * world_scale,     0.0    , 0.0, 0.0],
            [            0.0           , world_scale, 0.0, 0.0],
            [            0.0           ,     0.0    , 1.0, 0.0],
            [            0.0           ,     0.0    , 0.0, 1.0],
        ];
        let clip_to_world_transform:[[f32;4];4] = [
            [1.0 / (aspect_ratio*world_scale),         0.0      , 0.0, 0.0],
            [               0.0              , 1.0 / world_scale, 0.0, 0.0],
            [               0.0              ,         0.0      , 1.0, 0.0],
            [               0.0              ,         0.0      , 0.0, 1.0],
        ];

        let graphics_input = GraphicsInput {
            cursor_position:cursor_pos,
            world_to_clip_transfm:world_to_clip_transfm,
            canvas_dimensions:[size.height, size.width, 0, 0],
        };

        // create uniform buffer for the cursor position and other info such as aspect ratio
        let graphics_input_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Cursor Position and Aspect Ratio Buffer"),
                contents: bytemuck::cast_slice(&[graphics_input]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
                
        // create bind group LAYOUT with this buffer
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("uniform_bind_group_layout"),
        });

        // create ACTUAL bind group FROM LAYOUT and BUFFER that we just made
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: graphics_input_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });

        // create proper pipeline layout (declares buffers and such, look into this more)
        // use the LAYOUT we created
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Basic Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[], // ???
            });
        
        // create a render pipeline
        let render_pipeline = 
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
                label: Some("Basic Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vert_main",
                    buffers: &[vertex_buffer_layout, CircleInstance::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "frag_main",
                    targets: &[Some(wgpu::ColorTargetState{
                        format:config.format,                   // matches the color config of the SurfaceTexture
                        blend: Some(wgpu::BlendState::REPLACE), // replace the old pixel data with new data
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // every three verts forms a triangle
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // counter-clockwise is the direction tris are drawn
                    cull_mode: Some(wgpu::Face::Back), // if the triangle is facing away, do not draw it
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None, // is stencil like a silhoutte of object?
                multisample: wgpu::MultisampleState {
                    count: 1, // ?? look into the topic of "multisampling", like raytrace samples?
                    mask: !0, // all samples should be active ?? how is "!0" diff than "1"
                    alpha_to_coverage_enabled: false, // ?? has to do with anti-aliasing
                },
                multiview: None, // ?? look into what "array textures" are, more than one surface texture?
            });

        Self {
            surface,
            device,
            load_color,
            queue,
            config,
            size,
            window,
            
            render_pipeline,

            aspect_ratio,
            world_scale,
            clip_to_world_transform,

            cursor_pos:[0.0, 0.0],
            graphics_input_buffer,
            
            uniform_bind_group,

            vertex_buffer,
            tri_index_buffer,
            num_tri_indices,

            circle_instances:circle_instances.to_vec(),
            circle_instances_buffer
        }
    }

    pub fn add_circle_instance(&mut self, world_position:[f32;3], scale:f32) {
        let new_circle = CircleInstance {
            position:world_position,
            right_nbr_pos:[world_position[0], 0.0, 0.0], //TODO: placeholder neighbor pos
            scale:scale,
        };

        // if empty list, populate the head, else search for place within list where this fits
        match self.circle_instances.is_empty() {
            true => {
                self.circle_instances.push(new_circle);
            },
            false => {
                // search for where this circle fits in this list of circles ordered by x axis position
                let res = self.circle_instances.binary_search_by(|probe| probe.position[0].total_cmp(&new_circle.position[0]));
                match res {
                    Ok(index) => {
                        // binary search was able to find an element at this exact position in the node list, don't add
                        log::warn!("Error: there is already a circle at position: {} not adding node to list", new_circle.position[0]);
                    },
                    Err(index) => {
                        // binary search could not find a node at this wave position, tells us the index of where it 
                        // would be in the list if it existed, use that to insert the node and preserve sort by wave pos
                        self.circle_instances.insert(index, new_circle);
                        // update the right neighbor
                        self.circle_instances[index].right_nbr_pos = self.circle_instances[(index + 1) % self.circle_instances.len()].position;
                        // update the right neighbor of this new nodes left neighbor
                        // extra logic allows end of list to have right neighbor at begining of list
                        let num_circs = self.circle_instances.len(); // need this bc we cant borrow within borrow on next line
                        self.circle_instances[(index + num_circs - 1) % num_circs].right_nbr_pos = self.circle_instances[index].position;
                        log::warn!("node added at index: {}", index);
                    }
                }
            }
        }
        
        
        
        log::warn!("Content of instances is: {:?}",self.circle_instances);
        // Write the entire instances buffer again to new buffer 
        // TODO: this is bad, use offset instead if there is extra capacity, reset the buff once it has reach capacity
        //self.cursor_pos_buffer.unmap();
        self.circle_instances_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Circle Instance Buffer, Grown"),
                contents: bytemuck::cast_slice(&self.circle_instances.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
    }

    // takes a position in clip space (usually the mouse location) and returns the index of the circle instance
    // that exists there if one exists
    // todo: move this to a compute shader once you figure out how that can cooperate with the current browsers
    pub fn circle_at_location(&self, target_world_pos:[f32; 2]) -> Option<usize> {
        let mut index = 0;
        for circle in &self.circle_instances {
            // calculate vector between origin of this circle instance and passed position
            let diff_vector:[f32;2] = [circle.position[0] - target_world_pos[0], circle.position[1] - target_world_pos[1]];
            // helps rule out circles before doing proper distance calculation
            if !(diff_vector[0].abs() > circle.scale || diff_vector[1].abs() > circle.scale) {
                let dist = (diff_vector[0].powf(2.0) + diff_vector[1].powf(2.0)).sqrt();
                if dist < circle.scale {
                    return Some(index);
                }
            }
            index += 1;
        }
        None
    }

    pub fn expand_circle(&mut self, circle_index:usize) -> Result<(), &str> {
        if circle_index < self.circle_instances.len() {
            // do something visually to the circle now that it has been clicked? idk
        }
        Err("index out of bounds")
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        // TODO: write new size to graphics input
    }

    pub fn get_cursor_clip_location(&self) -> [f32;4] {
        let cursor_clip_x = ((self.cursor_pos[0] / self.size.width as f32 ) - 0.5) * 2.0;
        let cursor_clip_y = ((self.cursor_pos[1] / self.size.height as f32) - 0.5) * -2.0;
        [cursor_clip_x, cursor_clip_y, 0.0, 1.0]
    }

    pub fn add_circle_at_clip_location(&mut self, clip_loc:[f32;4]){
                
        let world_loc = dot_product(self.clip_to_world_transform, clip_loc);
            
        // determine whether the clicked position is within an existing circle
        match self.circle_at_location([world_loc[0], world_loc[1]]) {
            Some(index) => {
                log::warn!("Clicked circle at index: {index}");
                match self.expand_circle(index) {
                    Err(msg) => log::error!("ADD CIRCLE AT CLIP LOC ERR: {msg}"),
                    _ => {},
                }
            },
            None => {
                log::warn!("new circle created at world location: {:?}", world_loc);
                self.add_circle_instance([world_loc[0], world_loc[1], world_loc[2]], 1.0);
            }
        }
    }

    // pub fn add_circle_at_cursor_location(&mut self, state:&ElementState, button:&MouseButton){
    //     match state {
    //         ElementState::Pressed => {
                
    //             let cursor_clip_pos = self.get_cursor_clip_location();
    //             let cursor_world_pos = dot_product(self.clip_to_world_transform, cursor_clip_pos);
                
    //             // determine whether the clicked position is within an existing circle
    //             match self.circle_at_location([cursor_world_pos[0], cursor_world_pos[1]]) {
    //                 Some(index) => {
    //                     log::warn!("Clicked circle at index: {index}");
    //                     match self.expand_circle(index) {
    //                         Err(msg) => log::warn!("{msg}"),
    //                         Ok(_) => {}
    //                     }
    //                 },
    //                 None => {
    //                     log::warn!("new circle created at world location: {:?}", cursor_world_pos);
    //                     self.add_circle_instance([cursor_world_pos[0], cursor_world_pos[1], cursor_world_pos[2]], 1.0);
    //                 }
    //             }
    //         },
    //         ElementState::Released => {/* TODO, could add another UI effect when the button is released, like a ripple */}
    //     }
    // }

    // this is where more user input for rendering can be added
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // match to some input events you want to handle
        // if this event is not in your list of input events, return false so it may be consumed
        // by another process
        match event {
            WindowEvent::CursorMoved{ position,.. } => {
                // change load color (default background color)
                let redval = ((position.x + f64::MIN_POSITIVE) / self.size.width as f64) % 1.0;
                let greenval = ((position.y + f64::MIN_POSITIVE) / self.size.height as f64) % 1.0;
                self.load_color = wgpu::Color { r: redval, g:greenval, b:1.0, a:1.0 };

                self.cursor_pos = [position.x as f32, position.y as f32];
                // write the new cursor pos to buffer
                self.queue.write_buffer(
                    &self.graphics_input_buffer, 
                    0, 
                    bytemuck::cast_slice(&[[
                        [self.cursor_pos[0], self.cursor_pos[1], 0.0f32, 1.0f32]
                    ]]));
                
                true
            },
            _ => false
        }
    }

    pub fn update(&mut self) {

    }

    pub async fn init_rendering() -> (EventLoop<()>, State) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            } else {
                env_logger::init();
            }
        }
    
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
    
        //#[cfg(target_arch = "wasm32")]
        //{
            // TODO: how can we pass the dimnsions of the window from javascript so that the UI takes up the whole screen?
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            window.set_inner_size(PhysicalSize::new(800, 800));
    
            use winit::platform::web::WindowExtWebSys;
    
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("ui-box")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        //}

        let render_state = Self::new(window).await;

        (event_loop, render_state)
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output_texture = self.surface.get_current_texture()?;
        let view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // create the GPU command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // from: https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#render
        //  begin_render_pass() borrows encoder mutably (aka &mut self). We can't call encoder.finish() until
        //  we release that mutable borrow. The block tells rust to drop any variables within it when the code
        //  leaves that scope thus releasing the mutable borrow on encoder and allowing us to finish() it.
        //  If you don't like the {}, you can also use drop(render_pass) to achieve the same effect.
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets in the frag shader wgsl code
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            //load: wgpu::LoadOp::Clear(color),
                            load: wgpu::LoadOp::Clear(self.load_color),
                            store: true,
                        },
                    })
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            // Designate a vertex buffer
            // The reason "slice" is used is because we can store many objects in a single vertex buffer
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.circle_instances_buffer.slice(..)); // set the instance buffer
            render_pass.set_index_buffer(self.tri_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            // draw something with 4 vertices, and 1 instance. This is where @builtin(vertex_index) comes from in the vert shader wgsl code TODO: comment doesn't make sense
            if self.circle_instances.len() > 0 {
                // TODO: these should prob be on separate (render passes?), shaders, etc. Like... the draw command for the background plane
                // probably should have no concept of the circle instances: different vert bufs, etc. It's a bit of a tricky thing to go back and untangle
                render_pass.draw_indexed(0..6, 0, 0..1); // draw background, remember range is not max inclusive
                render_pass.draw_indexed(6..self.num_tri_indices, 0, 0..self.circle_instances.len() as u32); // draw circles
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output_texture.present();

        Ok(())
    }

    pub fn handle_window_maintenance_events(&mut self, event:&Event<()>, control_flow:&mut ControlFlow) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == &self.window().id() => {
                if !self.input(event) {
                    // UPDATED!
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            self.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            self.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == &self.window().id() => {
                self.update();
                match self.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        self.resize(self.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                self.window().request_redraw();
            }
            _ => {}
        }
    }
}


