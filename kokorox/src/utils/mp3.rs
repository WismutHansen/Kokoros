use mp3lame_encoder::{Builder, FlushNoGap, Id3Tag, MonoPcm};

pub fn pcm_to_mp3(pcm_data: &[f32], sample_rate: u32) -> Result<Vec<u8>, std::io::Error> {
    let mut mp3_encoder = Builder::new().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Encoder init failed".to_string(),
    ))?;

    mp3_encoder.set_num_channels(1).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Set channels failed: {:?}", e),
        )
    })?;
    mp3_encoder.set_sample_rate(sample_rate).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Set sample rate failed: {:?}", e),
        )
    })?;
    mp3_encoder
        .set_brate(mp3lame_encoder::Bitrate::Kbps192)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Set bitrate failed: {:?}", e),
            )
        })?;
    mp3_encoder
        .set_quality(mp3lame_encoder::Quality::Best)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Set quality failed: {:?}", e),
            )
        })?;

    let _ = mp3_encoder.set_id3_tag(Id3Tag {
        title: b"Generated Audio",
        artist: b"TTS Model",
        album: b"Synthesized Speech",
        year: b"Current year",
        album_art: &[],
        comment: b"Generated by TTS",
    });

    let mut mp3_encoder = mp3_encoder.build().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Build encoder failed: {:?}", e),
        )
    })?;

    let pcm_i16: Vec<i16> = pcm_data
        .iter()
        .map(|&x| (x * i16::MAX as f32) as i16)
        .collect();
    let pcm = MonoPcm(&pcm_i16);

    let mut mp3_out_buffer = Vec::new();
    mp3_out_buffer.reserve(mp3lame_encoder::max_required_buffer_size(pcm.0.len()));

    let encoded_size = mp3_encoder
        .encode(pcm, mp3_out_buffer.spare_capacity_mut())
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Encoding failed: {:?}", e),
            )
        })?;

    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    let flush_size = mp3_encoder
        .flush::<FlushNoGap>(mp3_out_buffer.spare_capacity_mut())
        .map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Flush failed: {:?}", e))
        })?;
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(flush_size));
    }

    Ok(mp3_out_buffer)
}
