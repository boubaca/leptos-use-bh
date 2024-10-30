use crate::core::MaybeRwSignal;
use default_struct_builder::DefaultBuilder;
use js_sys::{Object, Reflect};
use leptos::*;
use wasm_bindgen::{JsCast, JsValue};

/// Reactive [`mediaDevices.getUserMedia`](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getUserMedia) streaming.
///
/// ## Demo
///
/// [Link to Demo](https://github.com/Synphonyte/leptos-use/tree/main/examples/use_user_media)
///
/// ## Usage
///
/// ```
/// # use leptos::*;
/// # use leptos_use::{use_user_media, UseUserMediaReturn};
/// #
/// # #[component]
/// # fn Demo() -> impl IntoView {
/// let video_ref = create_node_ref::<leptos::html::Video>();
///
/// let UseUserMediaReturn { stream, start, .. } = use_user_media();
///
/// start();
///
/// create_effect(move |_|
///     video_ref.get().map(|v| {
///         match stream.get() {
///             Some(Ok(s)) => v.set_src_object(Some(&s)),
///             Some(Err(e)) => logging::error!("Failed to get media stream: {:?}", e),
///             None => logging::log!("No stream yet"),
///         }
///     })
/// );
///
/// view! { <video node_ref=video_ref controls=false autoplay=true muted=true></video> }
/// # }
/// ```
///
/// ## Server-Side Rendering
///
/// On the server calls to `start` or any other way to enable the stream will be ignored
/// and the stream will always be `None`.
pub fn use_user_media() -> UseUserMediaReturn<impl Fn() + Clone, impl Fn() + Clone> {
    use_user_media_with_options(UseUserMediaOptions::default())
}

/// Version of [`use_user_media`] that takes a `UseUserMediaOptions`. See [`use_user_media`] for how to use.
pub fn use_user_media_with_options(
    options: UseUserMediaOptions,
) -> UseUserMediaReturn<impl Fn() + Clone, impl Fn() + Clone> {
    let UseUserMediaOptions {
        enabled,
        video,
        audio,
        ..
    } = options;

    let (enabled, set_enabled) = enabled.into_signal();

    let (stream, set_stream) = create_signal(None::<Result<web_sys::MediaStream, JsValue>>);

    let _start = {
        let audio = audio.clone();
        let video = video.clone();

        move || async move {
            #[cfg(not(feature = "ssr"))]
            {
                if stream.get_untracked().is_some() {
                    return;
                }

                let stream = create_media(Some(video), Some(audio)).await;

                set_stream.update(|s| *s = Some(stream));
            }

            #[cfg(feature = "ssr")]
            {
                let _ = video;
                let _ = audio;
            }
        }
    };

    let _stop = move || {
        if let Some(Ok(stream)) = stream.get_untracked() {
            for track in stream.get_tracks() {
                track.unchecked_ref::<web_sys::MediaStreamTrack>().stop();
            }
        }

        set_stream.set(None);
    };

    let start = {
        let _start = _start.clone();
        move || {
            #[cfg(not(feature = "ssr"))]
            {
                spawn_local({
                    let _start = _start.clone();
                    async move {
                        _start().await;
                        stream.with_untracked(move |stream| {
                            if let Some(Ok(_)) = stream {
                                set_enabled.set(true);
                            }
                        });
                    }
                });
            }
        }
    };

    let stop = move || {
        _stop();
        set_enabled.set(false);
    };

    let _ = {
        watch(
            move || enabled.get(),
            move |enabled, _, _| {
                if *enabled {
                    spawn_local({
                        let _start = _start.clone();
                        async move {
                            _start().await;
                        }
                    });
                } else {
                    _stop();
                }
            },
            true,
        )
    };
    UseUserMediaReturn {
        stream: stream.into(),
        start,
        stop,
        enabled,
        set_enabled,
    }
}

