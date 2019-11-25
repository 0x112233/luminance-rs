//! Framebuffers and utility types and functions.
//!
//! Framebuffers are at the core of rendering. They’re the support of rendering operations and can
//! be used to highly enhance the visual aspect of a render. You’re always provided with at least
//! one framebuffer, `Framebuffer::back_buffer`. That function returns a framebuffer that represents –
//! for short – the current back framebuffer. You can render to that framebuffer and when you
//! *swap* the buffers, your render appears in the front framebuffer (likely your screen).
//!
//! # Framebuffers
//!
//! A framebuffer is an object maintaining the required GPU state to hold images you render to. It
//! gathers two important concepts:
//!
//!   - *Color buffers*.
//!   - *Depth buffers*.
//!
//! The *color buffers* hold the color images you render to. A framebuffer can hold several of them
//! with different color formats. The *depth buffers* hold the depth images you render to.
//! Framebuffers can hold only one depth buffer.
//!
//! # Framebuffer slots
//!
//! A framebuffer slot contains either its color buffers or its depth buffer. Sometimes, you might
//! find it handy to have no slot at all for a given type of buffer. In that case, we use `()`.
//!
//! The slots are a way to convert the different formats you use for your framebuffers’ buffers into
//! their respective texture representation so that you can handle the corresponding texels.
//!
//! Color buffers are abstracted by `ColorSlot` and the depth buffer by `DepthSlot`.

use crate::context::GraphicsContext;
use crate::pixel::{ColorPixel, DepthPixel, PixelFormat, RenderablePixel};
use crate::texture::{Dim2, Dimensionable, Layerable};

pub trait Framebuffer<C, L, D>: Sized
where
  L: Layerable,
  D: Dimensionable,
{
  type BackBuffer;

  type Textures;

  type ColorSlot: ColorSlot<C, L, D, Self::Textures>;

  type DepthSlot: DepthSlot<C, L, D, Self::Textures>;

  type Err;

  /// Get the back buffer with the given dimension.
  fn back_buffer(
    ctx: &mut C,
    size: <Dim2 as Dimensionable>::Size,
  ) -> Result<Self::BackBuffer, Self::Err>;

  /// Create a new framebuffer.
  ///
  /// You’re always handed at least the base level of the texture. If you require any *additional*
  /// levels, you can pass the number via the `mipmaps` parameter.
  fn new(ctx: &mut C, size: D::Size, mipmaps: usize) -> Result<Self, Self::Err>;

  /// Dimension of the framebuffer.
  fn dimension(&self) -> D::Size;

  /// Access the underlying color slot.
  fn color_slot(&self) -> &Self::ColorSlot;

  /// Access the underlying depth slot.
  fn depth_slot(&self) -> &Self::DepthSlot;
}

pub trait ColorSlot<C, L, D, I>
where
  L: Layerable,
  D: Dimensionable,
{
  type ColorTextures;

  const COLOR_FORMATS: &'static [PixelFormat];

  /// Reify a list of raw textures.
  fn reify_textures(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    textures: &mut I,
  ) -> Self::ColorTextures;
}

impl<C, L, D, I> ColorSlot<C, L, D, I> for ()
where
  L: Layerable,
  D: Dimensionable,
{
  type ColorTextures = ();

  const COLOR_FORMATS: &'static [PixelFormat] = &[];

  fn reify_textures(_: &mut C, _: D::Size, _: usize, _: &mut I) -> Self::ColorTextures {
    ()
  }
}

impl<C, L, D, I, P> ColorSlot<C, L, D, I> for P
where
  L: Layerable,
  D: Dimensionable,
  I: ReifyTexture<C, L, D, P>,
  Self: ColorPixel + RenderablePixel,
{
  type ColorTextures = <I as ReifyTexture<C, L, D, P>>::Texture;

  const COLOR_FORMATS: &'static [PixelFormat] = &[Self::PIXEL_FORMAT];

  fn reify_textures(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    state: &mut I,
  ) -> Self::ColorTextures {
    I::reify_texture(ctx, size, mipmaps, state)
  }
}

macro_rules! impl_color_slot_tuple {
  ($($pf:ident),*) => {
    impl<C, L, D, I, $($pf),*> ColorSlot<C, L, D, I> for ($($pf),*)
    where
      L: Layerable,
      D: Dimensionable,
      $(
        I: ReifyTexture<C, L, D, $pf>,
        $pf: ColorPixel + RenderablePixel
      ),* {
      type ColorTextures = ($(<I as ReifyTexture<C, L, D, $pf>>::Texture),*);

      const COLOR_FORMATS: &'static [PixelFormat] = &[$($pf::PIXEL_FORMAT),*];

      fn reify_textures(
        ctx: &mut C,
        size: D::Size,
        mipmaps: usize,
        state: &mut I,
      ) -> Self::ColorTextures {
        ( $( <I as ReifyTexture<C, L, D, $pf>>::reify_texture(ctx, size, mipmaps, state) ),* )
      }
    }
  }
}

macro_rules! impl_color_slot_tuples {
  ($first:ident , $second:ident) => {
    // stop at pairs
    impl_color_slot_tuple!($first, $second);
  };

  ($first:ident , $($pf:ident),*) => {
    // implement the same list without the first type (reduced by one)
    impl_color_slot_tuples!($($pf),*);
    // implement the current list
    impl_color_slot_tuple!($first, $($pf),*);
  };
}

impl_color_slot_tuples!(PF, PE, PD, PC, PB, PA, P9, P8, P7, P6, P5, P4, P3, P2, P1, P0);

pub trait DepthSlot<C, L, D, I>
where
  L: Layerable,
  D: Dimensionable,
{
  type DepthTexture;

  const DEPTH_FORMAT: Option<PixelFormat>;

  fn reify_texture(ctx: &mut C, size: D::Size, mipmaps: usize, state: &mut I)
    -> Self::DepthTexture;
}

impl<C, L, D, I> DepthSlot<C, L, D, I> for ()
where
  L: Layerable,
  D: Dimensionable,
{
  type DepthTexture = ();

  const DEPTH_FORMAT: Option<PixelFormat> = None;

  fn reify_texture(_: &mut C, _: D::Size, _: usize, _: &mut I) -> Self::DepthTexture {
    ()
  }
}

impl<C, L, D, I, P> DepthSlot<C, L, D, I> for P
where
  L: Layerable,
  D: Dimensionable,
  I: ReifyTexture<C, L, D, Self>,
  Self: DepthPixel,
{
  type DepthTexture = <I as ReifyTexture<C, L, D, Self>>::Texture;

  const DEPTH_FORMAT: Option<PixelFormat> = Some(Self::PIXEL_FORMAT);

  fn reify_texture(
    ctx: &mut C,
    size: D::Size,
    mipmaps: usize,
    state: &mut I,
  ) -> Self::DepthTexture {
    I::reify_texture(ctx, size, mipmaps, state)
  }
}

pub trait ReifyTexture<C, L, D, P>
where
  L: Layerable,
  D: Dimensionable,
{
  type Texture;

  fn reify_texture(ctx: &mut C, size: D::Size, mipmaps: usize, state: &mut Self) -> Self::Texture;
}
