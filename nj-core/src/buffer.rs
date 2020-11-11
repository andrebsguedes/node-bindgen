
use std::ptr;

use log::trace;

use crate::TryIntoJs;
use crate::sys::napi_value;
use crate::sys::napi_env;
use crate::val::JsEnv;
use crate::NjError;
use crate::convert::JSValue;
use crate::napi_call_result;


/// pass rust byte arry as Node.js ArrayBuffer
pub struct ArrayBuffer{
    data: Vec<u8>
}

impl ArrayBuffer {

    pub fn new(data: Vec<u8>) -> Self {
        Self { data}
    }

    extern "C" fn finalize_buffer(_env: napi_env,_finalize_data: *mut ::std::os::raw::c_void,
        finalize_hint: *mut ::std::os::raw::c_void
    ) {

        trace!("finalize array buffer");
        unsafe {
            // use hint to reconstruct box instead of finalize data
            let ptr: *mut Vec<u8> = finalize_hint as *mut Vec<u8>;
            let _rust = Box::from_raw(ptr);
        }

    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

// impl TryIntoJs for ArrayBuffer {
//
//     fn try_to_js(self, js_env: &JsEnv) -> Result<napi_value, NjError> {
//
//         let len = self.data.len();
//
//         let box_data = Box::new(self.data);
//
//         let mut napi_buffer = ptr::null_mut();
//
//         // get pointer to vec's buffer
//         let data_buffer = box_data.as_ptr();
//
//         // get raw pointer to box, this will be used to reconstruct box
//         let data_box_ptr = Box::into_raw(box_data) as *mut core::ffi::c_void;
//
//         crate::napi_call_result!(
//             crate::sys::napi_create_external_buffer(
//                 js_env.inner(),
//                 len,
//                 data_buffer as *mut core::ffi::c_void ,
//                 Some(Self::finalize_buffer),
//                 data_box_ptr,
//                 &mut napi_buffer
//             )
//         )?;
//
//         Ok(napi_buffer)
//
//     }
// }

impl TryIntoJs for ArrayBuffer {

    fn try_to_js(self, js_env: &JsEnv) -> Result<napi_value, NjError> {
        use libc::size_t;

        let len = self.data.len();

        let mut boxed_slice = self.data.into_boxed_slice();

        let mut buffer: *mut core::ffi::c_void = boxed_slice.as_mut_ptr() as *mut core::ffi::c_void;
        // let buffer_ptr: *mut *mut core::ffi::c_void = &mut buffer;

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


impl JSValue for ArrayBuffer {

    fn convert_to_rust(env: &JsEnv, js_value: napi_value) -> Result<Self, NjError> {

        if !env.is_buffer(js_value)? {
            return Err(NjError::Other(format!("Type is not a Buffer")))
        }

        use crate::sys::napi_get_buffer_info;
        use libc::size_t;

        let mut buffer: *mut core::ffi::c_void = ptr::null_mut();
        let buffer_ptr: *mut *mut core::ffi::c_void = &mut buffer;
        let mut size: size_t = 0;

        napi_call_result!(
            napi_get_buffer_info(env.inner(), js_value, buffer_ptr, &mut size)
        )?;

        if size != 0 {

            let boxed_slice = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(*buffer_ptr as *mut u8, size)) };

            let vec_buffer: Vec<u8> = boxed_slice[0..size].into();

            Box::leak(boxed_slice);

            Ok(ArrayBuffer::new(vec_buffer))
        } else {
            Ok(ArrayBuffer::new(vec!()))
        }
    }

}

use std::fmt;
use std::fmt::Debug;

impl Debug for ArrayBuffer {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("ArrayBuffer len: {}",self.data.len()))
    }

}



