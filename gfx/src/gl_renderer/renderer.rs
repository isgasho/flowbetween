use super::buffer::*;
use super::shader::*;
use super::texture::*;
use super::vertex_array::*;
use super::render_target::*;
use super::shader_program::*;

use crate::action::*;
use crate::buffer::*;

use std::ffi::{CString};

///
/// OpenGL action renderer
///
pub struct GlRenderer {
    /// The buffers allocated to this renderer
    buffers: Vec<Option<Buffer>>,

    /// The textures allocated to this renderer
    textures: Vec<Option<Texture>>,

    /// The 'main' render target that represents the output for this renderer
    default_render_target: Option<RenderTarget>,

    /// The render targets assigned to this renderer
    render_targets: Vec<Option<RenderTarget>>,

    /// The simple shader program
    simple_shader: ShaderProgram
}

impl GlRenderer {
    ///
    /// Creates a new renderer that will render to the specified device and factory
    ///
    pub fn new() -> GlRenderer {
        let simple_vertex_shader    = Shader::compile(&String::from_utf8(include_bytes!["../../shaders/simple/simple.glslv"].to_vec()).unwrap(), ShaderType::Vertex, vec!["a_Pos", "a_Color", "a_TexCoord"]);
        let simple_fragment_shader  = Shader::compile(&String::from_utf8(include_bytes!["../../shaders/simple/simple.glslf"].to_vec()).unwrap(), ShaderType::Fragment, vec![]);
        let simple_shader           = ShaderProgram::from_shaders(vec![simple_vertex_shader, simple_fragment_shader]);

        GlRenderer {
            buffers:                vec![],
            textures:               vec![],
            default_render_target:  None,
            render_targets:         vec![],
            simple_shader:          simple_shader
        }
    }

    ///
    /// Prepares to render to the active framebuffer
    ///
    pub fn prepare_to_render_to_active_framebuffer(&mut self, width: usize, height: usize) {
        unsafe {
            // Set the default render target to be a reference to the current render target
            self.default_render_target = Some(RenderTarget::reference_to_current());

            // Set the viewport to the specified width and height
            gl::Viewport(0, 0, width as gl::types::GLsizei, height as gl::types::GLsizei);
        }
    }

    ///
    /// Performs rendering of the specified actions to this device target
    ///
    pub fn render<Actions: IntoIterator<Item=GfxAction>>(&mut self, actions: Actions) {
        for action in actions {
            use self::GfxAction::*;

            match action {
                CreateVertex2DBuffer(id, vertices)                                      => { self.create_vertex_buffer_2d(id, vertices); }
                FreeVertexBuffer(id)                                                    => { self.free_vertex_buffer(id); }
                CreateRenderTarget(render_id, texture_id, width, height, render_type)   => { self.create_render_target(render_id, texture_id, width, height, render_type); }
                FreeRenderTarget(render_id)                                             => { self.free_render_target(render_id); }
                SelectRenderTarget(render_id)                                           => { self.select_render_target(render_id); }
                RenderToFrameBuffer                                                     => { self.select_main_frame_buffer(); }
                ShowFrameBuffer                                                         => { /* This doesn't double-buffer so nothing to do */ }
                CreateTextureBgra(texture_id, width, height)                            => { self.create_bgra_texture(texture_id, width, height); }
                FreeTexture(texture_id)                                                 => { self.free_texture(texture_id); }
                Clear(color)                                                            => { self.clear(color); }
            }
        }
    }

