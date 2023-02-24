// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;

use usvg_text_layout::TreeTextToPath;
use xmlwriter::XmlWriter;

use rosvgtree::{AttributeId, ElementId};
use usvg::*;

/// Extension trait for exporting usvg::Tree
pub trait TreeExportExt {
    /// Converts text nodes to paths. Must be called before exporting, otherwise text won't be exported at all.
    fn convert_text_to_paths(&mut self, fontdb: &usvg_text_layout::fontdb::Database);
    /// Exports the tree. When text nodes should be exported, `convert_text_to_paths()` must be called first.
    fn to_string(&self, opt: &ExportOptions) -> String;
}

impl TreeExportExt for usvg::Tree {
    fn convert_text_to_paths(&mut self, fontdb: &usvg_text_layout::fontdb::Database) {
        self.convert_text(fontdb);
    }

    fn to_string(&self, opt: &ExportOptions) -> String {
        convert(self, opt)
    }
}

/// Export options
#[derive(Clone, Default, Debug)]
pub struct ExportOptions {
    /// Used to add a custom prefix to each element ID during writing.
    pub id_prefix: Option<String>,
    /// `xmlwriter` options.
    pub writer_opts: xmlwriter::Options,
}

pub(crate) fn convert(tree: &Tree, opt: &ExportOptions) -> String {
    let mut xml = XmlWriter::new(opt.writer_opts);

    xml.start_svg_element(ElementId::Svg);
    xml.write_svg_attribute(AttributeId::Width, &tree.size.width());
    xml.write_svg_attribute(AttributeId::Height, &tree.size.height());
    xml.write_viewbox(&tree.view_box);
    xml.write_attribute("xmlns", "http://www.w3.org/2000/svg");
    if has_xlink(tree) {
        xml.write_attribute("xmlns:xlink", "http://www.w3.org/1999/xlink");
    }

    xml.start_svg_element(ElementId::Defs);
    conv_defs(tree, opt, &mut xml);
    xml.end_element();

    conv_elements(&tree.root, false, opt, &mut xml);

    xml.end_document()
}

fn collect_clip_paths(root: Node, clip_paths: &mut Vec<Rc<ClipPath>>) {
    for n in root.descendants() {
        if let NodeKind::Group(ref g) = *n.borrow() {
            if let Some(ref cp) = g.clip_path {
                if !clip_paths.iter().any(|other| Rc::ptr_eq(cp, other)) {
                    clip_paths.push(cp.clone());
                }

                if let Some(ref cp) = cp.clip_path {
                    collect_clip_paths(cp.root.clone(), clip_paths);
                }

                collect_clip_paths(cp.root.clone(), clip_paths);
            }
        }
    }
}

fn collect_masks(root: Node, masks: &mut Vec<Rc<Mask>>) {
    for n in root.descendants() {
        if let NodeKind::Group(ref g) = *n.borrow() {
            if let Some(ref mask) = g.mask {
                if !masks.iter().any(|other| Rc::ptr_eq(mask, other)) {
                    masks.push(mask.clone());
                }

                if let Some(ref mask) = mask.mask {
                    collect_masks(mask.root.clone(), masks);
                }

                collect_masks(mask.root.clone(), masks);
            }
        }
    }
}

fn collect_paint_servers(root: Node, paint_servers: &mut Vec<Paint>) {
    for n in root.descendants() {
        if let NodeKind::Path(ref path) = *n.borrow() {
            if let Some(ref fill) = path.fill {
                if !paint_servers.contains(&fill.paint) {
                    paint_servers.push(fill.paint.clone());
                }

                if let Paint::Pattern(ref patt) = fill.paint {
                    collect_paint_servers(patt.root.clone(), paint_servers);
                }
            }

            if let Some(ref stroke) = path.stroke {
                if !paint_servers.contains(&stroke.paint) {
                    paint_servers.push(stroke.paint.clone());
                }

                if let Paint::Pattern(ref patt) = stroke.paint {
                    collect_paint_servers(patt.root.clone(), paint_servers);
                }
            }
        }
    }
}

fn collect_filters(root: Node, filters: &mut Vec<Rc<filter::Filter>>) {
    for n in root.descendants() {
        if let NodeKind::Group(ref g) = *n.borrow() {
            for filter in &g.filters {
                if !filters.iter().any(|other| Rc::ptr_eq(other, filter)) {
                    filters.push(filter.clone());
                }
            }
        }
    }
}

