use core::f32;

use taffy::{
    AlignContent, AlignItems, AvailableSpace, Dimension, FlexDirection, Layout as TaffyLayout,
    LengthPercentage, LengthPercentageAuto, NodeId, Rect, Size, Style as TaffyStyle, TaffyTree,
    style_helpers::{TaffyAuto, TaffyZero, length, percent},
};

use crate::{
    font::{self, FontManager},
    render::{self, RenderNode, RenderNodeKind},
};

#[derive(Debug)]
pub struct LayoutNode<'a> {
    pub render: &'a RenderNode,
    pub layout: LayoutRect,
    pub children: Vec<LayoutNode<'a>>,
}

#[derive(Debug)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn build_layout_tree<'a>(
    render_tree: &'a RenderNode,
    width: usize,
    height: usize,
    font_mgr: &font::FontManager,
) -> anyhow::Result<LayoutNode<'a>> {
    let mut taffy_tree = TaffyTree::new();

    let root = build_taffy_tree(render_tree, &mut taffy_tree, false, &(None, None))?;

    taffy_tree.compute_layout_with_measure(
        root.id,
        Size {
            width: length(width as f32),
            height: length(height as f32),
        },
        |known_dimensions, available_space, _node_id, ctx, _style| {
            measure(known_dimensions, available_space, ctx, font_mgr)
        },
    )?;


    Ok(build_render_tree_inner(&root, &taffy_tree)?)
}

fn measure(
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    ctx: Option<&mut &RenderNode>,
    font_mgr: &FontManager,
) -> Size<f32> {
    if let Size {
        width: Some(width),
        height: Some(height),
    } = known_dimensions
    {
        return Size { width, height };
    }

    match &ctx {
        Some(RenderNode {
            kind: RenderNodeKind::Text { style, text },
            children: _,
            style: _,
        }) => {
            let width = known_dimensions
                .width
                .unwrap_or_else(|| match available_space.width {
                    AvailableSpace::Definite(width) => width,
                    AvailableSpace::MinContent => f32::MAX,
                    AvailableSpace::MaxContent => f32::MAX,
                });

            let height = known_dimensions
                .height
                .unwrap_or_else(|| match available_space.height {
                    AvailableSpace::Definite(height) => height,
                    AvailableSpace::MinContent => f32::MAX,
                    AvailableSpace::MaxContent => f32::MAX,
                });

            let mut paragraph = font_mgr.create_paragraph(style, text);
            paragraph.layout(width);

            match available_space.width {
                AvailableSpace::Definite(_) => Size {
                    width: paragraph.longest_line().ceil().min(width),
                    height: paragraph.height().min(height),
                },
                AvailableSpace::MinContent => Size {
                    width: paragraph.min_intrinsic_width().ceil().min(width),
                    height: paragraph.height().min(height),
                },
                AvailableSpace::MaxContent => Size {
                    width: paragraph.max_intrinsic_width().ceil().min(width),
                    height: paragraph.height().min(height),
                },
            }
        }
        Some(RenderNode {
            kind:
                RenderNodeKind::Image {
                    style: _,
                    image: Some(image),
                },
            children: _,
            style: _,
        }) => {
            let img_width = image.width() as f32;
            let img_height = image.height() as f32;

            match (known_dimensions.width, known_dimensions.height) {
                (Some(width), Some(height)) => Size { width, height },
                (Some(width), None) => Size {
                    width,
                    height: img_height * (width / img_width),
                },
                (None, Some(height)) => Size {
                    width: img_width * (height / img_height),
                    height,
                },
                (None, None) => Size {
                    width: img_width,
                    height: img_height,
                },
            }
        }
        Some(RenderNode {
            kind:
                RenderNodeKind::Svg {
                    style: _,
                    svg: Some(svg),
                },
            children: _,
            style: _,
        }) => match svg.root().view_box() {
            Some(rect) => {
                let img_width = rect.width() as f32;
                let img_height = rect.height() as f32;

                match (known_dimensions.width, known_dimensions.height) {
                    (Some(width), Some(height)) => Size { width, height },
                    (Some(width), None) => Size {
                        width,
                        height: img_height * (width / img_width),
                    },
                    (None, Some(height)) => Size {
                        width: img_width * (height / img_height),
                        height,
                    },
                    (None, None) => Size {
                        width: img_width,
                        height: img_height,
                    },
                }
            }
            None => Size::ZERO,
        },
        _ => Size::ZERO,
    }
}

