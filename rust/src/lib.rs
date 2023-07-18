use jni::{
    errors::Result as JNIResult,
    objects::{JByteArray, JClass},
    sys::jint,
    JNIEnv,
};
use opus::{Application, Channels, Decoder, Encoder, Result as OPUSResult};
use std::cell::RefCell;

thread_local! {
static ENCODER: RefCell<Option<Encoder>> = RefCell::new(Option::None);
static DECODER: RefCell<Option<Decoder>> = RefCell::new(Option::None);
}

pub struct OpusCodecOptions {
    pub frame_size: i32,
    pub sample_rate: i32,
    pub channels: i32,
    pub bitrate: i32,
    pub max_frame_size: i32,
    pub max_packet_size: i32,
}

pub struct OpusEncodeInfo {}

pub struct OpusDecodeInfo {}

impl OpusCodecOptions {
    pub fn new(
        frame_size: i32,
        sample_rate: i32,
        channels: i32,
        bitrate: i32,
        max_frame_size: i32,
        max_packet_size: i32,
    ) -> OpusCodecOptions {
        OpusCodecOptions {
            frame_size,
            sample_rate,
            channels,
            bitrate,
            max_frame_size,
            max_packet_size,
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_ospx_opus_OpusCodec_encodeFrame<'local>(
    mut env: JNIEnv<'local>,
    mut class: JClass<'local>,
    in_buff: JByteArray<'local>,
    in_buff_offset: jint,
    in_buff_length: jint,
) -> JByteArray<'local> {
    let options = read_opus_config(&mut env, &mut class).unwrap();
    let encoding_buffer = env.convert_byte_array(&in_buff).unwrap();
    let pcm_bytes = encode_frame(
        &options,
        &encoding_buffer,
        in_buff_offset.try_into().unwrap(),
        in_buff_length.try_into().unwrap(),
    );

    env.byte_array_from_slice(&pcm_bytes.unwrap()[..]).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_com_ospx_opus_OpusCodec_decodeFrame<'local>(
    mut env: JNIEnv<'local>,
    mut class: JClass<'local>,
    in_buff: JByteArray<'local>,
) -> JByteArray<'local> {
    let options = read_opus_config(&mut env, &mut class).unwrap();
    let decoding_buffer = env.convert_byte_array(&in_buff).unwrap();
    let pcm_bytes = decode_frame(&options, &decoding_buffer);

    env.byte_array_from_slice(&pcm_bytes.unwrap()[..]).unwrap()
}

pub fn encode_frame(
    options: &OpusCodecOptions,
    encoding_buffer: &Vec<u8>,
    offset: usize,
    length: usize,
) -> OPUSResult<Vec<u8>> {
    ENCODER.with(|encoder_opt| {
        let channels = match options.channels {
            0 | 1 => Channels::Mono,
            2 => Channels::Stereo,
            channels => panic!("Channel {} is not defined!", channels),
        };

        let mut encoder_opt = encoder_opt.borrow_mut();
        let encoder = encoder_opt.get_or_insert_with(|| {
            Encoder::new(options.sample_rate as u32, channels, Application::Audio).unwrap()
        });

        encoder.set_bitrate(opus::Bitrate::Bits(options.bitrate))?;

        let mut input: Vec<i16> =
            Vec::with_capacity((options.frame_size * options.channels * 2) as usize);

        for i in 0..(length / 2) {
            let pcm_index = offset + 2 * i;

            input.push(
                (encoding_buffer[pcm_index + 1] as i16) << 8 | encoding_buffer[pcm_index] as i16,
            );
        }

        let byte_vec = encoder
            .encode_vec(input.as_mut_slice(), options.max_packet_size as usize)
            .unwrap();
        Ok(byte_vec)
    })
}

pub fn decode_frame(options: &OpusCodecOptions, decoding_buffer: &Vec<u8>) -> OPUSResult<Vec<u8>> {
    DECODER.with(|decoder_opt| {
        let channels = match options.channels {
            0 | 1 => Channels::Mono,
            2 => Channels::Stereo,
            channels => panic!("Channel {} is not defined!", channels),
        };

        let mut decoder_opt = decoder_opt.borrow_mut();
        let decoder = decoder_opt
            .get_or_insert_with(|| Decoder::new(options.sample_rate as u32, channels).unwrap());

        let mut decoded = Vec::new();

        decoded.resize((options.max_frame_size * options.channels) as usize, 0);

        let frame_size =
            decoder.decode(decoding_buffer.as_slice(), decoded.as_mut_slice(), false)?;
        let mut pcm_bytes = Vec::<u8>::new();

        pcm_bytes.resize(options.channels as usize * frame_size * 2, 0);

        for i in 0..(options.channels as usize * frame_size) {
            pcm_bytes[2 * i] = (decoded[i] & 0xFF) as u8;
            pcm_bytes[2 * i + 1] = ((decoded[i] >> 8) & 0xFF) as u8;
        }

        Ok(pcm_bytes)
    })
}

pub fn read_opus_config(env: &mut JNIEnv, class: &JClass) -> JNIResult<OpusCodecOptions> {
    Ok(OpusCodecOptions::new(
        env.get_field(class, "frameSize", "I")?.i()?,
        env.get_field(class, "sampleRate", "I")?.i()?,
        env.get_field(class, "channels", "I")?.i()?,
        env.get_field(class, "bitrate", "I")?.i()?,
        env.get_field(class, "maxFrameSize", "I")?.i()?,
        env.get_field(class, "maxPacketSize", "I")?.i()?,
    ))
}