#[cfg(not(feature = "ssr"))]
async fn create_media(
    video: Option<VideoConstraints>,
    audio: Option<AudioConstraints>,
) -> Result<web_sys::MediaStream, JsValue> {
    use crate::js_fut;
    use crate::use_window::use_window;

    let media = use_window()
        .navigator()
        .ok_or_else(|| JsValue::from_str("Failed to access window.navigator"))
        .and_then(|n| n.media_devices())?;

    let constraints = web_sys::MediaStreamConstraints::new();
    if let Some(video_shadow_constraints) = video {
        match video_shadow_constraints {
            VideoConstraints::Bool(b) => constraints.set_video(&JsValue::from(b)),
            VideoConstraints::Constraints(VideoTrackConstraints {
                device_id,
                facing_mode,
                frame_rate,
                height,
                width,
                viewport_height,
                viewport_width,
                viewport_offset_x,
                viewport_offset_y,
            }) => {
                let video_constraints = web_sys::MediaTrackConstraints::new();

                if let Some(value) = device_id {
                    video_constraints.set_device_id(&value.to_jsvalue());
                }

                if let Some(value) = facing_mode {
                    video_constraints.set_facing_mode(&value.to_jsvalue());
                }

                if let Some(value) = frame_rate {
                    video_constraints.set_frame_rate(&value.to_jsvalue());
                }

                if let Some(value) = height {
                    video_constraints.set_height(&value.to_jsvalue());
                }

                if let Some(value) = width {
                    video_constraints.set_width(&value.to_jsvalue());
                }

                if let Some(value) = viewport_height {
                    video_constraints.set_viewport_height(&value.to_jsvalue());
                }

                if let Some(value) = viewport_width {
                    video_constraints.set_viewport_width(&value.to_jsvalue());
                }
                if let Some(value) = viewport_offset_x {
                    video_constraints.set_viewport_offset_x(&value.to_jsvalue());
                }

                if let Some(value) = viewport_offset_y {
                    video_constraints.set_viewport_offset_y(&value.to_jsvalue());
                }

                constraints.set_video(&JsValue::from(video_constraints));
            }
        }
    }
    if let Some(audio_shadow_constraints) = audio {
        match audio_shadow_constraints {
            AudioConstraints::Bool(b) => constraints.set_audio(&JsValue::from(b)),
            AudioConstraints::Constraints(AudioTrackConstraints {
                device_id,
                auto_gain_control,
                channel_count,
                echo_cancellation,
                noise_suppression
            }) => {
                let audio_constraints = web_sys::MediaTrackConstraints::new();

                if let Some(value) = device_id {
                    audio_constraints.set_device_id(&JsValue::from(&value.to_jsvalue()));
                }
                if let Some(value) = auto_gain_control {
                    audio_constraints.set_auto_gain_control(&JsValue::from(&value.to_jsvalue()));
                }
                if let Some(value) = channel_count {
                    audio_constraints.set_channel_count(&JsValue::from(&value.to_jsvalue()));
                }
                if let Some(value) = echo_cancellation {
                    audio_constraints.set_echo_cancellation(&JsValue::from(&value.to_jsvalue()));
                }
                if let Some(value) = noise_suppression {
                    audio_constraints.set_noise_suppression(&JsValue::from(&value.to_jsvalue()));
                }

                constraints.set_audio(&JsValue::from(audio_constraints));
            }
        }
    }

    let promise = media.get_user_media_with_constraints(&constraints)?;
    let res = js_fut!(promise).await?;

    Ok::<_, JsValue>(web_sys::MediaStream::unchecked_from_js(res))
}

/// Options for [`use_user_media_with_options`].
///
/// Either or both constraints must be specified.
/// If the browser cannot find all media tracks with the specified types that meet the constraints given,
/// then the returned promise is rejected with `NotFoundError`
#[derive(DefaultBuilder, Clone, Debug)]
pub struct UseUserMediaOptions {
    /// If the stream is enabled. Defaults to `false`.
    enabled: MaybeRwSignal<bool>,
    /// Constraint parameter describing video media type requested
    /// The default value is `false`.
    #[builder(into)]
    video: VideoConstraints,
    /// Constraint parameter describing audio media type requested
    /// The default value is `false`.
    #[builder(into)]
    audio: AudioConstraints,
}

impl Default for UseUserMediaOptions {
    fn default() -> Self {
        Self {
            enabled: false.into(),
            video: false.into(),
            audio: false.into(),
        }
    }
}

/// Return type of [`use_user_media`].
#[derive(Clone)]
pub struct UseUserMediaReturn<StartFn, StopFn>
where
    StartFn: Fn() + Clone,
    StopFn: Fn() + Clone,
{
    /// The current [`MediaStream`](https://developer.mozilla.org/en-US/docs/Web/API/MediaStream) if it exists.
    /// Initially this is `None` until `start` resolved successfully.
    /// In case the stream couldn't be started, for example because the user didn't grant permission,
    /// this has the value `Some(Err(...))`.
    pub stream: Signal<Option<Result<web_sys::MediaStream, JsValue>>>,

    /// Starts the screen streaming. Triggers the ask for permission if not already granted.
    pub start: StartFn,

    /// Stops the screen streaming
    pub stop: StopFn,

    /// A value of `true` indicates that the returned [`MediaStream`](https://developer.mozilla.org/en-US/docs/Web/API/MediaStream)
    /// has resolved successfully and thus the stream is enabled.
    pub enabled: Signal<bool>,

    /// A value of `true` is the same as calling `start()` whereas `false` is the same as calling `stop()`.
    pub set_enabled: WriteSignal<bool>,
}

