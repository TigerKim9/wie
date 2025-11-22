use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class net.wie.WieAudioClip
pub struct WieAudioClip;

impl WieAudioClip {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/WieAudioClip",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["com/skt/m/AudioClip"],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("open", "([BII)V", Self::open, Default::default()),
                JavaMethodProto::new("play", "()V", Self::play, Default::default()),
                JavaMethodProto::new("loop", "()V", Self::r#loop, Default::default()),
                JavaMethodProto::new("stop", "()V", Self::stop, Default::default()),
                JavaMethodProto::new("pause", "()V", Self::pause, Default::default()),
                JavaMethodProto::new("resume", "()V", Self::resume, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("audioHandle", "I", Default::default()),
                JavaFieldProto::new("isOpen", "Z", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, _name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::<init>({this:?}, {_name:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        // Initialize fields
        jvm.put_field(&mut this, "audioHandle", "I", -1i32).await?;
        jvm.put_field(&mut this, "isOpen", "Z", false).await?;

        Ok(())
    }

    async fn open(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        buffer_size: i32,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::open({this:?}, {data:?}, {offset}, {buffer_size})");

        // Read audio data from byte array
        let array_data: alloc::vec::Vec<i8> = jvm.load_array(&data, offset as _, buffer_size as _).await?;

        // Convert i8 to u8
        let audio_data: alloc::vec::Vec<u8> = array_data.iter().map(|&x| x as u8).collect();

        // Load SMAF audio
        let audio_handle = context.system().audio().load_smaf(&audio_data).unwrap();

        // Store audio handle
        jvm.put_field(&mut this, "audioHandle", "I", audio_handle as i32).await?;
        jvm.put_field(&mut this, "isOpen", "Z", true).await?;

        Ok(())
    }

    async fn play(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::play({this:?})");

        let is_open: bool = jvm.get_field(&this, "isOpen", "Z").await?;
        if !is_open {
            return Err(jvm.exception("java/io/IOException", "AudioClip not opened").await);
        }

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        let system = context.system();

        system.audio().play(system, audio_handle as u32).unwrap();

        Ok(())
    }

    async fn r#loop(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::loop({this:?})");

        let is_open: bool = jvm.get_field(&this, "isOpen", "Z").await?;
        if !is_open {
            return Err(jvm.exception("java/io/IOException", "AudioClip not opened").await);
        }

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        let system = context.system();

        // Use play_with_loop to enable looping
        system.audio().play_with_loop(system, audio_handle as u32, true).unwrap();

        Ok(())
    }

    async fn stop(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::stop({this:?})");

        let is_open: bool = jvm.get_field(&this, "isOpen", "Z").await?;
        if !is_open {
            return Err(jvm.exception("java/io/IOException", "AudioClip not opened").await);
        }

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;
        let system = context.system();

        system.audio().stop(system, audio_handle as u32).unwrap();

        Ok(())
    }

    async fn pause(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::pause({this:?})");

        let is_open: bool = jvm.get_field(&this, "isOpen", "Z").await?;
        if !is_open {
            return Err(jvm.exception("java/io/IOException", "AudioClip not opened").await);
        }

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;

        context.system().audio().pause(audio_handle as u32).ok();

        Ok(())
    }

    async fn resume(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::resume({this:?})");

        let is_open: bool = jvm.get_field(&this, "isOpen", "Z").await?;
        if !is_open {
            return Err(jvm.exception("java/io/IOException", "AudioClip not opened").await);
        }

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;

        context.system().audio().resume(audio_handle as u32).ok();

        Ok(())
    }

    async fn close(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieAudioClip::close({this:?})");

        let is_open: bool = jvm.get_field(&this, "isOpen", "Z").await?;
        if !is_open {
            return Ok(()); // Already closed, silently succeed
        }

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;

        context.system().audio().close(audio_handle as u32).unwrap();

        // Mark as closed
        jvm.put_field(&mut this, "isOpen", "Z", false).await?;
        jvm.put_field(&mut this, "audioHandle", "I", -1i32).await?;

        Ok(())
    }
}