fn conv_filters(tree: &Tree, opt: &ExportOptions, xml: &mut XmlWriter) {
    let mut filters = Vec::new();
    collect_filters(tree.root.clone(), &mut filters);

    let mut written_fe_image_nodes: Vec<String> = Vec::new();
    for filter in filters {
        for fe in &filter.primitives {
            if let filter::Kind::Image(ref img) = fe.kind {
                if let filter::ImageKind::Use(ref node) = img.data {
                    if !written_fe_image_nodes.iter().any(|id| id == &*node.id()) {
                        conv_element(node, false, opt, xml);
                        written_fe_image_nodes.push(node.id().to_string());
                    }
                }
            }
        }

        xml.start_svg_element(ElementId::Filter);
        xml.write_id_attribute(&filter.id, opt);
        xml.write_rect_attrs(filter.rect);
        xml.write_units(
            AttributeId::FilterUnits,
            filter.units,
            Units::ObjectBoundingBox,
        );
        xml.write_units(
            AttributeId::PrimitiveUnits,
            filter.primitive_units,
            Units::UserSpaceOnUse,
        );

        for fe in &filter.primitives {
            match fe.kind {
                filter::Kind::DropShadow(ref shadow) => {
                    xml.start_svg_element(ElementId::FeDropShadow);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &shadow.input);
                    xml.write_attribute_fmt(
                        AttributeId::StdDeviation.to_str(),
                        format_args!("{} {}", shadow.std_dev_x.get(), shadow.std_dev_y.get()),
                    );
                    xml.write_svg_attribute(AttributeId::Dx, &shadow.dx);
                    xml.write_svg_attribute(AttributeId::Dy, &shadow.dy);
                    xml.write_color(AttributeId::FloodColor, shadow.color);
                    xml.write_svg_attribute(AttributeId::FloodOpacity, &shadow.opacity.get());
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);
                    xml.end_element();
                }
                filter::Kind::GaussianBlur(ref blur) => {
                    xml.start_svg_element(ElementId::FeGaussianBlur);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &blur.input);
                    xml.write_attribute_fmt(
                        AttributeId::StdDeviation.to_str(),
                        format_args!("{} {}", blur.std_dev_x.get(), blur.std_dev_y.get()),
                    );
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);
                    xml.end_element();
                }
                filter::Kind::Offset(ref offset) => {
                    xml.start_svg_element(ElementId::FeOffset);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &offset.input);
                    xml.write_svg_attribute(AttributeId::Dx, &offset.dx);
                    xml.write_svg_attribute(AttributeId::Dy, &offset.dy);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);
                    xml.end_element();
                }
                filter::Kind::Blend(ref blend) => {
                    xml.start_svg_element(ElementId::FeBlend);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &blend.input1);
                    xml.write_filter_input(AttributeId::In2, &blend.input2);
                    xml.write_svg_attribute(
                        AttributeId::Mode,
                        match blend.mode {
                            usvg::BlendMode::Normal => "normal",
                            usvg::BlendMode::Multiply => "multiply",
                            usvg::BlendMode::Screen => "screen",
                            usvg::BlendMode::Overlay => "overlay",
                            usvg::BlendMode::Darken => "darken",
                            usvg::BlendMode::Lighten => "lighten",
                            usvg::BlendMode::ColorDodge => "color-dodge",
                            usvg::BlendMode::ColorBurn => "color-burn",
                            usvg::BlendMode::HardLight => "hard-light",
                            usvg::BlendMode::SoftLight => "soft-light",
                            usvg::BlendMode::Difference => "difference",
                            usvg::BlendMode::Exclusion => "exclusion",
                            usvg::BlendMode::Hue => "hue",
                            usvg::BlendMode::Saturation => "saturation",
                            usvg::BlendMode::Color => "color",
                            usvg::BlendMode::Luminosity => "luminosity",
                        },
                    );
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);
                    xml.end_element();
                }
                filter::Kind::Flood(ref flood) => {
                    xml.start_svg_element(ElementId::FeFlood);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_color(AttributeId::FloodColor, flood.color);
                    xml.write_svg_attribute(AttributeId::FloodOpacity, &flood.opacity.get());
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);
                    xml.end_element();
                }
                filter::Kind::Composite(ref composite) => {
                    xml.start_svg_element(ElementId::FeComposite);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &composite.input1);
                    xml.write_filter_input(AttributeId::In2, &composite.input2);
                    xml.write_svg_attribute(
                        AttributeId::Operator,
                        match composite.operator {
                            filter::CompositeOperator::Over => "over",
                            filter::CompositeOperator::In => "in",
                            filter::CompositeOperator::Out => "out",
                            filter::CompositeOperator::Atop => "atop",
                            filter::CompositeOperator::Xor => "xor",
                            filter::CompositeOperator::Arithmetic { .. } => "arithmetic",
                        },
                    );

                    if let filter::CompositeOperator::Arithmetic { k1, k2, k3, k4 } =
                        composite.operator
                    {
                        xml.write_svg_attribute(AttributeId::K1, &k1);
                        xml.write_svg_attribute(AttributeId::K2, &k2);
                        xml.write_svg_attribute(AttributeId::K3, &k3);
                        xml.write_svg_attribute(AttributeId::K4, &k4);
                    }

                    xml.write_svg_attribute(AttributeId::Result, &fe.result);
                    xml.end_element();
                }
                filter::Kind::Merge(ref merge) => {
                    xml.start_svg_element(ElementId::FeMerge);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);
                    for input in &merge.inputs {
                        xml.start_svg_element(ElementId::FeMergeNode);
                        xml.write_filter_input(AttributeId::In, input);
                        xml.end_element();
                    }

                    xml.end_element();
                }
                filter::Kind::Tile(ref tile) => {
                    xml.start_svg_element(ElementId::FeTile);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &tile.input);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);
                    xml.end_element();
                }
                filter::Kind::Image(ref img) => {
                    xml.start_svg_element(ElementId::FeImage);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_aspect(img.aspect);
                    xml.write_svg_attribute(
                        AttributeId::ImageRendering,
                        match img.rendering_mode {
                            ImageRendering::OptimizeQuality => "optimizeQuality",
                            ImageRendering::OptimizeSpeed => "optimizeSpeed",
                        },
                    );
                    match img.data {
                        filter::ImageKind::Image(ref kind) => {
                            xml.write_image_data(kind);
                        }
                        filter::ImageKind::Use(ref node) => {
                            let prefix = opt.id_prefix.as_deref().unwrap_or_default();
                            xml.write_attribute_fmt(
                                "xlink:href",
                                format_args!("#{}{}", prefix, node.id()),
                            );
                        }
                    }

                    xml.write_svg_attribute(AttributeId::Result, &fe.result);
                    xml.end_element();
                }
                filter::Kind::ComponentTransfer(ref transfer) => {
                    xml.start_svg_element(ElementId::FeComponentTransfer);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &transfer.input);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);

                    xml.write_filter_transfer_function(ElementId::FeFuncR, &transfer.func_r);
                    xml.write_filter_transfer_function(ElementId::FeFuncG, &transfer.func_g);
                    xml.write_filter_transfer_function(ElementId::FeFuncB, &transfer.func_b);
                    xml.write_filter_transfer_function(ElementId::FeFuncA, &transfer.func_a);

                    xml.end_element();
                }
                filter::Kind::ColorMatrix(ref matrix) => {
                    xml.start_svg_element(ElementId::FeColorMatrix);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &matrix.input);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);

                    match matrix.kind {
                        filter::ColorMatrixKind::Matrix(ref values) => {
                            xml.write_svg_attribute(AttributeId::Type, "matrix");
                            xml.write_numbers(AttributeId::Values, values);
                        }
                        filter::ColorMatrixKind::Saturate(value) => {
                            xml.write_svg_attribute(AttributeId::Type, "saturate");
                            xml.write_svg_attribute(AttributeId::Values, &value.get());
                        }
                        filter::ColorMatrixKind::HueRotate(angle) => {
                            xml.write_svg_attribute(AttributeId::Type, "hueRotate");
                            xml.write_svg_attribute(AttributeId::Values, &angle);
                        }
                        filter::ColorMatrixKind::LuminanceToAlpha => {
                            xml.write_svg_attribute(AttributeId::Type, "luminanceToAlpha");
                        }
                    }

                    xml.end_element();
                }
                filter::Kind::ConvolveMatrix(ref matrix) => {
                    xml.start_svg_element(ElementId::FeConvolveMatrix);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &matrix.input);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);

                    xml.write_attribute_fmt(
                        AttributeId::Order.to_str(),
                        format_args!("{} {}", matrix.matrix.columns, matrix.matrix.rows),
                    );
                    xml.write_numbers(AttributeId::KernelMatrix, &matrix.matrix.data);
                    xml.write_svg_attribute(AttributeId::Divisor, &matrix.divisor.value());
                    xml.write_svg_attribute(AttributeId::Bias, &matrix.bias);
                    xml.write_svg_attribute(AttributeId::TargetX, &matrix.matrix.target_x);
                    xml.write_svg_attribute(AttributeId::TargetY, &matrix.matrix.target_y);
                    xml.write_svg_attribute(
                        AttributeId::EdgeMode,
                        match matrix.edge_mode {
                            filter::EdgeMode::None => "none",
                            filter::EdgeMode::Duplicate => "duplicate",
                            filter::EdgeMode::Wrap => "wrap",
                        },
                    );
                    xml.write_svg_attribute(
                        AttributeId::PreserveAlpha,
                        if matrix.preserve_alpha {
                            "true"
                        } else {
                            "false"
                        },
                    );

                    xml.end_element();
                }
                filter::Kind::Morphology(ref morphology) => {
                    xml.start_svg_element(ElementId::FeMorphology);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &morphology.input);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);

                    xml.write_svg_attribute(
                        AttributeId::Operator,
                        match morphology.operator {
                            filter::MorphologyOperator::Erode => "erode",
                            filter::MorphologyOperator::Dilate => "dilate",
                        },
                    );
                    xml.write_attribute_fmt(
                        AttributeId::Radius.to_str(),
                        format_args!(
                            "{} {}",
                            morphology.radius_x.get(),
                            morphology.radius_y.get()
                        ),
                    );

                    xml.end_element();
                }
                filter::Kind::DisplacementMap(ref map) => {
                    xml.start_svg_element(ElementId::FeDisplacementMap);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_filter_input(AttributeId::In, &map.input1);
                    xml.write_filter_input(AttributeId::In2, &map.input2);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);

                    xml.write_svg_attribute(AttributeId::Scale, &map.scale);

                    let mut write_channel = |c, aid| {
                        xml.write_svg_attribute(
                            aid,
                            match c {
                                filter::ColorChannel::R => "R",
                                filter::ColorChannel::G => "G",
                                filter::ColorChannel::B => "B",
                                filter::ColorChannel::A => "A",
                            },
                        );
                    };
                    write_channel(map.x_channel_selector, AttributeId::XChannelSelector);
                    write_channel(map.y_channel_selector, AttributeId::YChannelSelector);

                    xml.end_element();
                }
                filter::Kind::Turbulence(ref turbulence) => {
                    xml.start_svg_element(ElementId::FeTurbulence);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);

                    xml.write_point(AttributeId::BaseFrequency, turbulence.base_frequency);
                    xml.write_svg_attribute(AttributeId::NumOctaves, &turbulence.num_octaves);
                    xml.write_svg_attribute(AttributeId::Seed, &turbulence.seed);
                    xml.write_svg_attribute(
                        AttributeId::StitchTiles,
                        match turbulence.stitch_tiles {
                            true => "stitch",
                            false => "noStitch",
                        },
                    );
                    xml.write_svg_attribute(
                        AttributeId::Type,
                        match turbulence.kind {
                            filter::TurbulenceKind::FractalNoise => "fractalNoise",
                            filter::TurbulenceKind::Turbulence => "turbulence",
                        },
                    );

                    xml.end_element();
                }
                filter::Kind::DiffuseLighting(ref light) => {
                    xml.start_svg_element(ElementId::FeDiffuseLighting);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);

                    xml.write_svg_attribute(AttributeId::SurfaceScale, &light.surface_scale);
                    xml.write_svg_attribute(AttributeId::DiffuseConstant, &light.diffuse_constant);
                    xml.write_color(AttributeId::LightingColor, light.lighting_color);
                    write_light_source(&light.light_source, xml);

                    xml.end_element();
                }
                filter::Kind::SpecularLighting(ref light) => {
                    xml.start_svg_element(ElementId::FeSpecularLighting);
                    xml.write_filter_primitive_attrs(fe);
                    xml.write_svg_attribute(AttributeId::Result, &fe.result);

                    xml.write_svg_attribute(AttributeId::SurfaceScale, &light.surface_scale);
                    xml.write_svg_attribute(
                        AttributeId::SpecularConstant,
                        &light.specular_constant,
                    );
                    xml.write_svg_attribute(
                        AttributeId::SpecularExponent,
                        &light.specular_exponent,
                    );
                    xml.write_color(AttributeId::LightingColor, light.lighting_color);
                    write_light_source(&light.light_source, xml);

                    xml.end_element();
                }
            };
        }

        xml.end_element();
    }
}

