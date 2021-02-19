use std::ptr;
use std::ops::Deref;

use log::trace;

use crate::TryIntoJs;
use crate::JSValue;
use crate::sys::{napi_value, napi_ref, napi_env};
use crate::val::JsEnv;
use crate::NjError;

/// pass rust byte arry as Node.js ArrayBuffer
pub struct ArrayBuffer {
    data: Vec<u8>,
}

use std::fmt;
use std::fmt::Debug;

impl Debug for ArrayBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("ArrayBuffer len: {}", self.data.len()))
    }
}

impl ArrayBuffer {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    extern "C" fn finalize_buffer(
        _env: napi_env,
        _finalize_data: *mut ::std::os::raw::c_void,
        finalize_hint: *mut ::std::os::raw::c_void,
    ) {
        trace!("finalize array buffer");
        unsafe {
            // use hint to reconstruct box instead of finalize data
            let ptr: *mut Vec<u8> = finalize_hint as *mut Vec<u8>;
            let _rust = Box::from_raw(ptr);
        }
    }
}

impl TryIntoJs for ArrayBuffer {
    fn try_to_js(self, js_env: &JsEnv) -> Result<napi_value, NjError> {
        let len = self.data.len();

        let box_data = Box::new(self.data);

        let mut napi_buffer = ptr::null_mut();

        // get pointer to vec's buffer
        let data_buffer = box_data.as_ptr();

        // get raw pointer to box, this will be used to reconstruct box
        let data_box_ptr = Box::into_raw(box_data) as *mut core::ffi::c_void;

        crate::napi_call_result!(crate::sys::napi_create_external_arraybuffer(
            js_env.inner(),
            data_buffer as *mut core::ffi::c_void,
            len,
            Some(Self::finalize_buffer),
            data_box_ptr,
            &mut napi_buffer
        ))?;

        Ok(napi_buffer)
    }
}

pub struct Buffer {
    data: Vec<u8>
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Buffer len: {}", self.data.len()))
    }
}

impl Buffer {

    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

impl TryIntoJs for Buffer {

    fn try_to_js(self, js_env: &JsEnv) -> Result<napi_value, NjError> {
        let len = self.data.len();

        let mut boxed_slice = self.data.into_boxed_slice();

        let buffer: *mut core::ffi::c_void = boxed_slice.as_mut_ptr() as *mut core::ffi::c_void;

        let mut napi_buffer = ptr::null_mut();

        crate::napi_call_result!(
            crate::sys::napi_create_buffer_copy(
                js_env.inner(),
                len,
                buffer,
                ptr::null_mut(),
                &mut napi_buffer
            )
        )?;

        Ok(napi_buffer)

    }
}

impl JSValue<'_> for Buffer {

    fn convert_to_rust(env: &JsEnv, js_value: napi_value) -> Result<Self, NjError> {

        if !env.is_buffer(js_value)? {
            return Err(NjError::Other(format!("Type is not a Buffer")))
        }

        use crate::sys::napi_get_buffer_info;
        use libc::size_t;

        let mut buffer: *mut core::ffi::c_void = ptr::null_mut();
        let buffer_ptr: *mut *mut core::ffi::c_void = &mut buffer;
        let mut size: size_t = 0;

        crate::napi_call_result!(
            napi_get_buffer_info(env.inner(), js_value, buffer_ptr, &mut size)
        )?;

        if size != 0 {
            let boxed_slice = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(*buffer_ptr as *mut u8, size)) };

            let vec_buffer: Vec<u8> = boxed_slice[0..size].into();

            Box::leak(boxed_slice);

            Ok(Buffer::new(vec_buffer))
        } else {
            Ok(Buffer::new(vec!()))
        }
    }

}

impl<'a> JSValue<'a> for &'a [u8] {
    fn convert_to_rust(env: &'a JsEnv, js_value: napi_value) -> Result<Self, NjError> {
        // check if this is really buffer
        if !env.is_buffer(js_value)? {
            return Err(NjError::InvalidType(
                "Buffer".to_owned(),
                env.value_type_string(js_value)?.to_owned(),
            ));
        }

        let buffer = env.get_buffer_info(js_value)?;

        Ok(buffer)
    }
}

/// Rust representation of Nodejs [ArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/ArrayBuffer)
/// This is safe to pass around rest of Rust code since this manages Node.js GC lifecycle.
/// JSArrayBuffer is deference as `&[u8]`
///
/// # Examples
///
/// In this example, JS String is passed as array buffer.  Rust code convert to String and concate with prefix message.
///
/// ```no_run
/// use node_bindgen::derive::node_bindgen;
/// use node_bindgen::core::buffer::JSArrayBuffer;
///
/// #[node_bindgen]
/// fn hello(data: JSArrayBuffer) -> Result<String, NjError> {
///   let message = String::from_utf8(data.to_vec())?;
///    Ok(format!("reply {}", message))
/// }
/// ```
///
/// This can be invoked from Node.js
/// ```text
/// let addon = require('./your_module');
/// console.log(Buffer.from("hello"));
/// ```
pub struct JSArrayBuffer {
    env: JsEnv,
    napi_ref: napi_ref,
    buffer: &'static [u8],
}

impl JSValue<'_> for JSArrayBuffer {
    fn convert_to_rust(env: &JsEnv, napi_value: napi_value) -> Result<Self, NjError> {
        use std::mem::transmute;

        let napi_ref = env.create_reference(napi_value, 1)?;

        // it is oky to transmute as static byte slice since we are managing slice
        let buffer: &'static [u8] =
            unsafe { transmute::<&[u8], &'static [u8]>(env.convert_to_rust(napi_value)?) };
        Ok(Self {
            env: *env,
            napi_ref,
            buffer,
        })
    }
}

impl Drop for JSArrayBuffer {
    fn drop(&mut self) {
        self.env
            .delete_reference(self.napi_ref)
            .expect("reference can't be deleted to array buf");
    }
}

impl Deref for JSArrayBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