fn build_render_tree_inner<'a>(
    out: &TaffyOut<'a>,
    tree: &TaffyTree<&'a RenderNode>,
) -> anyhow::Result<LayoutNode<'a>> {
    Ok(LayoutNode {
        render: out.node,
        layout: tree.layout(out.id)?.into(),
        children: out
            .children
            .iter()
            .map(|c| build_render_tree_inner(c, tree))
            .collect::<anyhow::Result<Vec<_>>>()?,
    })
}

struct TaffyOut<'a> {
    id: NodeId,
    node: &'a RenderNode,
    children: Vec<TaffyOut<'a>>,
}

fn build_taffy_tree<'a>(
    tree: &'a RenderNode,
    taffy_tree: &mut TaffyTree<&'a RenderNode>,
    parent_stack: bool,
    parent_place: &(Option<render::styles::Place>, Option<render::styles::Place>),
) -> anyhow::Result<TaffyOut<'a>> {
    let is_stack = match &tree.kind {
        RenderNodeKind::Stack { style: _ } => true,
        _ => false,
    };

    let places = match &tree.kind {
        RenderNodeKind::Stack { style } => (style.place_x.clone(), style.place_y.clone()),
        _ => (None, None),
    };

    let children = tree
        .children
        .iter()
        .map(|c| build_taffy_tree(c, taffy_tree, is_stack, &places))
        .collect::<anyhow::Result<Vec<_>>>()?;

    let children_ids = children.iter().map(|c| c.id).collect::<Vec<_>>();

    let id = taffy_tree.new_with_children(
        make_taffy_style(tree, parent_stack, parent_place),
        &children_ids,
    )?;

    taffy_tree.set_node_context(id, Some(tree))?;

    Ok(TaffyOut {
        id,
        node: tree,
        children,
    })
}

fn make_taffy_style(
    node: &RenderNode,
    parent_stack: bool,
    parent_place: &(Option<render::styles::Place>, Option<render::styles::Place>),
) -> TaffyStyle {
    let mut taffy_style = TaffyStyle {
        ..Default::default()
    };

    if parent_stack {
        taffy_style.position = taffy::Position::Absolute;

        let (place_x, place_y) = parent_place;

        if let Some(val) = place_x.clone() {
            match val {
                render::styles::Place::Start => {
                    taffy_style.inset.left = percent(0.0);
                }
                render::styles::Place::Center => {
                    taffy_style.inset.left = percent(0.5);
                    taffy_style.inset.right = percent(0.5);
                }
                render::styles::Place::End => {
                    taffy_style.inset.right = percent(0.0);
                }
            }
        }

        if let Some(val) = place_y.clone() {
            match val {
                render::styles::Place::Start => {
                    taffy_style.inset.top = percent(0.0);
                }
                render::styles::Place::Center => {
                    taffy_style.inset.top = percent(0.5);
                    taffy_style.inset.bottom = percent(0.5);
                }
                render::styles::Place::End => {
                    taffy_style.inset.bottom = percent(0.0);
                }
            }
        }
    }

    if let Some(val) = node.style.width {
        taffy_style.size.width = val.into();
    }
    if let Some(val) = node.style.height {
        taffy_style.size.height = val.into();
    }
    if let Some(val) = node.style.max_width {
        taffy_style.max_size.width = val.into();
    }
    if let Some(val) = node.style.max_height {
        taffy_style.max_size.height = val.into();
    }
    if let Some(val) = node.style.min_width {
        taffy_style.min_size.width = val.into();
    }
    if let Some(val) = node.style.min_height {
        taffy_style.min_size.height = val.into();
    }
    if let Some(val) = node.style.margin {
        taffy_style.margin = val.into();
    }
    if let Some(val) = node.style.padding {
        taffy_style.padding = val.into();
    }
    if let Some(val) = node.style.border {
        taffy_style.border = length(val);
    }
    if let Some(val) = node.style.grow {
        taffy_style.flex_grow = val;
    }
    if let Some(val) = node.style.shrink {
        taffy_style.flex_shrink = val;
    }
    if let Some(val) = node.style.basis {
        taffy_style.flex_basis = length(val);
    }

    match &node.kind {
        RenderNodeKind::Box { style } => {
            if let Some(val) = style.gap {
                // TODO: GAP
                taffy_style.gap.width = val.into();
                taffy_style.gap.height = val.into();
            }

            if let Some(val) = style.direction {
                taffy_style.flex_direction = val.into();
            }
            if let Some(val) = style.main {
                taffy_style.justify_content = Some(val.into());
            }
            if let Some(val) = style.cross {
                taffy_style.align_items = Some(val.into());
            }
        }

        _ => {}
    }

    taffy_style
}