    ///
    /// Clears the current render target
    ///
    fn clear(&mut self, Rgba8([r, g, b, a]): Rgba8) {
        let r = (r as f32)/255.0;
        let g = (g as f32)/255.0;
        let b = (b as f32)/255.0;
        let a = (a as f32)/255.0;

        unsafe { 
            // Clear the buffer
            gl::ClearBufferfv(gl::COLOR, 0, &[r, g, b, a][0]); 

            // Draw a test triangle
            let mut buffer  = Buffer::new();
            let vertices    = vec![
                Vertex2D { pos: [0.0, 1.0],     tex_coord: [0.0, 0.0], color: [255, 0, 0, 255] },
                Vertex2D { pos: [-1.0, -1.0],   tex_coord: [0.0, 0.0], color: [0, 255, 0, 255] },
                Vertex2D { pos: [1.0, -1.0],    tex_coord: [0.0, 0.0], color: [0, 0, 255, 255] }
            ];
            buffer.static_draw(&vertices);

            gl::UseProgram(*self.simple_shader);

            let transform_loc = gl::GetUniformLocation(*self.simple_shader, CString::new("u_Transform").unwrap().as_ptr());
            gl::UniformMatrix4fv(transform_loc, 1, gl::FALSE, [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0 
            ].as_ptr());

            gl::Enable(gl::BLEND);

            let vertex_array = VertexArray::new();
            gl::BindVertexArray(*vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, *buffer);

            Vertex2D::define_attributes();
            gl::DrawArrays(gl::TRIANGLES, 0, 3);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    ///
    /// Creates a 2D vertex buffer
    ///
    fn create_vertex_buffer_2d(&mut self, VertexBufferId(buffer_id): VertexBufferId, vertices: Vec<Vertex2D>) {
        // Extend the buffers array as needed
        if buffer_id >= self.buffers.len() {
            self.buffers.extend((self.buffers.len()..(buffer_id+1))
                .into_iter()
                .map(|_| None));
        }

        // Release the previous buffer
        self.buffers[buffer_id] = None;

        // Create a buffer containing these vertices
        let mut buffer = Buffer::new();
        buffer.static_draw(&vertices);

        // Store in the buffers collections
        self.buffers[buffer_id] = Some(buffer);
    }

    ///
    /// Frees the vertex buffer with the specified ID
    ///
    fn free_vertex_buffer(&mut self, VertexBufferId(id): VertexBufferId) {
        self.buffers[id] = None;
    }

    ///
    /// Creates a new BGRA texture
    ///
    fn create_bgra_texture(&mut self, TextureId(texture_id): TextureId, width: usize, height: usize) {
        // Extend the textures array as needed
        if texture_id >= self.textures.len() {
            self.textures.extend((self.textures.len()..(texture_id+1))
                .into_iter()
                .map(|_| None));
        }

        // Free any existing texture
        self.textures[texture_id] = None;

        // Create a new texture
        let mut new_texture = Texture::new();
        new_texture.create_empty(width as u16, height as u16);

        // Store the texture
        self.textures[texture_id] = Some(new_texture);
    }

    ///
    /// Releases an existing render target
    ///
    fn free_texture(&mut self, TextureId(texture_id): TextureId) {
        self.textures[texture_id] = None;
    }

    ///
    /// Creates a new render target
    ///
    fn create_render_target(&mut self, RenderTargetId(render_id): RenderTargetId, TextureId(texture_id): TextureId, width: usize, height: usize, render_type: RenderTargetType) {
        // Extend the textures array as needed
        if texture_id >= self.textures.len() {
            self.textures.extend((self.textures.len()..(texture_id+1))
                .into_iter()
                .map(|_| None));
        }

        // Extend the render targets array as needed
        if render_id >= self.render_targets.len() {
            self.render_targets.extend((self.render_targets.len()..(render_id+1))
                .into_iter()
                .map(|_| None));
        }

        // Free any existing texture and render target
        self.textures[texture_id]       = None;
        self.render_targets[render_id]  = None;

        // Create the new render target
        let new_render_target           = RenderTarget::new(width as u16, height as u16, render_type);

        // Store the properties of the new render target
        self.textures[texture_id]       = new_render_target.texture();
        self.render_targets[render_id]  = Some(new_render_target);
    }

    ///
    /// Chooses which buffer rendering instructions will be sent to
    ///
    fn select_render_target(&mut self, RenderTargetId(render_id): RenderTargetId) {
        self.render_targets[render_id].as_ref().map(|render_target| {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, **render_target)
            }
        });
    }

    ///
    /// Sends rendering instructions to the primary frame buffer for display
    ///
    fn select_main_frame_buffer(&mut self) {
        self.default_render_target.as_ref().map(|render_target| {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, **render_target)
            }
        });
    }

    ///
    /// Releases an existing render target
    ///
    fn free_render_target(&mut self, RenderTargetId(render_id): RenderTargetId) {
        self.render_targets[render_id] = None;
    }

    ///
    /// Flushes all changes to the device
    ///
    pub fn flush(&mut self) {
        unsafe {
            gl::Flush();
        }
    }
}