fn conv_defs(tree: &Tree, opt: &ExportOptions, xml: &mut XmlWriter) {
    let mut paint_servers = Vec::new();
    collect_paint_servers(tree.root.clone(), &mut paint_servers);
    for paint in paint_servers {
        match paint {
            Paint::Color(_) => {}
            Paint::LinearGradient(lg) => {
                xml.start_svg_element(ElementId::LinearGradient);
                xml.write_id_attribute(&lg.id, opt);
                xml.write_svg_attribute(AttributeId::X1, &lg.x1);
                xml.write_svg_attribute(AttributeId::Y1, &lg.y1);
                xml.write_svg_attribute(AttributeId::X2, &lg.x2);
                xml.write_svg_attribute(AttributeId::Y2, &lg.y2);
                write_base_grad(&lg.base, xml);
                xml.end_element();
            }
            Paint::RadialGradient(rg) => {
                xml.start_svg_element(ElementId::RadialGradient);
                xml.write_id_attribute(&rg.id, opt);
                xml.write_svg_attribute(AttributeId::Cx, &rg.cx);
                xml.write_svg_attribute(AttributeId::Cy, &rg.cy);
                xml.write_svg_attribute(AttributeId::R, &rg.r.get());
                xml.write_svg_attribute(AttributeId::Fx, &rg.fx);
                xml.write_svg_attribute(AttributeId::Fy, &rg.fy);
                write_base_grad(&rg.base, xml);
                xml.end_element();
            }
            Paint::Pattern(pattern) => {
                xml.start_svg_element(ElementId::Pattern);
                xml.write_id_attribute(&pattern.id, opt);
                xml.write_rect_attrs(pattern.rect);
                xml.write_units(
                    AttributeId::PatternUnits,
                    pattern.units,
                    Units::ObjectBoundingBox,
                );
                xml.write_units(
                    AttributeId::PatternContentUnits,
                    pattern.content_units,
                    Units::UserSpaceOnUse,
                );
                xml.write_transform(AttributeId::PatternTransform, pattern.transform);

                if let Some(ref vbox) = pattern.view_box {
                    xml.write_viewbox(vbox);
                }

                conv_elements(&pattern.root, false, opt, xml);

                xml.end_element();
            }
        }
    }

    conv_filters(tree, opt, xml);

    let mut clip_paths = Vec::new();
    collect_clip_paths(tree.root.clone(), &mut clip_paths);
    for clip in clip_paths {
        xml.start_svg_element(ElementId::ClipPath);
        xml.write_id_attribute(&clip.id, opt);
        xml.write_units(
            AttributeId::ClipPathUnits,
            clip.units,
            Units::UserSpaceOnUse,
        );
        xml.write_transform(AttributeId::Transform, clip.transform);

        if let Some(ref clip) = clip.clip_path {
            xml.write_func_iri(AttributeId::ClipPath, &clip.id, opt);
        }

        conv_elements(&clip.root, true, opt, xml);

        xml.end_element();
    }

    let mut masks = Vec::new();
    collect_masks(tree.root.clone(), &mut masks);
    for mask in masks {
        xml.start_svg_element(ElementId::Mask);
        xml.write_id_attribute(&mask.id, opt);
        xml.write_units(AttributeId::MaskUnits, mask.units, Units::ObjectBoundingBox);
        xml.write_units(
            AttributeId::MaskContentUnits,
            mask.content_units,
            Units::UserSpaceOnUse,
        );
        xml.write_rect_attrs(mask.rect);

        if let Some(ref mask) = mask.mask {
            xml.write_func_iri(AttributeId::Mask, &mask.id, opt);
        }

        conv_elements(&mask.root, false, opt, xml);

        xml.end_element();
    }
}