impl From<render::styles::Size> for Dimension {
    fn from(size: render::styles::Size) -> Self {
        match size {
            render::styles::Size::Length(v) => length(v),
            render::styles::Size::Percent(v) => percent(v),
            render::styles::Size::Auto => Dimension::AUTO,
            render::styles::Size::Zero => Dimension::ZERO,
        }
    }
}

impl From<render::styles::Size> for LengthPercentageAuto {
    fn from(size: render::styles::Size) -> Self {
        match size {
            render::styles::Size::Length(v) => length(v),
            render::styles::Size::Percent(v) => percent(v),
            render::styles::Size::Auto => LengthPercentageAuto::AUTO,
            render::styles::Size::Zero => LengthPercentageAuto::ZERO,
        }
    }
}

impl From<render::styles::Size> for LengthPercentage {
    fn from(size: render::styles::Size) -> Self {
        match size {
            render::styles::Size::Length(v) => length(v),
            render::styles::Size::Percent(v) => percent(v),
            render::styles::Size::Auto => LengthPercentage::ZERO,
            render::styles::Size::Zero => LengthPercentage::ZERO,
        }
    }
}

impl From<render::styles::EdgeInsets> for Rect<LengthPercentageAuto> {
    fn from(insets: render::styles::EdgeInsets) -> Self {
        Rect {
            left: insets.left.into(),
            right: insets.right.into(),
            top: insets.top.into(),
            bottom: insets.bottom.into(),
        }
    }
}

impl From<render::styles::EdgeInsets> for Rect<LengthPercentage> {
    fn from(insets: render::styles::EdgeInsets) -> Self {
        Rect {
            left: insets.left.into(),
            right: insets.right.into(),
            top: insets.top.into(),
            bottom: insets.bottom.into(),
        }
    }
}
impl From<render::styles::BoxDirection> for FlexDirection {
    fn from(dir: render::styles::BoxDirection) -> Self {
        match dir {
            render::styles::BoxDirection::Column => FlexDirection::Column,
            render::styles::BoxDirection::Row => FlexDirection::Row,
        }
    }
}
impl From<render::styles::MainAlign> for AlignContent {
    fn from(align: render::styles::MainAlign) -> Self {
        match align {
            render::styles::MainAlign::Center => AlignContent::Center,
            render::styles::MainAlign::Start => AlignContent::Start,
            render::styles::MainAlign::End => AlignContent::End,
            render::styles::MainAlign::FlexStart => AlignContent::FlexStart,
            render::styles::MainAlign::FlexEnd => AlignContent::FlexEnd,
            render::styles::MainAlign::Between => AlignContent::SpaceBetween,
            render::styles::MainAlign::Around => AlignContent::SpaceAround,
            render::styles::MainAlign::Evenly => AlignContent::SpaceEvenly,
            render::styles::MainAlign::Stretch => AlignContent::Stretch,
        }
    }
}
impl From<render::styles::CrossAlign> for AlignItems {
    fn from(align: render::styles::CrossAlign) -> Self {
        match align {
            render::styles::CrossAlign::Start => AlignItems::Start,
            render::styles::CrossAlign::End => AlignItems::End,
            render::styles::CrossAlign::FlexStart => AlignItems::FlexStart,
            render::styles::CrossAlign::FlexEnd => AlignItems::FlexEnd,
            render::styles::CrossAlign::Center => AlignItems::Center,
            render::styles::CrossAlign::Baseline => AlignItems::Baseline,
            render::styles::CrossAlign::Stretch => AlignItems::Stretch,
        }
    }
}

impl From<&TaffyLayout> for LayoutRect {
    fn from(layout: &TaffyLayout) -> Self {
        LayoutRect {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        }
    }
}
