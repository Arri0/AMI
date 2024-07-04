use super::info;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, Device, FromSample, Host, SampleFormat, SampleRate, SizedSample, Stream,
    StreamConfig,
};
use ringbuf::traits::{Consumer, Observer, Split};
use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};
use tracing::error;

pub type BufferTx = ringbuf::wrap::caching::Caching<
    Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>,
    true,
    false,
>;
pub type BufferRx = ringbuf::wrap::caching::Caching<
    Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>,
    false,
    true,
>;
pub type OutputResult = Result<ConnectedOutput, Error>;

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

pub struct ConnectedOutput {
    pub stream: Stream,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub num_channels: usize,
    pub required_num_samples: Arc<AtomicUsize>,
    pub lbuf_tx: BufferTx,
    pub rbuf_tx: BufferTx,
}

pub struct OutputDeviceParams<'a> {
    pub host_name: &'a str,
    pub device_name: &'a str,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub num_channels: usize,
}

pub struct DefaultOutputDeviceParams {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub num_channels: usize,
}

struct StreamParams {
    sample_format: cpal::SampleFormat,
    device: Device,
    cfg: StreamConfig,
    required_num_samples: Arc<AtomicUsize>,
    lbuf_rx: BufferRx,
    rbuf_rx: BufferRx,
}

pub fn connect_to_default_output_device(params: DefaultOutputDeviceParams) -> OutputResult {
    let host = info::get_default_host();
    let device_name = info::get_default_output_device_name(&host).ok_or(Error::NoDefaultDevice)?;
    let host_name = info::get_default_host_name();

    connect_to_output_device(OutputDeviceParams {
        host_name: &host_name,
        device_name: &device_name,
        sample_rate: params.sample_rate,
        buffer_size: params.buffer_size,
        num_channels: 2,
    })
}

pub fn connect_to_output_device(params: OutputDeviceParams) -> OutputResult {
    let host = find_host(params.host_name).ok_or(Error::HostNotFound)?;
    let device = find_output_device(host, params.device_name).ok_or(Error::DeviceNotFound)?;
    let sample_format = sample_format(&device)?;
    let cfg = create_stream_config(&params);
    let ((lbuf_tx, lbuf_rx), (rbuf_tx, rbuf_rx)) = create_buffers(params.buffer_size);
    let required_num_samples = Arc::new(AtomicUsize::new(0));
    let stream = create_stream_dispatched(StreamParams {
        sample_format,
        device,
        cfg,
        required_num_samples: Arc::clone(&required_num_samples),
        lbuf_rx,
        rbuf_rx,
    })?;
    Ok(ConnectedOutput {
        stream,
        sample_rate: params.sample_rate,
        buffer_size: params.buffer_size,
        num_channels: params.num_channels,
        required_num_samples,
        lbuf_tx,
        rbuf_tx,
    })
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

fn sample_format(device: &Device) -> Result<SampleFormat, Error> {
    let config = device
        .default_output_config()
        .map_err(|_| Error::NoDefaultConfig)?;
    Ok(config.sample_format())
}

fn create_stream_config(params: &OutputDeviceParams) -> StreamConfig {
    StreamConfig {
        channels: params.num_channels as u16,
        sample_rate: SampleRate(params.sample_rate),
        buffer_size: BufferSize::Fixed(params.buffer_size as u32),
    }
}

fn create_stream_dispatched(params: StreamParams) -> Result<Stream, Error> {
    match params.sample_format {
        cpal::SampleFormat::I8 => create_stream::<i8>(params),
        cpal::SampleFormat::I16 => create_stream::<i16>(params),
        cpal::SampleFormat::I32 => create_stream::<i32>(params),
        cpal::SampleFormat::I64 => create_stream::<i64>(params),
        cpal::SampleFormat::U8 => create_stream::<u8>(params),
        cpal::SampleFormat::U16 => create_stream::<u16>(params),
        cpal::SampleFormat::U32 => create_stream::<u32>(params),
        cpal::SampleFormat::U64 => create_stream::<u64>(params),
        cpal::SampleFormat::F32 => create_stream::<f32>(params),
        cpal::SampleFormat::F64 => create_stream::<f64>(params),
        f => Err(Error::UnsupportedSampleFormat(f)),
    }
}

fn create_stream<T>(mut params: StreamParams) -> Result<Stream, Error>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = params.cfg.channels as usize;
    // let mut next_value = move || 0.0;
    let err_fn = |err| error!("An error occurred on stream: {}", err); //TODO: handle this case

    let stream = params
        .device
        .build_output_stream(
            &params.cfg,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let curr_buf_size = data.len() / channels;

                params.lbuf_rx.clear();
                params.rbuf_rx.clear();

                params
                    .required_num_samples
                    .store(curr_buf_size, std::sync::atomic::Ordering::Relaxed);

                while params.lbuf_rx.occupied_len() < curr_buf_size
                    || params.rbuf_rx.occupied_len() < curr_buf_size
                {
                    std::thread::sleep(Duration::from_micros(10));
                }

                for frame in data.chunks_mut(channels) {
                    let lval = params.lbuf_rx.try_pop().expect("Sample expected");
                    let rval = params.rbuf_rx.try_pop().expect("Sample expected");
                    let values = [T::from_sample(lval), T::from_sample(rval)];

                    for (k, sample) in frame.iter_mut().enumerate() {
                        *sample = values[k & 1];
                    }
                }

                // futures::executor::block_on(async {
                //     let mut renderer = renderer.lock().await;
                //     renderer.render(lbuf_slice, rbuf_slice);
                // });
                // for (n, frame) in data.chunks_mut(channels).enumerate() {
                //     let values = [T::from_sample(lbuf_slice[n]), T::from_sample(rbuf_slice[n])];
                //     for (k, sample) in frame.iter_mut().enumerate() {
                //         *sample = values[k & 1];
                //     }
                // }
            },
            err_fn,
            None,
        )
        .map_err(Error::BuildStream)?;
    stream.play().map_err(Error::PlayStream)?;
    Ok(stream)
}

fn create_buffers(buffer_size: usize) -> ((BufferTx, BufferRx), (BufferTx, BufferRx)) {
    let lbuf = ringbuf::HeapRb::<f32>::new(buffer_size);
    let rbuf = ringbuf::HeapRb::<f32>::new(buffer_size);
    (lbuf.split(), rbuf.split())
}

#[cfg(test)]
mod tests {
    // use crate::{audio::info, control, path::VirtualPaths, render};
    // use std::sync::Arc;
    // use tokio::sync::Mutex;

    #[test]
    fn controller() {
        //TODO: make new test

        // let (_midi_tx, midi_rx) = crate::midi::create_channel(1);
        // let (_req_tx, req_rx) = render::command::create_request_channel(1);
        // let (_dm_ctr_tx, dm_ctr_rx) = control::create_control_channel(1);
        // let renderer = Arc::new(Mutex::new(super::Renderer::new(
        //     midi_rx,
        //     req_rx,
        //     dm_ctr_rx,
        //     VirtualPaths::default(),
        // )));
        // let mut audio_ctr = super::Controller::new(renderer);
        // let host = info::get_default_host();
        // let device_name = info::get_default_output_device_name(&host).expect(concat!(
        //     "The host doesn't provide any default output device. ",
        //     "(Not an error in the code, it's just not available on your device.)"
        // ));
        // let host_name = info::get_default_host_name();
        // audio_ctr
        //     .connect_to_output_device(&host_name, &device_name)
        //     .unwrap();
        // assert!(audio_ctr.stream.is_some());
    }
}