fn conv_elements(parent: &Node, is_clip_path: bool, opt: &ExportOptions, xml: &mut XmlWriter) {
    for n in parent.children() {
        conv_element(&n, is_clip_path, opt, xml);
    }
}

fn conv_element(node: &Node, is_clip_path: bool, opt: &ExportOptions, xml: &mut XmlWriter) {
    match *node.borrow() {
        NodeKind::Text(_) => {
            // TODO
        }
        NodeKind::Path(ref p) => {
            write_path(p, is_clip_path, None, opt, xml);
        }
        NodeKind::Image(ref img) => {
            xml.start_svg_element(ElementId::Image);
            if !img.id.is_empty() {
                xml.write_id_attribute(&img.id, opt);
            }

            xml.write_rect_attrs(img.view_box.rect);
            if !img.view_box.aspect.is_default() {
                xml.write_aspect(img.view_box.aspect);
            }

            xml.write_visibility(img.visibility);

            match img.rendering_mode {
                ImageRendering::OptimizeQuality => {}
                ImageRendering::OptimizeSpeed => {
                    xml.write_svg_attribute(AttributeId::ImageRendering, "optimizeSpeed");
                }
            }

            xml.write_transform(AttributeId::Transform, img.transform);
            xml.write_image_data(&img.kind);

            xml.end_element();
        }
        NodeKind::Group(ref g) => {
            if is_clip_path {
                // ClipPath with a Group element is an `usvg` special case.
                // Group will contain a single Path element and we should set
                // `clip-path` on it.

                if let NodeKind::Path(ref path) = *node.first_child().unwrap().borrow() {
                    let clip_id = g.clip_path.as_ref().map(|cp| cp.id.as_str());
                    write_path(path, is_clip_path, clip_id, opt, xml);
                }

                return;
            }

            xml.start_svg_element(ElementId::G);
            if !g.id.is_empty() {
                xml.write_id_attribute(&g.id, opt);
            };

            if let Some(ref clip) = g.clip_path {
                xml.write_func_iri(AttributeId::ClipPath, &clip.id, opt);
            }

            if let Some(ref mask) = g.mask {
                xml.write_func_iri(AttributeId::Mask, &mask.id, opt);
            }

            if !g.filters.is_empty() {
                let prefix = opt.id_prefix.as_deref().unwrap_or_default();
                let ids: Vec<_> = g
                    .filters
                    .iter()
                    .map(|filter| format!("url(#{}{})", prefix, filter.id))
                    .collect();
                xml.write_svg_attribute(AttributeId::Filter, &ids.join(" "));

                if let Some(ref fill) = g.filter_fill {
                    write_paint(AttributeId::Fill, fill, opt, xml);
                }

                if let Some(ref stroke) = g.filter_stroke {
                    write_paint(AttributeId::Stroke, stroke, opt, xml);
                }
            }

            if g.opacity != Opacity::ONE {
                xml.write_svg_attribute(AttributeId::Opacity, &g.opacity.get());
            }

            xml.write_transform(AttributeId::Transform, g.transform);

            if let Some(eb) = g.enable_background {
                xml.write_enable_background(eb);
            }

            if g.blend_mode != BlendMode::Normal || g.isolate {
                let blend_mode = match g.blend_mode {
                    BlendMode::Normal => "normal",
                    BlendMode::Multiply => "multiply",
                    BlendMode::Screen => "screen",
                    BlendMode::Overlay => "overlay",
                    BlendMode::Darken => "darken",
                    BlendMode::Lighten => "lighten",
                    BlendMode::ColorDodge => "color-dodge",
                    BlendMode::ColorBurn => "color-burn",
                    BlendMode::HardLight => "hard-light",
                    BlendMode::SoftLight => "soft-light",
                    BlendMode::Difference => "difference",
                    BlendMode::Exclusion => "exclusion",
                    BlendMode::Hue => "hue",
                    BlendMode::Saturation => "saturation",
                    BlendMode::Color => "color",
                    BlendMode::Luminosity => "luminosity",
                };

                // For reasons unknown, `mix-blend-mode` and `isolation` must be written
                // as `style` attribute.
                let isolation = if g.isolate { "isolate" } else { "auto" };
                xml.write_attribute_fmt(
                    AttributeId::Style.to_str(),
                    format_args!("mix-blend-mode:{blend_mode};isolation:{isolation}"),
                );
            }

            conv_elements(node, false, opt, xml);

            xml.end_element();
        }
    }
}

