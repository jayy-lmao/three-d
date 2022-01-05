use crate::context::consts;
use crate::core::texture::*;

///
/// A texture that covers all 6 sides of a cube.
///
pub struct TextureCubeMap<T: TextureDataType> {
    context: Context,
    id: crate::context::Texture,
    width: u32,
    height: u32,
    format: Format,
    number_of_mip_maps: u32,
    is_hdr: bool,
    _dummy: T,
}

impl<T: TextureDataType> TextureCubeMap<T> {
    ///
    /// Creates a new texture cube map from the given cpu texture.
    /// The cpu texture data must contain 6 images all with the width and height specified in the cpu texture.
    /// The images are used in the following order; right, left, top, bottom, front, back.
    ///
    pub fn new(context: &Context, cpu_texture: &CPUTexture<T>) -> ThreeDResult<TextureCubeMap<T>> {
        let id = generate(context)?;
        let number_of_mip_maps = calculate_number_of_mip_maps(
            cpu_texture.mip_map_filter,
            cpu_texture.width,
            cpu_texture.height,
        );
        set_parameters(
            context,
            &id,
            consts::TEXTURE_CUBE_MAP,
            cpu_texture.min_filter,
            cpu_texture.mag_filter,
            if number_of_mip_maps == 1 {
                None
            } else {
                cpu_texture.mip_map_filter
            },
            cpu_texture.wrap_s,
            cpu_texture.wrap_t,
            Some(cpu_texture.wrap_r),
        );
        context.bind_texture(consts::TEXTURE_CUBE_MAP, &id);
        context.tex_storage_2d(
            consts::TEXTURE_CUBE_MAP,
            number_of_mip_maps,
            T::internal_format(cpu_texture.format)?,
            cpu_texture.width,
            cpu_texture.height,
        );
        let mut texture = Self {
            context: context.clone(),
            id,
            width: cpu_texture.width,
            height: cpu_texture.height,
            format: cpu_texture.format,
            number_of_mip_maps,
            is_hdr: T::bits_per_channel() > 8,
            _dummy: T::default(),
        };
        texture.fill(&cpu_texture.data)?;
        Ok(texture)
    }

    ///
    /// Fills the cube map texture with the given data which should contain pixel data for 6 images in the following order; right, left, top, bottom, front, back.
    ///
    /// # Errors
    /// Returns an error if the length of the data does not correspond to 6 images with the width, height and format specified at construction.
    ///
    pub fn fill(&mut self, data: &[T]) -> ThreeDResult<()> {
        let offset = data.len() / 6;
        check_data_length(self.width, self.height, 1, self.format, offset)?;
        self.context
            .bind_texture(consts::TEXTURE_CUBE_MAP, &self.id);
        for i in 0..6 {
            T::fill(
                &self.context,
                consts::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                self.width,
                self.height,
                self.format,
                &data[i * offset..(i + 1) * offset],
            );
        }
        self.generate_mip_maps();
        Ok(())
    }

    ///
    /// Creates a new color target cube map.
    ///
    pub fn new_empty(
        context: &Context,
        width: u32,
        height: u32,
        min_filter: Interpolation,
        mag_filter: Interpolation,
        mip_map_filter: Option<Interpolation>,
        wrap_s: Wrapping,
        wrap_t: Wrapping,
        wrap_r: Wrapping,
        format: Format,
    ) -> ThreeDResult<Self> {
        let id = generate(context)?;
        let number_of_mip_maps = calculate_number_of_mip_maps(mip_map_filter, width, height);
        set_parameters(
            context,
            &id,
            consts::TEXTURE_CUBE_MAP,
            min_filter,
            mag_filter,
            if number_of_mip_maps == 1 {
                None
            } else {
                mip_map_filter
            },
            wrap_s,
            wrap_t,
            Some(wrap_r),
        );
        context.bind_texture(consts::TEXTURE_CUBE_MAP, &id);
        context.tex_storage_2d(
            consts::TEXTURE_CUBE_MAP,
            number_of_mip_maps,
            T::internal_format(format)?,
            width,
            height,
        );
        let tex = Self {
            context: context.clone(),
            id,
            width,
            height,
            number_of_mip_maps,
            format,
            is_hdr: T::bits_per_channel() > 8,
            _dummy: T::default(),
        };
        tex.generate_mip_maps();
        Ok(tex)
    }

