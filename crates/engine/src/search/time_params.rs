use crate::tuneable::{
    tm_best_move_min_scale, tm_best_move_offset, tm_best_move_scale, tm_eval_stability_min_scale,
    tm_eval_stability_offset, tm_eval_stability_scale, tm_node_stability_offset,
    tm_node_stability_scale,
};

#[derive(Debug, Clone, Copy)]
pub struct TimeParams {
    pub node_stability_offset: f32,
    pub node_stability_scale: f32,
    pub eval_stability_offset: f32,
    pub eval_stability_scale: f32,
    pub eval_stability_min: f32,
    pub best_move_offset: f32,
    pub best_move_scale: f32,
    pub best_move_min: f32,
}

impl TimeParams {
    pub fn new() -> TimeParams {
        TimeParams {
            node_stability_offset: tm_node_stability_offset() as f32 / 10000.0,
            node_stability_scale: tm_node_stability_scale() as f32 / 10000.0,
            eval_stability_offset: tm_eval_stability_offset() as f32 / 10000.0,
            eval_stability_scale: tm_eval_stability_scale() as f32 / 10000.0,
            eval_stability_min: tm_eval_stability_min_scale() as f32 / 10000.0,
            best_move_offset: tm_best_move_offset() as f32 / 10000.0,
            best_move_scale: tm_best_move_scale() as f32 / 10000.0,
            best_move_min: tm_best_move_min_scale() as f32 / 10000.0,
        }
    }
}

impl Default for TimeParams {
    fn default() -> Self {
        Self::new()
    }
}