trait XmlWriterExt {
    fn start_svg_element(&mut self, id: ElementId);
    fn write_svg_attribute<V: Display + ?Sized>(&mut self, id: AttributeId, value: &V);
    fn write_id_attribute(&mut self, value: &str, opt: &ExportOptions);
    fn write_color(&mut self, id: AttributeId, color: Color);
    fn write_viewbox(&mut self, view_box: &ViewBox);
    fn write_aspect(&mut self, aspect: AspectRatio);
    fn write_units(&mut self, id: AttributeId, units: Units, def: Units);
    fn write_transform(&mut self, id: AttributeId, units: Transform);
    fn write_enable_background(&mut self, eb: EnableBackground);
    fn write_visibility(&mut self, value: Visibility);
    fn write_func_iri(&mut self, aid: AttributeId, id: &str, opt: &ExportOptions);
    fn write_rect_attrs(&mut self, r: Rect);
    fn write_numbers(&mut self, aid: AttributeId, list: &[f64]);
    fn write_point<T: Display>(&mut self, id: AttributeId, p: Point<T>);
    fn write_image_data(&mut self, kind: &ImageKind);

    fn write_filter_input(&mut self, id: AttributeId, input: &filter::Input);
    fn write_filter_primitive_attrs(&mut self, fe: &filter::Primitive);
    fn write_filter_transfer_function(&mut self, eid: ElementId, fe: &filter::TransferFunction);
}

impl XmlWriterExt for XmlWriter {
    #[inline(never)]
    fn start_svg_element(&mut self, id: ElementId) {
        self.start_element(id.to_str());
    }

    #[inline(never)]
    fn write_svg_attribute<V: Display + ?Sized>(&mut self, id: AttributeId, value: &V) {
        self.write_attribute(id.to_str(), value)
    }

    #[inline(never)]
    fn write_id_attribute(&mut self, value: &str, opt: &ExportOptions) {
        debug_assert!(!value.is_empty());
        if let Some(ref prefix) = opt.id_prefix {
            self.write_attribute_fmt("id", format_args!("{prefix}{value}"));
        } else {
            self.write_attribute("id", value);
        }
    }

    #[inline(never)]
    fn write_color(&mut self, id: AttributeId, c: Color) {
        static CHARS: &[u8] = b"0123456789abcdef";

        #[inline]
        fn int2hex(n: u8) -> (u8, u8) {
            (CHARS[(n >> 4) as usize], CHARS[(n & 0xf) as usize])
        }

        let (r1, r2) = int2hex(c.red);
        let (g1, g2) = int2hex(c.green);
        let (b1, b2) = int2hex(c.blue);

        self.write_attribute_raw(id.to_str(), |buf| {
            buf.extend_from_slice(&[b'#', r1, r2, g1, g2, b1, b2])
        });
    }