#[derive(Clone, Copy, Debug)]
pub enum ConstraintExactIdeal<T> {
    Single(Option<T>),
    ExactIdeal { exact: Option<T>, ideal: Option<T> },
}

#[derive(Clone, Copy, Debug)]
pub enum ConstraintRange<T> {
    Single(Option<T>),
    Range {
        min: Option<T>,
        max: Option<T>,
        exact: Option<T>,
        ideal: Option<T>,
    },
}

#[derive(Clone, Debug)]
pub enum ConstraintDeviceId {
    Single(Option<String>),
    Multiple(Vec<String>),
}

impl<T> ConstraintExactIdeal<T> {
    pub fn default() -> Self {
        ConstraintExactIdeal::Single(None)
    }

    pub fn exact(mut self, value: T) -> Self {
        match &mut self {
            ConstraintExactIdeal::ExactIdeal {
                exact: ref mut e, ..
            } => {
                *e = Some(value);
            }
            _ => {}
        }
        self
    }

    pub fn ideal(mut self, value: T) -> Self {
        match &mut self {
            ConstraintExactIdeal::ExactIdeal {
                ideal: ref mut i, ..
            } => {
                *i = Some(value);
            }
            _ => {}
        }
        self
    }
}

impl<T> ConstraintRange<T>
where
    T: Clone + std::fmt::Debug,
{
    pub fn new(value: Option<T>) -> Self {
        ConstraintRange::Single(value)
    }

    pub fn default() -> Self {
        ConstraintRange::Range {
            min: None,
            max: None,
            exact: None,
            ideal: None,
        }
    }

    pub fn min(mut self, value: T) -> Self {
        if let ConstraintRange::Range { ref mut min, .. } = self {
            *min = Some(value);
        }
        self
    }

    pub fn max(mut self, value: T) -> Self {
        if let ConstraintRange::Range { ref mut max, .. } = self {
            *max = Some(value);
        }
        self
    }

    pub fn exact(mut self, value: T) -> Self {
        match &mut self {
            ConstraintRange::Range { ref mut exact, .. } => {
                *exact = Some(value);
            }
            _ => {}
        }
        self
    }

    pub fn ideal(mut self, value: T) -> Self {
        match &mut self {
            ConstraintRange::Range { ref mut ideal, .. } => {
                *ideal = Some(value);
            }
            _ => {}
        }
        self
    }
}

impl<T> Default for ConstraintExactIdeal<T>
where
    T: Default,
{
    fn default() -> Self {
        ConstraintExactIdeal::Single(Some(T::default()))
    }
}

impl<T> Default for ConstraintRange<T>
where
    T: Default,
{
    fn default() -> Self {
        ConstraintRange::Single(Some(T::default()))
    }
}

impl ConstraintFacingMode {
    pub fn to_jsvalue(&self) -> JsValue {
        match self {
            ConstraintExactIdeal::Single(value) => {
                JsValue::from_str(value.clone().unwrap().as_str())
            }
            ConstraintExactIdeal::ExactIdeal { exact, ideal } => {
                let obj = Object::new();

                if let Some(value) = exact {
                    Reflect::set(
                        &obj,
                        &JsValue::from_str("exact"),
                        &JsValue::from_str(value.as_str()),
                    )
                    .unwrap();
                }
                if let Some(value) = ideal {
                    Reflect::set(
                        &obj,
                        &JsValue::from_str("ideal"),
                        &JsValue::from_str(value.as_str()),
                    )
                    .unwrap();
                }

                JsValue::from(obj)
            }
        }
    }
}

