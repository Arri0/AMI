use super::info;
use crate::render::Renderer;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, Device, FromSample, Host, SampleRate, SizedSample, Stream, StreamConfig,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

#[derive(Debug)]
pub enum Error {
    HostNotFound,
    DeviceNotFound,
    NoDefaultDevice,
    UnsupportedSampleFormat(cpal::SampleFormat),
    UnsupportedBufferSize,
    NoDefaultConfig,
    BuildStream(cpal::BuildStreamError),
    PlayStream(cpal::PlayStreamError),
}

pub struct Controller {
    renderer: Arc<Mutex<Renderer>>,
    stream: Option<Stream>,
    pub sample_rate: u32,
    pub buffer_size: usize,
    num_channels: usize,
}

impl Controller {
    pub fn new(renderer: Arc<Mutex<Renderer>>) -> Self {
        Self {
            renderer,
            stream: Default::default(),
            sample_rate: 44100,
            buffer_size: 128,
            num_channels: 0,
        }
    }

    pub fn connect_to_output_device(
        &mut self,
        host_name: &str,
        device_name: &str,
    ) -> Result<(), Error> {
        let (stream, num_channels) = init_output_device(
            host_name,
            device_name,
            self.sample_rate,
            self.buffer_size as u32,
            Arc::clone(&self.renderer),
        )?;
        self.stream = Some(stream);
        self.num_channels = num_channels;
        futures::executor::block_on(async {
            self.renderer.lock().await.set_sample_rate(self.sample_rate);
        });
        Ok(())
    }

    pub fn connect_to_default_output_device(&mut self) -> Result<(), Error> {
        let host = info::get_default_host();
        let device_name =
            info::get_default_output_device_name(&host).ok_or(Error::NoDefaultDevice)?;
        let host_name = info::get_default_host_name();
        self.connect_to_output_device(&host_name, &device_name)
    }
}

fn find_host(host_name: &str) -> Option<Host> {
    let host_id = cpal::available_hosts()
        .into_iter()
        .find(|host| host.name() == host_name)?;
    cpal::host_from_id(host_id).ok()
}

fn find_output_device(host: Host, device_name: &str) -> Option<Device> {
    host.output_devices().ok()?.find(|device| {
        if let Ok(name) = device.name() {
            name == device_name
        } else {
            false
        }
    })
}

fn init_output_device(
    host_name: &str,
    device_name: &str,
    sample_rate: u32,
    buffer_size: u32,
    renderer: Arc<Mutex<Renderer>>,
) -> Result<(Stream, usize), Error> {
    let host = find_host(host_name).ok_or(Error::HostNotFound)?;
    let device = find_output_device(host, device_name).ok_or(Error::DeviceNotFound)?;
    let config = device
        .default_output_config()
        .map_err(|_| Error::NoDefaultConfig)?;
    let sample_format = config.sample_format();
    let mut cfg: StreamConfig = config.into();
    cfg.buffer_size = BufferSize::Fixed(buffer_size);
    cfg.sample_rate = SampleRate(sample_rate);
    cfg.channels = 2;
    let stream = create_stream_dispatched(sample_format, device, &cfg, renderer)?;
    Ok((stream, cfg.channels as usize))
}

fn create_stream_dispatched(
    sample_format: cpal::SampleFormat,
    device: Device,
    cfg: &StreamConfig,
    renderer: Arc<Mutex<Renderer>>,
) -> Result<Stream, Error> {
    match sample_format {
        cpal::SampleFormat::I8 => create_stream::<i8>(&device, cfg, renderer),
        cpal::SampleFormat::I16 => create_stream::<i16>(&device, cfg, renderer),
        cpal::SampleFormat::I32 => create_stream::<i32>(&device, cfg, renderer),
        cpal::SampleFormat::I64 => create_stream::<i64>(&device, cfg, renderer),
        cpal::SampleFormat::U8 => create_stream::<u8>(&device, cfg, renderer),
        cpal::SampleFormat::U16 => create_stream::<u16>(&device, cfg, renderer),
        cpal::SampleFormat::U32 => create_stream::<u32>(&device, cfg, renderer),
        cpal::SampleFormat::U64 => create_stream::<u64>(&device, cfg, renderer),
        cpal::SampleFormat::F32 => create_stream::<f32>(&device, cfg, renderer),
        cpal::SampleFormat::F64 => create_stream::<f64>(&device, cfg, renderer),
        f => Err(Error::UnsupportedSampleFormat(f)),
    }
}

fn create_stream<T>(
    device: &Device,
    config: &StreamConfig,
    renderer: Arc<Mutex<Renderer>>,
) -> Result<Stream, Error>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    // let mut next_value = move || 0.0;
    let err_fn = |err| error!("An error occurred on stream: {}", err); //TODO: handle this case
    let mut lbuf = vec![];
    let mut rbuf = vec![];

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let curr_buf_size = data.len() / channels;
                if lbuf.len() < curr_buf_size {
                    lbuf.resize(curr_buf_size, 0.0);
                    rbuf.resize(curr_buf_size, 0.0);
                }
                let lbuf_slice = &mut lbuf[..curr_buf_size];
                let rbuf_slice = &mut rbuf[..curr_buf_size];

                futures::executor::block_on(async {
                    let mut renderer = renderer.lock().await;
                    renderer.render(lbuf_slice, rbuf_slice);
                });
                for (n, frame) in data.chunks_mut(channels).enumerate() {
                    let values = [T::from_sample(lbuf_slice[n]), T::from_sample(rbuf_slice[n])];
                    for (k, sample) in frame.iter_mut().enumerate() {
                        *sample = values[k & 1];
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(Error::BuildStream)?;
    stream.play().map_err(Error::PlayStream)?;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use crate::{audio::info, control, path::VirtualPaths, render};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn controller() {
        let (_midi_tx, midi_rx) = crate::midi::create_channel(1);
        let (_req_tx, req_rx) = render::command::create_request_channel(1);
        let (_dm_ctr_tx, dm_ctr_rx) = control::create_control_channel(1);
        let renderer = Arc::new(Mutex::new(super::Renderer::new(
            midi_rx,
            req_rx,
            dm_ctr_rx,
            VirtualPaths::default(),
        )));
        let mut audio_ctr = super::Controller::new(renderer);
        let host = info::get_default_host();
        let device_name = info::get_default_output_device_name(&host).expect(concat!(
            "The host doesn't provide any default output device. ",
            "(Not an error in the code, it's just not available on your device.)"
        ));
        let host_name = info::get_default_host_name();
        audio_ctr
            .connect_to_output_device(&host_name, &device_name)
            .unwrap();
        assert!(audio_ctr.stream.is_some());
    }
}
