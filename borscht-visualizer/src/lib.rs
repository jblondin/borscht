/*!
 * Visualizer for BIRCH CFTrees.
 */

use std::ops::{Range, RangeBounds};

use palettes::{Palette, Triple};
use plotters::{
    coord::Shift,
    prelude::{BitMapBackend, DrawingArea, IntoDrawingArea, RGBColor, TextStyle, WHITE},
};
use plotters_backend::BackendTextStyle;
use thiserror::Error;

use borscht::{
    cfeature::{birch::CFeature as BirchFeature, CFeature},
    cftree::Node,
};
use plotters_bitmap::bitmap_pixel::RGBPixel;

pub mod palettes;

const IMG_WIDTH: u32 = 512;

const TITLE_TEXT: &str = "Sample Visualization";
const TITLE_FONT_FAMILY: &str = "sans-serif";
const TITLE_HEIGHT: u32 = 30;
const TITLE_STYLE: (&str, u32) = (TITLE_FONT_FAMILY, TITLE_HEIGHT);

const DRAW_AREA_LR_MARGIN: u32 = 12;
const DRAW_AREA_WIDTH: u32 = IMG_WIDTH - DRAW_AREA_LR_MARGIN * 2;
const DRAW_AREA_TB_MARGIN: u32 = 4;

const NODE_HEIGHT: u32 = 40;

#[derive(Error, Debug)]
pub enum VisualizerError {
    #[error("font layout error")]
    FontLayout(Box<dyn std::error::Error>),
    #[error("drawing error")]
    Drawing(Box<dyn std::error::Error>),
}

type TreeNode = Node<BirchFeature<3>, 3>;
type Result<T> = std::result::Result<T, VisualizerError>;
type DrawArea<'a> = DrawingArea<BitMapBackend<'a, RGBPixel>, Shift>;

fn estimate_title_height(text: &str, style: &TextStyle) -> Result<u32> {
    let layout = style
        .layout_box(text)
        .map_err(|e| VisualizerError::FontLayout(Box::new(e)))?;
    let layout_h = ((layout.1).1 - (layout.0).1) as u32;
    Ok(layout_h + (layout_h / 2).min(5) * 2)
}

fn split_into_subareas(area: DrawArea, mut xs: Vec<f64>) -> Vec<DrawArea> {
    if let Some(x) = xs.pop() {
        let (l, r) = area.split_horizontally(x as u32);

        let l_width = l.dim_in_pixel().0 as f64;
        let mut rest =
            split_into_subareas(r, xs.drain(..).map(|x| x - l_width).collect::<Vec<_>>());
        let mut vec_l = vec![l];
        vec_l.append(&mut rest);
        vec_l
    } else {
        vec![area]
    }
}

pub struct ColorIter<'a> {
    n: usize,
    palette: &'a Palette,
}
const COLOR_GAP: usize = 64;
impl<'a> ColorIter<'a> {
    fn new(palette: &'a Palette) -> ColorIter<'a> {
        ColorIter { n: 0, palette }
    }
    fn next(&mut self) -> Triple {
        let orig_n = self.n;
        self.n = (self.n + COLOR_GAP) % self.palette.len();
        self.palette[orig_n]
    }
}

trait TransposeRange {
    type Type;
    fn transpose_range(&self) -> Range<(Self::Type, Self::Type)>;
}
impl<T> TransposeRange for (Range<T>, Range<T>)
where
    T: Copy,
{
    type Type = T;
    fn transpose_range(&self) -> Range<(Self::Type, Self::Type)> {
        return (self.0.start_bound(), self.1.start_bound())
            ..(self.0.end_bound(), self.1.end_bound());
    }
}

pub fn draw_node_to_area(
    area: &DrawArea,
    node: &TreeNode,
    color_iter: &mut ColorIter,
) -> Result<()> {
    let height = node.height();
    let (sum, xs) = node
        .entries
        .iter()
        .fold((0.0, vec![]), |(sum, mut v), entry| {
            let new_sum = sum + entry.feature.size();
            v.push(new_sum);
            (new_sum, v)
        });
    let xs = xs
        .iter()
        .take(xs.len() - 1)
        .map(|x| x / sum * DRAW_AREA_WIDTH as f64)
        .rev()
        .collect::<Vec<_>>();
    println!("start area: {:?}", area.get_pixel_range());
    let hsplits = split_into_subareas(area.clone(), xs);
    println!(
        "hsplits: {:?}",
        hsplits
            .iter()
            .map(|hs| hs.get_pixel_range())
            .collect::<Vec<_>>()
    );
    for (i, hsplit) in hsplits.iter().enumerate() {
        let vsplits = hsplit.split_evenly((height, 1));
        println!(
            "vsplits: {:?}",
            vsplits
                .iter()
                .map(|vs| vs.get_pixel_range())
                .collect::<Vec<_>>()
        );
        for ((j, vsplit), entry) in vsplits.iter().enumerate().zip(node.entries.iter()) {
            println!("split ({}, {}): {:?}", i, j, vsplit.get_pixel_range());
            let (r, g, b) = color_iter.next();
            vsplit
                .fill(&RGBColor(r, g, b))
                .map_err(|e| VisualizerError::Drawing(Box::new(e)))?;
            if let Some(child) = entry.child.as_ref() {
                draw_node_to_area(vsplit, child, color_iter)?;
            }
        }
    }
    Ok(())
}

pub fn draw_to_file(filename: &str, tree: &TreeNode) -> Result<()> {
    let draw_area_height = NODE_HEIGHT * tree.height() as u32;
    let title_style: TextStyle = TITLE_STYLE.into();
    let estimated_title_height = estimate_title_height(TITLE_TEXT, &title_style)?;
    let img_height = draw_area_height + DRAW_AREA_TB_MARGIN * 2 + estimated_title_height;
    let root = BitMapBackend::new(filename, (IMG_WIDTH, img_height)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|e| VisualizerError::Drawing(Box::new(e)))?;
    let root = root
        .titled(TITLE_TEXT, TITLE_STYLE)
        .map_err(|e| VisualizerError::Drawing(Box::new(e)))?
        .shrink(
            (DRAW_AREA_LR_MARGIN, 0),
            (DRAW_AREA_WIDTH, draw_area_height),
        );
    let palette = &palettes::PALETTES[308];
    let mut palette_iter = ColorIter::new(palette);
    draw_node_to_area(&root, tree, &mut palette_iter)?;
    root.present()
        .map_err(|e| VisualizerError::Drawing(Box::new(e)))?;

    Ok(())
}