    ///
    /// Creates a new cube texture generated from the equirectangular texture given as input.
    ///
    pub fn new_from_equirectangular<T_: TextureDataType>(
        context: &Context,
        cpu_texture: &CPUTexture<T_>,
    ) -> ThreeDResult<Self> {
        let texture = Self::new_empty(
            &context,
            cpu_texture.width / 4,
            cpu_texture.width / 4,
            Interpolation::Linear,
            Interpolation::Linear,
            Some(Interpolation::Linear),
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            Format::RGBA,
        )?;

        {
            let map = Texture2D::new(context, cpu_texture)?;
            let fragment_shader_source = "uniform sampler2D equirectangularMap;
            const vec2 invAtan = vec2(0.1591, 0.3183);
            
            in vec3 pos;
            layout (location = 0) out vec4 outColor;
            
            vec2 sample_spherical_map(vec3 v)
            {
                vec2 uv = vec2(atan(v.z, v.x), asin(v.y));
                uv *= invAtan;
                uv += 0.5;
                return vec2(uv.x, 1.0 - uv.y);
            }
            
            void main()
            {		
                vec2 uv = sample_spherical_map(normalize(pos));
                outColor = vec4(texture(equirectangularMap, uv).rgb, 1.0);
            }";
            let program = ImageCubeEffect::new(context, fragment_shader_source)?;
            let render_target = RenderTargetCubeMap::new_color(context, &texture)?;
            let viewport = Viewport::new_at_origo(texture.width(), texture.height());
            let projection = perspective(degrees(90.0), viewport.aspect(), 0.1, 10.0);

            for side in CubeMapSide::iter() {
                program.use_texture("equirectangularMap", &map)?;
                program.apply(
                    &render_target,
                    side,
                    ClearState::default(),
                    RenderStates::default(),
                    projection,
                    viewport,
                )?;
            }
        }
        Ok(texture)
    }

    pub fn write(
        &self,
        side: CubeMapSide,
        clear_state: ClearState,
        render: impl FnOnce() -> ThreeDResult<()>,
    ) -> ThreeDResult<()> {
        RenderTargetCubeMap::new_color(&self.context, &self)?.write(side, clear_state, render)
    }

    pub fn write_to_mip_level(
        &self,
        side: CubeMapSide,
        mip_level: u32,
        clear_state: ClearState,
        render: impl FnOnce() -> ThreeDResult<()>,
    ) -> ThreeDResult<()> {
        RenderTargetCubeMap::new_color(&self.context, &self)?.write_to_mip_level(
            side,
            mip_level,
            clear_state,
            render,
        )
    }

    pub(in crate::core) fn generate_mip_maps(&self) {
        if self.number_of_mip_maps > 1 {
            self.context
                .bind_texture(consts::TEXTURE_CUBE_MAP, &self.id);
            self.context.generate_mipmap(consts::TEXTURE_CUBE_MAP);
        }
    }

    pub(in crate::core) fn bind_as_color_target(
        &self,
        side: CubeMapSide,
        channel: u32,
        mip_level: u32,
    ) {
        self.context.framebuffer_texture_2d(
            consts::DRAW_FRAMEBUFFER,
            consts::COLOR_ATTACHMENT0 + channel,
            side.to_const(),
            &self.id,
            mip_level,
        );
    }
}

impl<T: TextureDataType> TextureCube for TextureCubeMap<T> {
    fn bind(&self, location: u32) {
        bind_at(&self.context, &self.id, consts::TEXTURE_CUBE_MAP, location);
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
    fn format(&self) -> Format {
        self.format
    }
    fn is_hdr(&self) -> bool {
        self.is_hdr
    }
}

impl<T: TextureDataType> Drop for TextureCubeMap<T> {
    fn drop(&mut self) {
        self.context.delete_texture(&self.id);
    }
}