    fn write_viewbox(&mut self, view_box: &ViewBox) {
        let r = view_box.rect;
        self.write_attribute_fmt(
            AttributeId::ViewBox.to_str(),
            format_args!("{} {} {} {}", r.x(), r.y(), r.width(), r.height()),
        );

        if !view_box.aspect.is_default() {
            self.write_aspect(view_box.aspect);
        }
    }

    fn write_aspect(&mut self, aspect: AspectRatio) {
        let mut value = Vec::new();

        if aspect.defer {
            value.extend_from_slice(b"defer ");
        }

        let align = match aspect.align {
            Align::None => "none",
            Align::XMinYMin => "xMinYMin",
            Align::XMidYMin => "xMidYMin",
            Align::XMaxYMin => "xMaxYMin",
            Align::XMinYMid => "xMinYMid",
            Align::XMidYMid => "xMidYMid",
            Align::XMaxYMid => "xMaxYMid",
            Align::XMinYMax => "xMinYMax",
            Align::XMidYMax => "xMidYMax",
            Align::XMaxYMax => "xMaxYMax",
        };

        value.extend_from_slice(align.as_bytes());

        if aspect.slice {
            value.extend_from_slice(b" slice");
        }

        self.write_attribute_raw(AttributeId::PreserveAspectRatio.to_str(), |buf| {
            buf.extend_from_slice(&value)
        });
    }

    fn write_units(&mut self, id: AttributeId, units: Units, def: Units) {
        if units != def {
            self.write_attribute(
                id.to_str(),
                match units {
                    Units::UserSpaceOnUse => "userSpaceOnUse",
                    Units::ObjectBoundingBox => "objectBoundingBox",
                },
            );
        }
    }

    fn write_transform(&mut self, id: AttributeId, ts: Transform) {
        if !ts.is_default() {
            self.write_attribute_raw(id.to_str(), |buf| {
                buf.extend_from_slice(b"matrix(");
                write_num(ts.a, buf);
                buf.push(b' ');
                write_num(ts.b, buf);
                buf.push(b' ');
                write_num(ts.c, buf);
                buf.push(b' ');
                write_num(ts.d, buf);
                buf.push(b' ');
                write_num(ts.e, buf);
                buf.push(b' ');
                write_num(ts.f, buf);
                buf.extend_from_slice(b")");
            });
        }
    }

    fn write_enable_background(&mut self, eb: EnableBackground) {
        let id = AttributeId::EnableBackground.to_str();
        match eb {
            EnableBackground(None) => {
                self.write_attribute(id, "new");
            }
            EnableBackground(Some(r)) => {
                self.write_attribute_fmt(
                    id,
                    format_args!("new {} {} {} {}", r.x(), r.y(), r.width(), r.height()),
                );
            }
        }
    }

    fn write_visibility(&mut self, value: Visibility) {
        match value {
            Visibility::Visible => {}
            Visibility::Hidden => self.write_attribute(AttributeId::Visibility.to_str(), "hidden"),
            Visibility::Collapse => {
                self.write_attribute(AttributeId::Visibility.to_str(), "collapse")
            }
        }
    }

    fn write_func_iri(&mut self, aid: AttributeId, id: &str, opt: &ExportOptions) {
        let prefix = opt.id_prefix.as_deref().unwrap_or_default();
        self.write_attribute_fmt(aid.to_str(), format_args!("url(#{prefix}{id})"));
    }

    fn write_rect_attrs(&mut self, r: Rect) {
        self.write_svg_attribute(AttributeId::X, &r.x());
        self.write_svg_attribute(AttributeId::Y, &r.y());
        self.write_svg_attribute(AttributeId::Width, &r.width());
        self.write_svg_attribute(AttributeId::Height, &r.height());
    }

    fn write_numbers(&mut self, aid: AttributeId, list: &[f64]) {
        self.write_attribute_raw(aid.to_str(), |buf| {
            for n in list {
                buf.write_fmt(format_args!("{n} ")).unwrap();
            }

            if !list.is_empty() {
                buf.pop();
            }
        });
    }

    fn write_point<T: Display>(&mut self, id: AttributeId, p: Point<T>) {
        self.write_attribute_fmt(id.to_str(), format_args!("{} {}", p.x, p.y));
    }

    fn write_filter_input(&mut self, id: AttributeId, input: &filter::Input) {
        self.write_attribute(
            id.to_str(),
            match input {
                filter::Input::SourceGraphic => "SourceGraphic",
                filter::Input::SourceAlpha => "SourceAlpha",
                filter::Input::BackgroundImage => "BackgroundImage",
                filter::Input::BackgroundAlpha => "BackgroundAlpha",
                filter::Input::FillPaint => "FillPaint",
                filter::Input::StrokePaint => "StrokePaint",
                filter::Input::Reference(ref s) => s,
            },
        );
    }

    fn write_filter_primitive_attrs(&mut self, fe: &filter::Primitive) {
        if let Some(n) = fe.x {
            self.write_svg_attribute(AttributeId::X, &n);
        }
        if let Some(n) = fe.y {
            self.write_svg_attribute(AttributeId::Y, &n);
        }
        if let Some(n) = fe.width {
            self.write_svg_attribute(AttributeId::Width, &n);
        }
        if let Some(n) = fe.height {
            self.write_svg_attribute(AttributeId::Height, &n);
        }

        self.write_attribute(
            AttributeId::ColorInterpolationFilters.to_str(),
            match fe.color_interpolation {
                filter::ColorInterpolation::SRGB => "sRGB",
                filter::ColorInterpolation::LinearRGB => "linearRGB",
            },
        );
    }