impl<T> ConstraintRange<T>
where
    T: Into<JsValue> + Clone,
{
    pub fn to_jsvalue(&self) -> JsValue {
        match self {
            ConstraintRange::Single(value) => JsValue::from(value.clone().unwrap().into()),
            ConstraintRange::Range {
                min,
                max,
                exact,
                ideal,
            } => {
                let obj = Object::new();

                if let Some(min_value) = min {
                    Reflect::set(&obj, &JsValue::from_str("min"), &min_value.clone().into())
                        .unwrap();
                }
                if let Some(max_value) = max {
                    Reflect::set(&obj, &JsValue::from_str("max"), &max_value.clone().into())
                        .unwrap();
                }
                if let Some(value) = exact {
                    Reflect::set(&obj, &JsValue::from_str("exact"), &value.clone().into()).unwrap();
                }
                if let Some(value) = ideal {
                    Reflect::set(&obj, &JsValue::from_str("ideal"), &value.clone().into()).unwrap();
                }

                JsValue::from(obj)
            }
        }
    }
}

impl<T> ConstraintExactIdeal<T>
where
    T: Into<JsValue> + Clone,
{
    pub fn to_jsvalue(&self) -> JsValue {
        match self {
            ConstraintExactIdeal::Single(value) => JsValue::from(value.clone().unwrap().into()),
            ConstraintExactIdeal::ExactIdeal { exact, ideal } => {
                let obj = Object::new();

                if let Some(value) = exact {
                    Reflect::set(&obj, &JsValue::from_str("exact"), &value.clone().into()).unwrap();
                }
                if let Some(value) = ideal {
                    Reflect::set(&obj, &JsValue::from_str("ideal"), &value.clone().into()).unwrap();
                }

                JsValue::from(obj)
            }
        }
    }
}

impl ConstraintDeviceId {
    pub fn to_jsvalue(&self) -> JsValue {
        match self {
            ConstraintDeviceId::Single(value) => JsValue::from(value.clone().unwrap()),
            ConstraintDeviceId::Multiple(value) => JsValue::from(value.clone()),
        }
    }
}

pub type ConstraintDouble = ConstraintRange<f64>;
pub type ConstraintULong = ConstraintRange<u32>;
pub type ConstraintBool = ConstraintExactIdeal<bool>;

pub type ConstraintFacingMode = ConstraintExactIdeal<FacingMode>;

#[derive(Clone, Copy, Debug)]
pub enum FacingMode {
    User,
    Environment,
    Left,
    Right,
}

impl FacingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            FacingMode::User => "user",
            FacingMode::Environment => "environment",
            FacingMode::Left => "left",
            FacingMode::Right => "right",
        }
    }
}

#[derive(Clone, Debug)]
pub enum AudioConstraints {
    Bool(bool),
    Constraints(AudioTrackConstraints),
}

#[derive(Clone, Debug)]
pub enum VideoConstraints {
    Bool(bool),
    Constraints(VideoTrackConstraints),
}

impl From<bool> for VideoConstraints {
    fn from(value: bool) -> Self {
        VideoConstraints::Bool(value)
    }
}

impl From<VideoTrackConstraints> for VideoConstraints {
    fn from(value: VideoTrackConstraints) -> Self {
        VideoConstraints::Constraints(value)
    }
}

impl From<bool> for AudioConstraints {
    fn from(value: bool) -> Self {
        AudioConstraints::Bool(value)
    }
}

impl From<AudioTrackConstraints> for AudioConstraints {
    fn from(value: AudioTrackConstraints) -> Self {
        AudioConstraints::Constraints(value)
    }
}

impl From<FacingMode> for ConstraintFacingMode {
    fn from(value: FacingMode) -> Self {
        ConstraintFacingMode::Single(Some(value)) // Wrap FacingMode into the Single variant
    }
}

impl From<f64> for ConstraintRange<f64> {
    fn from(value: f64) -> Self {
        ConstraintRange::Single(Some(value)) // Wrap `f64` in the `Single` variant
    }
}

impl From<u32> for ConstraintRange<u32> {
    fn from(value: u32) -> Self {
        ConstraintRange::Single(Some(value)) // Wrap `u32` in the `Single` variant
    }
}

impl From<bool> for ConstraintExactIdeal<bool> {
    fn from(value: bool) -> Self {
        ConstraintExactIdeal::Single(Some(value)) // Wrap `bool` in the `Single` variant
    }
}

impl From<&'static str> for ConstraintExactIdeal<&'static str> {
    fn from(value: &'static str) -> Self {
        ConstraintExactIdeal::Single(Some(value)) // Wrap `&'static str` in the `Single` variant
    }
}

pub trait IntoDeviceIds<M> {
    fn into_device_ids(self) -> ConstraintDeviceId;
}

