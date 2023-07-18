package com.ospx.opus;

import java.io.IOException;
import org.scijava.nativelib.NativeLoader;

public class OpusCodec {
    private static OpusCodec INSTANCE;

    public static OpusCodec instance() {
        if (INSTANCE == null)
            return INSTANCE = new OpusCodec();
        return INSTANCE;
    }

    public int frameSize = 960;
    public int sampleRate = 48000;
    public int channels = 1;
    public int bitrate = 64000;
    public int maxFrameSize = 6 * 960;
    public int maxPacketSize = 3 * 1276;

    private OpusCodec() {
        try {
            NativeLoader.loadLibrary("opus_cp");
        } catch (IOException e) {
            throw new RuntimeException("Exception while loading opus_cp binary",e);
        }
    }

    public int getFrameSize() {
        return this.frameSize;
    }

    public int getSampleRate() {
        return this.sampleRate;
    }

    public int getChannels() {
        return this.channels;
    }

    public int getBitrate() {
        return this.bitrate;
    }

    public int getMaxFrameSize() {
        return this.maxFrameSize;
    }

    public int getMaxPacketSize() {
        return this.maxPacketSize;
    }

    /**
     * Encodes a chunk of raw PCM data.
     *
     * @param bytes data to encode. Must have a length of CHANNELS * FRAMESIZE * 2.
     * @return encoded data
     *         <p>
     *         throws {@link IllegalArgumentException} if bytes has an invalid
     *         length
     */
    public byte[] encodeFrame(byte[] bytes) {
        return this.encodeFrame(bytes, 0, bytes.length);
    }

    private native byte[] encodeFrame(byte[] in, int offset, int length);

    /**
     * Decodes a chunk of opus encoded pcm data.
     *
     * @param bytes data to decode. Length may vary because the less complex the
     *              encoded pcm data is, the compressed data size is smaller.
     * @return encoded data.
     */
    private native byte[] decodeFrame(byte[] out);
}