    fn write_filter_transfer_function(
        &mut self,
        element_id: ElementId,
        fe: &filter::TransferFunction,
    ) {
        self.start_svg_element(element_id);

        match fe {
            filter::TransferFunction::Identity => {
                self.write_svg_attribute(AttributeId::Type, "identity");
            }
            filter::TransferFunction::Table(ref values) => {
                self.write_svg_attribute(AttributeId::Type, "table");
                self.write_numbers(AttributeId::TableValues, values);
            }
            filter::TransferFunction::Discrete(ref values) => {
                self.write_svg_attribute(AttributeId::Type, "discrete");
                self.write_numbers(AttributeId::TableValues, values);
            }
            filter::TransferFunction::Linear { slope, intercept } => {
                self.write_svg_attribute(AttributeId::Type, "linear");
                self.write_svg_attribute(AttributeId::Slope, &slope);
                self.write_svg_attribute(AttributeId::Intercept, &intercept);
            }
            filter::TransferFunction::Gamma {
                amplitude,
                exponent,
                offset,
            } => {
                self.write_svg_attribute(AttributeId::Type, "gamma");
                self.write_svg_attribute(AttributeId::Amplitude, &amplitude);
                self.write_svg_attribute(AttributeId::Exponent, &exponent);
                self.write_svg_attribute(AttributeId::Offset, &offset);
            }
        }

        self.end_element();
    }

    fn write_image_data(&mut self, kind: &usvg::ImageKind) {
        let svg_string;
        let (mime, data) = match kind {
            usvg::ImageKind::JPEG(ref data) => ("jpeg", data.as_slice()),
            usvg::ImageKind::PNG(ref data) => ("png", data.as_slice()),
            usvg::ImageKind::GIF(ref data) => ("gif", data.as_slice()),
            usvg::ImageKind::SVG(ref tree) => {
                svg_string = tree.to_string(&ExportOptions::default());
                ("svg+xml", svg_string.as_bytes())
            }
        };

        self.write_attribute_raw("xlink:href", |buf| {
            buf.extend_from_slice(b"data:image/");
            buf.extend_from_slice(mime.as_bytes());
            buf.extend_from_slice(b";base64, ");

            let mut enc = base64::write::EncoderWriter::from(buf, &base64::engine::DEFAULT_ENGINE);
            enc.write_all(data).unwrap();
            enc.finish().unwrap();
        });
    }
}

fn has_xlink(tree: &Tree) -> bool {
    for n in tree.root.descendants() {
        match *n.borrow() {
            NodeKind::Group(ref g) => {
                for filter in &g.filters {
                    if filter
                        .primitives
                        .iter()
                        .any(|p| matches!(p.kind, filter::Kind::Image(_)))
                    {
                        return true;
                    }
                }
            }
            NodeKind::Image(_) => {
                return true;
            }
            _ => {}
        }
    }

    false
}

fn write_base_grad(g: &BaseGradient, xml: &mut XmlWriter) {
    xml.write_units(
        AttributeId::GradientUnits,
        g.units,
        Units::ObjectBoundingBox,
    );
    xml.write_transform(AttributeId::GradientTransform, g.transform);

    match g.spread_method {
        SpreadMethod::Pad => {}
        SpreadMethod::Reflect => xml.write_svg_attribute(AttributeId::SpreadMethod, "reflect"),
        SpreadMethod::Repeat => xml.write_svg_attribute(AttributeId::SpreadMethod, "repeat"),
    }

    for s in &g.stops {
        xml.start_svg_element(ElementId::Stop);
        xml.write_svg_attribute(AttributeId::Offset, &s.offset.get());
        xml.write_color(AttributeId::StopColor, s.color);
        if s.opacity != Opacity::ONE {
            xml.write_svg_attribute(AttributeId::StopOpacity, &s.opacity.get());
        }

        xml.end_element();
    }
}

fn write_path(
    path: &Path,
    is_clip_path: bool,
    clip_path: Option<&str>,
    opt: &ExportOptions,
    xml: &mut XmlWriter,
) {
    xml.start_svg_element(ElementId::Path);
    if !path.id.is_empty() {
        xml.write_id_attribute(&path.id, opt);
    }

    write_fill(&path.fill, is_clip_path, opt, xml);
    write_stroke(&path.stroke, opt, xml);

    xml.write_visibility(path.visibility);

    if path.paint_order == PaintOrder::StrokeAndFill {
        xml.write_svg_attribute(AttributeId::PaintOrder, "stroke");
    }

    match path.rendering_mode {
        ShapeRendering::OptimizeSpeed => {
            xml.write_svg_attribute(AttributeId::ShapeRendering, "optimizeSpeed");
        }
        ShapeRendering::CrispEdges => {
            xml.write_svg_attribute(AttributeId::ShapeRendering, "crispEdges")
        }
        ShapeRendering::GeometricPrecision => {}
    }

    if let Some(id) = clip_path {
        xml.write_func_iri(AttributeId::ClipPath, id, opt);
    }

    xml.write_transform(AttributeId::Transform, path.transform);

    xml.write_attribute_raw("d", |buf| {
        for seg in path.data.segments() {
            match seg {
                PathSegment::MoveTo { x, y } => {
                    buf.extend_from_slice(b"M ");
                    write_num(x, buf);
                    buf.push(b' ');
                    write_num(y, buf);
                    buf.push(b' ');
                }
                PathSegment::LineTo { x, y } => {
                    buf.extend_from_slice(b"L ");
                    write_num(x, buf);
                    buf.push(b' ');
                    write_num(y, buf);
                    buf.push(b' ');
                }
                PathSegment::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                } => {
                    buf.extend_from_slice(b"C ");
                    write_num(x1, buf);
                    buf.push(b' ');
                    write_num(y1, buf);
                    buf.push(b' ');
                    write_num(x2, buf);
                    buf.push(b' ');
                    write_num(y2, buf);
                    buf.push(b' ');
                    write_num(x, buf);
                    buf.push(b' ');
                    write_num(y, buf);
                    buf.push(b' ');
                }
                PathSegment::ClosePath => {
                    buf.extend_from_slice(b"Z ");
                }
            }
        }

        if !path.data.is_empty() {
            buf.pop();
        }
    });

    xml.end_element();
}

