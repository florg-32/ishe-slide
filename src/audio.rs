use std::time::Duration;

#[cfg(feature = "web")]
mod web {
    use gloo::timers::callback::Timeout;
    use js_sys::wasm_bindgen::JsValue;
    use std::time::Duration;
    use web_sys::OscillatorType;

    pub fn play_sine_for(freq: f32, end_freq: f32, duration: Duration) -> Result<(), JsValue> {
        let ctx = web_sys::AudioContext::new()?;
        let osc = ctx.create_oscillator()?;
        let volume = ctx.create_gain()?;

        osc.set_type(OscillatorType::Sine);
        osc.frequency().set_value(freq);
        osc.frequency()
            .linear_ramp_to_value_at_time(end_freq, ctx.current_time() + duration.as_secs_f64())?;
        volume.gain().set_value(1.0);
        osc.connect_with_audio_node(&volume)?;
        volume.connect_with_audio_node(&ctx.destination())?;

        osc.start()?;
        Timeout::new(duration.as_millis().try_into().unwrap(), move || {
            volume
                .gain()
                .set_target_at_time(0.01, ctx.current_time(), 0.015)
                .unwrap();
            Timeout::new(150, move || {
                osc.stop().unwrap();
                let _ = ctx.close().unwrap();
            })
            .forget();
        })
        .forget();

        Ok(())
    }
}

#[cfg(feature = "web")]
pub async fn play_pattern() {
    web::play_sine_for(400.0, 1000.0, Duration::from_millis(500)).unwrap();
    web::play_sine_for(2000.0, 3000.0, Duration::from_millis(500)).unwrap();
}

#[cfg(feature = "mobile")]
pub async fn play_pattern() {
    use rodio::Source;
    let stream = rodio::OutputStreamBuilder::from_default_device()
        .unwrap()
        .open_stream()
        .unwrap();
    stream
        .mixer()
        .add(rodio::source::chirp(44100, 480.0, 2000.0, Duration::from_secs(1)).amplify(0.8));

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}

#[cfg(not(any(feature = "web", feature = "mobile")))]
pub async fn play_pattern() {}