impl<T> IntoDeviceIds<String> for T
where
    T: Into<String>,
{
    fn into_device_ids(self) -> ConstraintDeviceId {
        ConstraintDeviceId::Single(Some(self.into()))
    }
}

pub struct VecMarker;

impl<T, I> IntoDeviceIds<VecMarker> for T
where
    T: IntoIterator<Item = I>,
    I: Into<String>,
{
    fn into_device_ids(self) -> ConstraintDeviceId {
        ConstraintDeviceId::Multiple(self.into_iter().map(Into::into).collect())
    }
}

#[derive(Default, Clone, Debug)]
pub struct AudioTrackConstraints {
    pub device_id: Option<ConstraintDeviceId>,
    pub auto_gain_control: Option<ConstraintBool>,
    pub channel_count: Option<ConstraintULong>,
    pub echo_cancellation: Option<ConstraintBool>,
    pub noise_suppression: Option<ConstraintBool>,
}

impl AudioTrackConstraints {
    pub fn new() -> Self {
        AudioTrackConstraints::default()
    }

    pub fn device_id<M>(mut self, value: impl IntoDeviceIds<M>) -> Self {
        self.device_id = Some(value.into_device_ids());
        self
    }

    pub fn auto_gain_control<T: Into<ConstraintBool>>(mut self, value: T) -> Self {
        self.auto_gain_control = Some(value.into());
        self
    }

    pub fn channel_count<T: Into<ConstraintULong>>(mut self, value: T) -> Self {
        self.channel_count = Some(value.into());
        self
    }

    pub fn echo_cancellation<T: Into<ConstraintBool>>(mut self, value: T) -> Self {
        self.echo_cancellation = Some(value.into());
        self
    }

    pub fn noise_suppression<T: Into<ConstraintBool>>(mut self, value: T) -> Self {
        self.noise_suppression = Some(value.into());
        self
    }
}

#[derive(Default, Clone, Debug)]
pub struct VideoTrackConstraints {
    pub device_id: Option<ConstraintDeviceId>,

    // TODO: implement array of device ids
    // pub device_ids: Option<Vec<ConstrainDOMString>>,
    pub facing_mode: Option<ConstraintFacingMode>,
    pub frame_rate: Option<ConstraintDouble>,
    pub height: Option<ConstraintULong>,
    pub width: Option<ConstraintULong>,
    pub viewport_offset_x: Option<ConstraintULong>,
    pub viewport_offset_y: Option<ConstraintULong>,
    pub viewport_height: Option<ConstraintULong>,
    pub viewport_width: Option<ConstraintULong>,
}

impl VideoTrackConstraints {
    pub fn new() -> Self {
        VideoTrackConstraints::default() // Start with default empty constraints
    }

    pub fn device_id<M>(mut self, value: impl IntoDeviceIds<M>) -> Self {
        self.device_id = Some(value.into_device_ids());
        self
    }

    // TODO: implement array of device ids

    // pub fn device_ids(mut self, values: Vec<&'static str>) -> Self {
    //     let constraints = values
    //       .into_iter()
    //       .map(|value| ConstraintExactIdeal::Single(Some(value)))
    //       .collect::<Vec<ConstraintDOMString>>();
    //
    //     self.device_ids = Some(constraints);
    //     self
    // }

    pub fn facing_mode<T: Into<ConstraintFacingMode>>(mut self, value: T) -> Self {
        self.facing_mode = Some(value.into());
        self
    }

    pub fn frame_rate<T: Into<ConstraintDouble>>(mut self, value: T) -> Self {
        self.frame_rate = Some(value.into());
        self
    }

    pub fn height<T: Into<ConstraintULong>>(mut self, value: T) -> Self {
        self.height = Some(value.into());
        self
    }
    pub fn width<T: Into<ConstraintULong>>(mut self, value: T) -> Self {
        self.width = Some(value.into());
        self
    }

    pub fn viewport_offset_x<T: Into<ConstraintULong>>(mut self, value: T) -> Self {
        self.viewport_offset_x = Some(value.into());
        self
    }

    pub fn viewport_offset_y<T: Into<ConstraintULong>>(mut self, value: T) -> Self {
        self.viewport_offset_y = Some(value.into());
        self
    }

    pub fn viewport_height<T: Into<ConstraintULong>>(mut self, value: T) -> Self {
        self.viewport_height = Some(value.into());
        self
    }

    pub fn viewport_width<T: Into<ConstraintULong>>(mut self, value: T) -> Self {
        self.viewport_width = Some(value.into());
        self
    }
}