fn write_fill(fill: &Option<Fill>, is_clip_path: bool, opt: &ExportOptions, xml: &mut XmlWriter) {
    if let Some(ref fill) = fill {
        write_paint(AttributeId::Fill, &fill.paint, opt, xml);

        if fill.opacity != Opacity::ONE {
            xml.write_svg_attribute(AttributeId::FillOpacity, &fill.opacity.get());
        }

        if !fill.rule.is_default() {
            let name = if is_clip_path {
                AttributeId::ClipRule
            } else {
                AttributeId::FillRule
            };

            xml.write_svg_attribute(name, "evenodd");
        }
    } else {
        xml.write_svg_attribute(AttributeId::Fill, "none");
    }
}

fn write_stroke(stroke: &Option<Stroke>, opt: &ExportOptions, xml: &mut XmlWriter) {
    if let Some(ref stroke) = stroke {
        write_paint(AttributeId::Stroke, &stroke.paint, opt, xml);

        if stroke.opacity != Opacity::ONE {
            xml.write_svg_attribute(AttributeId::StrokeOpacity, &stroke.opacity.get());
        }

        if !(stroke.dashoffset as f64).is_fuzzy_zero() {
            xml.write_svg_attribute(AttributeId::StrokeDashoffset, &stroke.dashoffset)
        }

        if !stroke.miterlimit.is_default() {
            xml.write_svg_attribute(AttributeId::StrokeMiterlimit, &stroke.miterlimit.get());
        }

        if stroke.width.get() != 1.0 {
            xml.write_svg_attribute(AttributeId::StrokeWidth, &stroke.width.get());
        }

        match stroke.linecap {
            LineCap::Butt => {}
            LineCap::Round => xml.write_svg_attribute(AttributeId::StrokeLinecap, "round"),
            LineCap::Square => xml.write_svg_attribute(AttributeId::StrokeLinecap, "square"),
        }

        match stroke.linejoin {
            LineJoin::Miter => {}
            LineJoin::Round => xml.write_svg_attribute(AttributeId::StrokeLinejoin, "round"),
            LineJoin::Bevel => xml.write_svg_attribute(AttributeId::StrokeLinejoin, "bevel"),
        }

        if let Some(ref array) = stroke.dasharray {
            xml.write_numbers(AttributeId::StrokeDasharray, array);
        }
    } else {
        // Always set `stroke` to `none` to override the parent value.
        // In 99.9% of the cases it's redundant, but a group with `filter` with `StrokePaint`
        // will set `stroke`, which will interfere with children nodes.
        xml.write_svg_attribute(AttributeId::Stroke, "none");
    }
}

fn write_paint(aid: AttributeId, paint: &Paint, opt: &ExportOptions, xml: &mut XmlWriter) {
    match paint {
        Paint::Color(c) => xml.write_color(aid, *c),
        Paint::LinearGradient(ref lg) => xml.write_func_iri(aid, &lg.id, opt),
        Paint::RadialGradient(ref rg) => xml.write_func_iri(aid, &rg.id, opt),
        Paint::Pattern(ref patt) => xml.write_func_iri(aid, &patt.id, opt),
    }
}

fn write_light_source(light: &filter::LightSource, xml: &mut XmlWriter) {
    match light {
        filter::LightSource::DistantLight(ref light) => {
            xml.start_svg_element(ElementId::FeDistantLight);
            xml.write_svg_attribute(AttributeId::Azimuth, &light.azimuth);
            xml.write_svg_attribute(AttributeId::Elevation, &light.elevation);
        }
        filter::LightSource::PointLight(ref light) => {
            xml.start_svg_element(ElementId::FePointLight);
            xml.write_svg_attribute(AttributeId::X, &light.x);
            xml.write_svg_attribute(AttributeId::Y, &light.y);
            xml.write_svg_attribute(AttributeId::Z, &light.z);
        }
        filter::LightSource::SpotLight(ref light) => {
            xml.start_svg_element(ElementId::FeSpotLight);
            xml.write_svg_attribute(AttributeId::X, &light.x);
            xml.write_svg_attribute(AttributeId::Y, &light.y);
            xml.write_svg_attribute(AttributeId::Z, &light.z);
            xml.write_svg_attribute(AttributeId::PointsAtX, &light.points_at_x);
            xml.write_svg_attribute(AttributeId::PointsAtY, &light.points_at_y);
            xml.write_svg_attribute(AttributeId::PointsAtZ, &light.points_at_z);
            xml.write_svg_attribute(AttributeId::SpecularExponent, &light.specular_exponent);
            if let Some(ref n) = light.limiting_cone_angle {
                xml.write_svg_attribute(AttributeId::LimitingConeAngle, n);
            }
        }
    }

    xml.end_element();
}

fn write_num(num: f64, buf: &mut Vec<u8>) {
    // If number is an integer, it's faster to write it as i32.
    if num.fract().is_fuzzy_zero() {
        write!(buf, "{}", num as i32).unwrap();
        return;
    }

    // Round numbers up to 8 digits to prevent writing
    // ugly numbers like 29.999999999999996.
    // It's not 100% correct, but differences are insignificant.
    //
    // Note that at least in Rust 1.64 the number formatting in debug and release modes
    // can be slightly different. So having a lower precision makes
    // our output and tests reproducible.
    let v = (num * 100_000_000.0).round() / 100_000_000.0;

    write!(buf, "{v}").unwrap();
}
