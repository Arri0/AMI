use node::RenderPtr;

pub mod midi_filter;
pub mod node;
pub mod preset_map;
pub mod velocity_map;
pub mod renderer;

pub const MAX_BUFFER_SIZE: usize = 192000;

pub fn amplify_buffer(buffer: &mut [f32], gain: f32) {
    if gain != 1.0 {
        buffer.iter_mut().for_each(|x| *x *= gain);
    }
}

pub fn clear_buffer(buffer: &mut [f32]) {
    buffer.fill(0.0);
}

pub fn render_nodes_to_bufs(nodes: &mut [RenderPtr], lbuf: &mut [f32], rbuf: &mut [f32]) {
    nodes
        .iter_mut()
        .for_each(|child| child.render_additive(lbuf, rbuf));
}

pub fn add_buf_to_buf(buffer: &mut [f32], tmp_buffer: &[f32]) {
    let len = usize::min(buffer.len(), tmp_buffer.len());
    for i in 0..len {
        buffer[i] += tmp_buffer[i];
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn amplify_buffer() {
        let gain = 3.2;
        let mut buffer = [1.0, 0.0, 3.2];
        super::amplify_buffer(&mut buffer, gain);
        assert_eq!(buffer, [1.0 * gain, 0.0 * gain, 3.2 * gain])
    }
}
